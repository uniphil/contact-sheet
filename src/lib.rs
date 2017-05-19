#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate serde_derive;
extern crate chrono;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate reqwest;
extern crate serde;
extern crate uuid;

pub mod schema;
pub mod models;

use dotenv::dotenv;
use diesel::pg::PgConnection;
use r2d2::{Pool, Config};
use r2d2_diesel::ConnectionManager;
use reqwest::Client;
use std::env;
use uuid::Uuid;
// use std::io::Read;


pub fn create_db_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let config = Config::default();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
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
        .basic_auth("api".to_string(), Some(mg_key))
        .form(&params)
        .send()
        .unwrap();
    println!("{:?}", res);
    assert!(res.status().is_success());
}
