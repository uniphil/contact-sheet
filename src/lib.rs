#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate chrono;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate reqwest;
extern crate uuid;

pub mod schema;
pub mod models;

use dotenv::dotenv;
use diesel::pg::PgConnection;
use r2d2::{ Pool, Config };
use r2d2_diesel::ConnectionManager;
use reqwest::Client;
use std::env;
// use std::io::Read;


// enum AccountStatus {
//     New,
//     Existing,
// }


pub fn create_db_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let config = Config::default();
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::new(config, manager)
        .expect("Failed to create pool.")
}


// pub fn create_activation(email: &str) -> {

// }


pub fn send_welcome(to: &str) -> () {
    dotenv().ok();
    let mg_url = env::var("MAILGUN_URL").expect("MAILGUN_URL must be set");
    let mg_key = env::var("MAILGUN_KEY").expect("MAILGUN_KEY must be set");
    let params = [
        ("from", "uniphil@gmail.com"),
        ("to", to),
        ("subject", "eyo"),
        ("text", "hey sup"),
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
