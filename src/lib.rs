#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate postgres_derive;
extern crate chrono;
extern crate dotenv;
extern crate postgres;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate reqwest;
extern crate rocket;
extern crate serde;
extern crate uuid;

pub mod models;

use dotenv::dotenv;
use r2d2::{Pool, Config};
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use reqwest::Client;
use reqwest::header::{Authorization, Bearer};
use std::env;
use uuid::Uuid;
use std::io::Read;


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
        .basic_auth("api".to_owned(), Some(mg_key))
        .form(&params)
        .send()
        .unwrap();
    println!("{:?}", res);
    assert!(res.status().is_success());
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


pub fn create_customer(subscribe: &StripeSubscribe, me: &models::Person) -> () {
    dotenv().ok();
    let stripe_sk = env::var("STRIPE_SECRET").expect("fdsa");
    let client = Client::new().unwrap();
    let params = [
        ("plan", "testing"),
        ("source", &subscribe.stripeToken),
        ("email", &me.email),
    ];
    let mut res = client.post("https://api.stripe.com/v1/customers")
        .header(Authorization(Bearer { token: stripe_sk }))
        .form(&params)
        .send()
        .unwrap();
    println!("{:?}", res);
    let mut content = String::new();
    res.read_to_string(&mut content).unwrap();
    println!("{}", content);
    if !res.status().is_success() {
        panic!("not successful");
    }
}
