#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate postgres;
#[macro_use] extern crate postgres_derive;
extern crate chrono;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate reqwest;
extern crate rocket;
extern crate serde;
extern crate serde_json;
extern crate uuid;

pub mod models;

use dotenv::dotenv;
use r2d2::{Pool, Config};
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use reqwest::Client;
use reqwest::header::{Authorization, Basic, Bearer};
use std::env;
use uuid::Uuid;


pub fn create_db_pool() -> Pool<PostgresConnectionManager> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let config = Config::default();
    let manager = PostgresConnectionManager::new(database_url, TlsMode::None)
        .expect("bleh?");
    Pool::new(config, manager)
        .expect("Failed to create pool.")
}


pub fn send_login(to: &str, login_key: &Uuid, new: bool) -> () {
    dotenv().ok();
    let mg_url = env::var("MAILGUN_URL").expect("MAILGUN_URL must be set");
    let mg_key = env::var("MAILGUN_KEY").expect("MAILGUN_KEY must be set");
    let host = env::var("HOST").expect("HOST must be set");
    let subject = if new { "Get started with Contact Sheet" }
                    else { "Log in to Contact Sheet" };
    let params = [
        ("from", "no-reply@email.contact-sheet.ca"),
        ("to", to),
        ("subject", subject),
        ("text", &format!("Here is your key: {}/login?key={}", host, login_key)),
    ];
    let client = Client::new().unwrap();
    let res = client.post(&format!("{}{}", mg_url, "/messages"))
        .header(Authorization(Basic {
            username: "api".to_owned(),
            password: Some(mg_key),
        }))
        .form(&params)
        .send()
        .unwrap();
    assert!(res.status().is_success());
}


pub fn create_customer(token: &str, me: &models::Person) -> models::StripeSubscribedCustomer {
    dotenv().ok();
    let stripe_sk = env::var("STRIPE_SECRET").expect("fdsa");
    let client = Client::new().unwrap();
    let params = [
        ("plan", "testing"),
        ("source", &token),
        ("email", &me.email),
    ];
    let mut res = client.post("https://api.stripe.com/v1/customers")
        .header(Authorization(Bearer { token: stripe_sk }))
        .form(&params)
        .send()
        .expect("tried to send request. sigh.");

    if !res.status().is_success() {
        panic!("not successful");
    }

    res
        .json::<models::StripeSubscribedCustomer>()
        .expect("tried to deserialize stuff. hhhhhhh.")
}
