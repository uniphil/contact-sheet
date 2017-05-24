use chrono::naive::datetime::NaiveDateTime;
use postgres::rows::Row;
use uuid::Uuid;


#[derive(Debug, FromSql)]
#[postgres(name="address")]
pub struct Address {
    pub name: String,
    pub line1: String,
    pub postal_code: String,
    pub city: String,
    pub province: String,
    pub country: String,
}

#[derive(Debug)]
pub struct Person {
    pub id: Uuid,
    pub created: NaiveDateTime,
    pub email: String,
    pub address: Option<Address>,
    pub customer: Option<String>,
}

impl Person {
    pub fn from_row(row: Row) -> Person {
        Person {
            id: row.get("id"),
            created: row.get("created"),
            email: row.get("email"),
            address: row.get("address"),
            customer: row.get("customer"),
        }
    }
}


#[derive(Clone, Debug)]
pub struct Session {
    pub login_key: Uuid,
    pub created: NaiveDateTime,
    pub account: Uuid,
    pub session_id: Option<Uuid>,
    pub accessed: Option<NaiveDateTime>,
}


impl Session {
    pub fn from_row(row: Row) -> Session {
        Session {
            login_key: row.get("login_key"),
            created: row.get("created"),
            account: row.get("account"),
            session_id: row.get("session_id"),
            accessed: row.get("accessed"),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Contact {
    pub id: Uuid,
    pub created: NaiveDateTime,
    pub account: Uuid,
    pub name: String,
    pub info: String,
}


impl Contact {
    pub fn from_row(row: Row) -> Contact {
        Contact {
            id: row.get("id"),
            created: row.get("created"),
            account: row.get("account"),
            name: row.get("name"),
            info: row.get("info"),
        }
    }
}
