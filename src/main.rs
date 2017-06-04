#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate error_chain;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate uuid;
extern crate contacts;

use std::collections::HashMap;
use r2d2::{Pool, PooledConnection, GetTimeout};
use r2d2_postgres::PostgresConnectionManager;
use rocket::Outcome::{Success, Failure, Forward};
use rocket::Request;
use rocket::config::{self, ConfigError};
use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{Form, FromRequest, Outcome};
use rocket::response::Redirect;
use rocket_contrib::{Template, UUID};
use uuid::Uuid;

use contacts::errors::*;
use contacts::models::{Person, Session, Contact, as_brand};


lazy_static! {
    pub static ref DB_POOL: Pool<PostgresConnectionManager> = contacts::create_db_pool().unwrap();
}


pub struct DB(PooledConnection<PostgresConnectionManager>);

impl DB {
    pub fn conn(&self) -> &postgres::Connection {
        &*self.0
    }
}


impl<'a, 'r> FromRequest<'a, 'r> for DB {
    type Error = GetTimeout;
    fn from_request(_: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match DB_POOL.get() {
            Ok(conn) => Success(DB(conn)),
            Err(e) => Failure((Status::ServiceUnavailable, e)),
        }
    }
}


#[derive(Debug, FromForm)]
struct Email {
    email: String,
}


#[post("/login", data="<form>")]
fn login(form: Form<Email>, cookies: &Cookies, db: DB) -> Result<Template> {
    let &Email { ref email } = form.get();

    // if we start an auth flow, kill whatever session may exist
    cookies.remove("session");

    let conn = db.conn();

    let (me, new) = {
        let res: Option<Person> = conn
            .query("SELECT * FROM people WHERE people.email = $1", &[&email])?
            .into_iter()
            .map(Person::from_row)
            .next();

        if let Some(me) = res {
            (me, false)
        } else {
            let new_me: Person = conn
                .query("INSERT INTO people (email) VALUES ($1) RETURNING *", &[&email])?
                .into_iter()
                .map(Person::from_row)
                .next()
                .ok_or("wat")?;
            (new_me, true)
        }
    };

    let login_key: Uuid = conn
        .query("INSERT INTO sessions (account) VALUES ($1) RETURNING login_key", &[&me.id])?
        .into_iter()
        .map(|row| row.get(0))
        .next()
        .ok_or("wat")?;

    contacts::send_login(email, &login_key, new)?;

    let mut context = HashMap::new();
    context.insert("email", email);

    Ok(Template::render("login", &context))
}


#[derive(Debug, FromForm)]
struct LoginKey {
    key: UUID,
}


#[get("/login?<form>")]
fn finish_login(form: LoginKey, cookies: &Cookies, db: DB) -> Result<Redirect> {
    let LoginKey { ref key } = form;

    // if we are in auth flow, kill whatever session may exist
    cookies.remove("session");

    let existing = db.conn()
        .query("SELECT * FROM sessions WHERE login_key = $1", &[&key.into_inner()])?
        .into_iter()
        .map(Session::from_row)
        .next();

    let session = existing.ok_or("missing session")?;

    if session.session_id.is_some() {
        bail!("already got this session whoops");
    }

    let id: Uuid = db.conn()
        .query("UPDATE sessions SET session_id = uuid_generate_v4() WHERE login_key = $1 RETURNING session_id", &[&key.into_inner()])?
        .into_iter()
        .map(|row| row.get(0))
        .next()
        .ok_or("failed to set session_id")?;

    let cookie = Cookie::build("session", id.to_string())
        // .domain(blah)
        .path("/")
        // .secure(true)
        .http_only(true)
        .finish();
    cookies.add(cookie);

    Ok(Redirect::to("/"))
}


#[derive(Debug)]
struct Me(Person);

fn get_me(cookies: &Cookies) -> Result<Option<Me>> {
    let cookie = match cookies.find("session") {
        Some(c) => c,
        None => {
            return Ok(None)
        }
    };
    let claimed_id: Uuid = cookie.value().parse()
        .chain_err(|| "Invalid session cookie")?;

    let db = DB(DB_POOL.get()?);

    let me = db.conn().query(
        "SELECT p.* FROM people p, sessions s WHERE s.account = p.id AND s.session_id = $1",
        &[&claimed_id])?
        .into_iter()
        .map(Person::from_row)
        .next()
        .map(Me);

    Ok(me)
}

impl<'a, 'r> FromRequest<'a, 'r> for Me {
    type Error = Error;

    fn from_request(request: &'a Request<'r>) -> Outcome<Me, Self::Error> {
        match get_me(request.cookies()) {
            Ok(Some(me)) => Success(me),
            Ok(None) => Forward(()),
            Err(e) => Failure((Status::ServiceUnavailable, e)),
        }
    }
}


#[derive(Debug, FromForm)]
struct NewContactForm {
    name: String,
    info: String,
}


#[post("/contacts", data="<form>")]
fn new_contact(form: Form<NewContactForm>, me: Me, db: DB) -> Result<Redirect> {
    let &NewContactForm { ref name, ref info } = form.get();

    db.conn().execute(
        "INSERT INTO contacts (account, name, info) VALUES ($1, $2, $3)",
        &[&me.0.id, &name, &info]
    )?;

    Ok(Redirect::to("/"))
}


#[derive(Debug, FromForm)]
struct DeleteContactForm {
    id: UUID,
    next: Option<String>,
}


#[get("/contacts/delete?<form>")]
fn delete_contact(form: DeleteContactForm, me: Me, db: DB) -> Result<Redirect> {
    let DeleteContactForm { id, next } = form;

    let n = db.conn().execute(
        "DELETE FROM contacts WHERE id = $1 AND account = $2",
        &[&id.into_inner(), &me.0.id]
    )?;

    match n {
        0 => bail!("contact not found"),
        1 => Ok(Redirect::to(&next.unwrap_or("/".into()))),
        _ => bail!("wat"),
    }
}

#[derive(Debug, FromForm)]
#[allow(non_snake_case)]
pub struct StripeSubscribe {
    stripeToken: String,
    stripeTokenType: String,
    stripeEmail: String,
    stripeBillingName: String,
    stripeBillingAddressLine1: String,
    stripeBillingAddressZip: String,
    stripeBillingAddressState: String,
    stripeBillingAddressCity: String,
    stripeBillingAddressCountry: String,
    stripeBillingAddressCountryCode: String,
    stripeShippingName: String,
    stripeShippingAddressLine1: String,
    stripeShippingAddressZip: String,
    stripeShippingAddressState: String,
    stripeShippingAddressCity: String,
    stripeShippingAddressCountry: String,
    stripeShippingAddressCountryCode: String,
}

#[post("/subscriptions", data="<form>")]
fn subscribe(form: Form<StripeSubscribe>, me: Me, db: DB) -> Result<Redirect> {
    let data = form.get();
    db.conn()
        .execute("UPDATE people SET address = ($2, $3, $4, $5, $6, $7) WHERE id = $1", &[
            &me.0.id,
            &data.stripeShippingName,
            &data.stripeShippingAddressLine1,
            &data.stripeShippingAddressZip,
            &data.stripeShippingAddressCity,
            &data.stripeShippingAddressState,
            &data.stripeShippingAddressCountry,
        ])?;

    let subscriber = contacts::create_customer(&data.stripeToken, &me.0)?;
    db.conn()
        .execute("UPDATE people SET customer = $1 WHERE id = $2",
            &[&subscriber.id, &me.0.id])?;

    let ref source = subscriber.sources.data[0];

    db.conn().execute(
        "INSERT INTO cards (id, brand, country, customer, last4, name) VALUES ($1, $2, $3, $4, $5, $6)",
        &[&source.id, &as_brand(&source.brand), &source.country, &source.customer, &source.last4, &source.name]
    )?;

    Ok(Redirect::to("/"))
}


#[derive(Serialize)]
struct HomeData<'a> {
    me: &'a Person,
    contacts: &'a [Contact],
    current_path: &'a str,
    stripe_public_key: &'a str,
}

#[get("/")]
fn home(me: Me, db: DB) -> Result<Template> {
    let stripe_public_key = config::active()
        .ok_or(ConfigError::NotFound)?
        .get_str("stripe_pk")?;

    let contacts = db.conn()
        .query("SELECT * FROM contacts WHERE account = $1", &[&me.0.id])?
        .into_iter()
        .map(Contact::from_row)
        .collect::<Vec<_>>();

    let context = HomeData {
        me: &me.0,
        contacts: &contacts,
        current_path: "/",
        stripe_public_key,
    };

    Ok(Template::render("home", &context))
}


#[get("/", rank = 2)]
fn index() -> Template {
    let nothing: HashMap<(), ()> = HashMap::new();
    Template::render("index", &nothing)
}


#[get("/logout")]
fn logout(cookies: &Cookies) -> Redirect {
    cookies.remove("session");
    Redirect::to("/")
}


fn main() {
    rocket::ignite()
        .mount("/", routes![
            index,
            login,
            finish_login,
            home,
            logout,
            new_contact,
            delete_contact,
            subscribe,
        ])
        .launch();
}
