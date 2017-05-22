use std::error::Error;
use diesel::types::{HasSqlType, FromSql, BigInt};
use diesel::pg::{Pg, PgTypeMetadata};


pub struct Address(pub i64);

impl HasSqlType<Address> for Pg {
    fn metadata() -> PgTypeMetadata {
        PgTypeMetadata {
            oid: 25762,
            array_oid: 0,
        }
    }
}

impl FromSql<Address, Pg> for Address {
    fn from_sql(bytes: Option<&[u8]>)
                -> Result<Self, Box<Error + Send + Sync>> {
        FromSql::<BigInt, Pg>::from_sql(bytes)
            .map(Address)
    }
}


// infer_schema! doesn't yet support citext, bleh
// https://gitter.im/diesel-rs/diesel/archives/2016/07/24
table! {
    use diesel::types::*;
    use schema::Address;
    people (id) {
        id -> Uuid,
        created -> Timestamp,
        email -> Text,
        activated -> Bool,
        disabled -> Bool,
        address -> Nullable<Address>,
        // customer -> Nullable<Text>,
    }
}

infer_table_from_schema!("dotenv:DATABASE_URL", "sessions");

// copy-pasted from `$ diesel print-schema` because more than one
// infer_table_from_schema breaks :(
table! {
    contacts (id) {
        id -> Uuid,
        created -> Timestamp,
        account -> Uuid,
        name -> Text,
        info -> Text,
    }
}
