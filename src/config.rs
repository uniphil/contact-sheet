use std::env::var;
use std::env::VarError::*;

fn get(k: &str, def: &str) -> String {
    match var(k) {
        Ok(v) => v.into(),
        Err(e) => {
            if var("ENV").map(|v| &v != "production").unwrap_or(true) {
                return def.into()
            }
            match e {
                NotPresent => panic!("Missing variable '{}' from environment", k),
                NotUnicode(_) => panic!("Invalid variable '{}' in environment", k),
            }
        }
    }
}

lazy_static! {
    pub static ref HOST: String =
        get("HOST", "localhost");

    pub static ref PORT: usize =
        get("PORT", "8000").parse().unwrap();

    pub static ref DATABASE_URL: String =
        get("DATABASE_URL", "postgres://postgres@localhost/contacts");

    pub static ref MAILGUN_URL: String =
        get("MAILGUN_URL", "https://api.mailgun.net/v3/sandbox47ec148bd79f4a90b8fafa5132289455.mailgun.org");
    pub static ref MAILGUN_KEY: String =
        get("MAILGUN_KEY", "key-69e59139997765ee2f3a423ccea21349");

    pub static ref STRIPE_PK: String =
        get("STRIPE_PK", "pk_test_DJiHbWJpzkotUXG1Cejx1m4J");
    pub static ref STRIPE_SK: String =
        get("STRIPE_SK", "sk_test_EDuyryEZLFw2V0jjYBxFmBlh");
}
