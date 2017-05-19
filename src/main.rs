#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate uuid;
extern crate contacts;

use std::collections::HashMap;
use diesel::pg::PgConnection;
use r2d2::{Pool, PooledConnection, GetTimeout};
use r2d2_diesel::ConnectionManager;
use rocket::Outcome::{Success, Failure, Forward};
use rocket::Request;
use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{Form, FromRequest, Outcome};
use rocket::response::Redirect;
use rocket_contrib::{Template, UUID};
use uuid::Uuid;

use contacts::models::{Person, Contact};


lazy_static! {
    pub static ref DB_POOL: Pool<ConnectionManager<PgConnection>> = contacts::create_db_pool();
}


pub struct DB(PooledConnection<ConnectionManager<PgConnection>>);

impl DB {
    pub fn conn(&self) -> &PgConnection {
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
    let email_ = email;

    // if we start an auth flow, kill whatever session may exist
    cookies.remove("session");

    let (me, new) = {
        use diesel::prelude::*;
        use contacts::schema::people;
        use contacts::schema::people::dsl::*;
        use contacts::models::{Person, NewPerson};

        let res = people.filter(email.eq(email))
            .load::<Person>(db.conn())
            .expect("couldn't query people");

        if let Some(me) = res.first() {
            (me.clone(), false)
        } else {
            let new_me = NewPerson {
                email: email_,
            };
            let me: Person = diesel::insert(&new_me).into(people::table)
                .get_result(db.conn())
                .expect("error saving me");
            (me, true)
        }
    };

    {
        use diesel::prelude::*;
        use contacts::schema::sessions;
        use contacts::models::{NewSession, Session};
        let new_session = NewSession {
            account: me.id,
        };
        let session: Session = diesel::insert(&new_session).into(sessions::table)
            .get_result(db.conn())
            .expect("error creating session");

        contacts::send_login(email, &session.login_key, new);
    }

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

    {
        use diesel::prelude::*;
        use contacts::schema::sessions::dsl::*;
        use contacts::models::Session;

        let res = sessions.find(key.into_inner()).load::<Session>(db.conn()).expect("blah");

        if let Some(session) = res.first() {
            if let Some(id) = session.session_id {
                return Err(format!("already got this session {} whoops", id))
            } else {
                let logged_in = session.login();
                logged_in.save_changes::<Session>(db.conn()).expect("failed to save login");
                let id = logged_in.session_id.unwrap().hyphenated().to_string();
                let cookie = Cookie::build("session", id)
                    // .domain(blah)
                    .path("/")
                    // .secure(true)
                    .http_only(true)
                    .finish();
                cookies.add(cookie);
                return Ok(Redirect::to("/"))
            }
        } else {
            println!("oooh {:?}", "asdf");
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
            use diesel::prelude::*;
            use contacts::schema::sessions;
            use contacts::schema::people::dsl::*;
            use contacts::schema::sessions::dsl::*;
            use contacts::models::{Person, Session};
            let db = DB(DB_POOL.get().expect("couldn't get db"));

            let data = people.inner_join(sessions::table)
                .filter(session_id.eq(claimed_uuid))
                .first::<(Person, Session)>(db.conn());

            if let Ok((me, _)) = data {
                return Success(Me(me));
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
    use diesel::prelude::*;
    use contacts::schema::contacts;
    use contacts::models::NewContact;

    let &NewContactForm { ref name, ref info } = form.get();

    let new_contact = NewContact {
        account: me.0.id,
        name,
        info,
    };

    diesel::insert(&new_contact)
        .into(contacts::table)
        .execute(db.conn())
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
    use diesel::prelude::*;
    use contacts::schema::contacts::dsl::*;

    let DeleteContactForm { id: id_, next } = form;

    diesel::delete(contacts.filter(account.eq(me.0.id).and(id.eq(*id_))))
        .execute(db.conn())
        .or_else(|_| Err("Could not delete contact".into()))
        .and_then(|num| if num == 1 {
            Ok(Redirect::to(&next.unwrap_or("/".into())))
        } else {
            Err("no contact found".into())
        })
}


#[derive(Serialize)]
struct HomeData<'a> {
    email: &'a str,
    contacts: &'a [Contact],
    current_path: &'a str,
}

#[get("/")]
fn home(me: Me, db: DB) -> Template {
    use diesel::prelude::*;
    let contacts: Vec<Contact> = Contact::belonging_to(&(me.0))
            .load(db.conn())
            .expect("could not load contacts");
    let context = HomeData {
        email: &me.0.email,
        contacts: &contacts,
        current_path: "/",
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
        .mount("/", routes![index, login, finish_login, home, logout, new_contact, delete_contact])
        .launch();
}
