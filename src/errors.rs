error_chain! {
    foreign_links {
        Config(::rocket::config::ConfigError);
        Db(::postgres::error::Error);
        DbConnect(::postgres::error::ConnectError);
        R2d2Init(::r2d2::InitializationError);
        R2d2Timeout(::r2d2::GetTimeout);
        Reqwest(::reqwest::Error);
    }
}
