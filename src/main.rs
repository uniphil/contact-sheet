#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate dotenv;
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
use std::env;
use dotenv::dotenv;
use r2d2::{Pool, PooledConnection, GetTimeout};
use r2d2_postgres::PostgresConnectionManager;
use rocket::Outcome::{Success, Failure, Forward};
use rocket::Request;
use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{Form, FromRequest, Outcome};
use rocket::response::Redirect;
use rocket_contrib::{Template, UUID};
use uuid::Uuid;

use contacts::models::{Person, Session, Contact, as_brand};


lazy_static! {
    pub static ref DB_POOL: Pool<PostgresConnectionManager> = contacts::create_db_pool();
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
fn login(form: Form<Email>, cookies: &Cookies, db: DB) -> Template {
    let &Email { ref email } = form.get();

    // if we start an auth flow, kill whatever session may exist
    cookies.remove("session");

    let conn = db.conn();

    let (me, new) = {
        let res: Option<Person> = conn
            .query("SELECT * FROM people WHERE people.email = $1", &[&email])
            .expect("oops")
            .into_iter()
            .map(Person::from_row)
            .next();

        if let Some(me) = res {
            (me, false)
        } else {
            let new_me: Person = conn
                .query("INSERT INTO people (email) VALUES ($1) RETURNING *", &[&email])
                .expect("oopsie")
                .into_iter()
                .map(Person::from_row)
                .next()
                .expect("shoulda");
            (new_me, true)
        }
    };

    let login_key: Uuid = conn
        .query("INSERT INTO sessions (account) VALUES ($1) RETURNING login_key", &[&me.id])
        .expect("bleh")
        .into_iter()
        .map(|row| row.get(0))
        .next()
        .expect("whatev");

    contacts::send_login(email, &login_key, new);

    let mut context = HashMap::new();
    context.insert("email", email);

    Template::render("login", &context)
}


#[derive(Debug, FromForm)]
struct LoginKey {
    key: UUID,
}


#[get("/login?<form>")]
fn finish_login(form: LoginKey, cookies: &Cookies, db: DB) -> Result<Redirect, String> {
    let LoginKey { ref key } = form;

    // if we are in auth flow, kill whatever session may exist
    cookies.remove("session");

    let existing = db.conn()
        .query("SELECT * FROM sessions WHERE login_key = $1", &[&key.into_inner()])
        .expect("yea yeah")
        .into_iter()
        .map(Session::from_row)
        .next();

    if let Some(session) = existing {
        if let Some(_) = session.session_id {
            return Err(format!("already got this session whoops"))
        } else {
            let id: Uuid = db.conn()
                .query("UPDATE sessions SET session_id = uuid_generate_v4() WHERE login_key = $1 RETURNING session_id", &[&key.into_inner()])
                .expect("kyo")
                .into_iter()
                .map(|row| row.get(0))
                .next()
                .expect("mhmm");

            let cookie = Cookie::build("session", id.to_string())
                // .domain(blah)
                .path("/")
                // .secure(true)
                .http_only(true)
                .finish();
            cookies.add(cookie);
            return Ok(Redirect::to("/"))
        }
    }

    Err("asdf".to_string())
}


#[derive(Debug)]
struct Me(Person);

impl<'a, 'r> FromRequest<'a, 'r> for Me {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Me, ()> {
        let claimed_id: Option<Uuid> = request
            .cookies()
            .find("session")
            .and_then(|cookie| cookie.value().parse().ok());

        if let Some(claimed_uuid) = claimed_id {
            let db = DB(DB_POOL.get().expect("couldn't get db"));
            let res = db.conn()
                .query("SELECT p.* FROM people p, sessions s WHERE s.account = p.id AND s.session_id = $1", &[&claimed_uuid])
                .expect("k")
                .into_iter()
                .map(Person::from_row)
                .next();
            if let Some(me) = res {
                return Success(Me(me))
            }
        }

        Forward(())
    }
}


#[derive(Debug, FromForm)]
struct NewContactForm {
    name: String,
    info: String,
}


#[post("/contacts", data="<form>")]
fn new_contact(form: Form<NewContactForm>, me: Me, db: DB) ->
Result<Redirect, String> {
    let &NewContactForm { ref name, ref info } = form.get();

    db.conn()
        .execute("INSERT INTO contacts (account, name, info) VALUES ($1, $2, $3)", &[&me.0.id, &name, &info])
        .and_then(|_| Ok(Redirect::to("/")))
        .or_else(|_| Err("could not save contact".into()))
}


#[derive(Debug, FromForm)]
struct DeleteContactForm {
    id: UUID,
    next: Option<String>,
}


#[get("/contacts/delete?<form>")]
fn delete_contact(form: DeleteContactForm, me: Me, db: DB) ->
Result<Redirect, String> {
    let DeleteContactForm { id, next } = form;

    db.conn()
        .execute("DELETE FROM contacts WHERE id = $1 AND account = $2", &[&id.into_inner(), &me.0.id])
        .or_else(|_| Err("could not delete contact".into()))
        .and_then(|num| match num {
            0 => Err("contact not found".into()),
            1 => Ok(Redirect::to(&next.unwrap_or("/".into()))),
            _ => Err("whaaaaa".into()),
        })
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
fn subscribe(form: Form<StripeSubscribe>, me: Me, db: DB) ->
Result<Redirect, String> {
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
        ])
        .expect("couldn't set address");
    let subscriber = contacts::create_customer(&data.stripeToken, &me.0);
    db.conn()
        .execute("UPDATE people SET customer = $1 WHERE id = $2",
            &[&subscriber.id, &me.0.id])
        .expect("couldn't set subscriber");
    let ref source = subscriber.sources.data[0];
    db.conn()
        .execute("INSERT INTO cards (id, brand, country, customer, last4, name) VALUES ($1, $2, $3, $4, $5, $6)",
            &[&source.id, &as_brand(&source.brand), &source.country, &source.customer, &source.last4, &source.name])
        .expect("couldn't save card");
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
fn home(me: Me, db: DB) -> Template {
    dotenv().ok();
    let stripe_public_key = &env::var("STRIPE_PUBLIC").expect("STRIPE_PUBLIC must be set");

    let contacts = db.conn()
        .query("SELECT * FROM contacts WHERE account = $1", &[&me.0.id])
        .expect("hi")
        .into_iter()
        .map(Contact::from_row)
        .collect::<Vec<_>>();

    let context = HomeData {
        me: &me.0,
        contacts: &contacts,
        current_path: "/",
        stripe_public_key,
    };

    Template::render("home", &context)
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
