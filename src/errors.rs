error_chain! {
    foreign_links {
        Db(::postgres::error::Error);
    }
}
