use std::env::var;
use std::env::VarError::*;

macro_rules! config {
    (
        $($name:ident: $t:ty = $k:ident ($def:expr);)*
    ) => (
        $(pub fn $name() -> $t {
            match var(stringify!($k)) {
                Ok(s) => match s.parse::<$t>() {
                    Ok(v) => v,
                    Err(_) => panic!("could not parse env '{}={}' into {}", stringify!($k), s, stringify!($t)),
                },
                Err(e) => {
                    if var("ENV").map(|v| &v != "production").unwrap_or(true) {
                        return $def
                    }
                    match e {
                        NotPresent => panic!("missing env var '{}'", stringify!($k)),
                        NotUnicode(_) => panic!("env var '{}' is no valid unicode", stringify!($k)),
                    }
                },
            }
        })*

        pub fn check() {
            $($name();)*;
        }
    )
}

config! {
    host: String = HOST ("localhost".into());
    port: u16 = PORT (8000);
    database_url: String = DATABASE_URL ("postgres://postgres@localhost/contacts".into());
    mailgun_url: String = MAILGUN_URL ("https://api.mailgun.net/v3/sandbox47ec148bd79f4a90b8fafa5132289455.mailgun.org".into());
    mailgun_key: String = MAILGUN_KEY ("key-69e59139997765ee2f3a423ccea21349".into());
    stripe_public: String = STRIPE_PK ("pk_test_DJiHbWJpzkotUXG1Cejx1m4J".into());
    stripe_secret: String = STRIPE_SK ("sk_test_EDuyryEZLFw2V0jjYBxFmBlh".into());
}
