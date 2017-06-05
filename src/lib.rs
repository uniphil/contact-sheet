#![recursion_limit = "1024"]
#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate error_chain;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate postgres;
#[macro_use] extern crate postgres_derive;
extern crate chrono;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate reqwest;
extern crate rocket;
extern crate serde;
extern crate serde_json;
extern crate uuid;

pub mod config;
pub mod errors;
pub mod models;
pub mod query;

use errors::*;
use r2d2::{Pool, Config};
use r2d2_postgres::{TlsMode, PostgresConnectionManager};
use reqwest::Client;
use reqwest::header::{Authorization, Basic, Bearer};
use uuid::Uuid;


pub fn create_db_pool() -> Result<Pool<PostgresConnectionManager>> {
    let database_url: &str = &config::DATABASE_URL;
    let config = Config::default();
    let manager = PostgresConnectionManager::new(database_url, TlsMode::None)?;
    Ok(Pool::new(config, manager)?)
}


pub fn send_login(to: &str, login_key: &Uuid, new: bool) -> Result<()> {
    let host: &str = "fixme";

    let mg_url: &str = &config::MAILGUN_URL;
    let mg_key: &str = &config::MAILGUN_KEY;

    let subject = if new { "Get started with Contact Sheet" }
                    else { "Log in to Contact Sheet" };
    let params = [
        ("from", "no-reply@email.contact-sheet.ca"),
        ("to", to),
        ("subject", subject),
        ("text", &format!("Here is your key: {}/login?key={}", host, login_key)),
    ];
    let client = Client::new()?;
    let res = client.post(&format!("{}{}", mg_url, "/messages"))
        .header(Authorization(Basic {
            username: "api".to_owned(),
            password: Some(mg_key.to_owned()),
        }))
        .form(&params)
        .send()
        .chain_err(|| "Could not send login email")?;

    if ! res.status().is_success() {
        bail!("Could not send login email");
    }

    Ok(())
}


pub fn create_customer(token: &str, me: &models::Person) ->
Result<models::StripeSubscribedCustomer> {
    let stripe_sk: &str = &config::STRIPE_SK;

    let client = Client::new()?;
    let params = [
        ("plan", "testing"),
        ("source", &token),
        ("email", &me.email),
    ];
    let mut res = client.post("https://api.stripe.com/v1/customers")
        .header(Authorization(Bearer { token: stripe_sk.into() }))
        .form(&params)
        .send()
        .chain_err(|| "tried to send request. sigh.")?;

    if ! res.status().is_success() {
        bail!("not successful");
    }

    let customer = res
        .json::<models::StripeSubscribedCustomer>()
        .chain_err(|| "tried to deserialize stuff. hhhhhhh.")?;

    Ok(customer)
}
