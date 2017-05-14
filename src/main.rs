#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate lazy_static;
extern crate diesel;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
extern crate uuid;
extern crate contacts;

use std::collections::HashMap;
use diesel::pg::PgConnection;
use r2d2::{Pool, PooledConnection, GetTimeout};
use r2d2_diesel::ConnectionManager;
use rocket::Outcome::{Success, Failure};
use rocket::Request;
use rocket::http::{Cookie, Cookies, Status};
use rocket::request::{Form, FromRequest, Outcome};
use rocket_contrib::Template;
use uuid::Uuid;


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


#[post("/login", data = "<form>")]
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
            (me.clone(), true)
        } else {
            let new_me = NewPerson {
                email: email_,
            };
            let me: Person = diesel::insert(&new_me).into(people::table)
                .get_result(db.conn())
                .expect("error saving me");
            (me, false)
        }
    };

    // contacts::send_welcome(email_);

    if let Some(ref cookie) = cookies.find("email") {
        println!("welcome back, {}", cookie.value());
    } else {
        println!("welcome noob");
        let cookie = Cookie::build("email", email.clone())
            // .domain(blah)
            .path("/")
            // .secure(true)
            .http_only(true)
            .finish();
        cookies.add(cookie);
    }

    let mut context = HashMap::new();
    context.insert("email", email);

    Template::render("login", &context)
}


#[get("/")]
fn index() -> Template {
    let nothing: HashMap<(), ()> = HashMap::new();
    Template::render("index", &nothing)
}


fn main() {
    rocket::ignite().mount("/", routes![index, login]).launch();
}
