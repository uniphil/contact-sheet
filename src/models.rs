use chrono::naive::datetime::NaiveDateTime;
use postgres::rows::Row;
use uuid::Uuid;


#[derive(Debug, Serialize, FromSql)]
#[postgres(name="address")]
pub struct Address {
    pub name: String,
    pub line1: String,
    pub postal_code: String,
    pub city: String,
    pub province: String,
    pub country: String,
}

#[derive(Debug, Serialize)]
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


#[derive(Debug, Deserialize, ToSql)]
#[postgres(name="ccbrand")]
pub enum CCBrand {
    Visa,
    #[postgres(name="American Express")]
    AmericanExpress,
    MasterCard,
    Discover,
    JCB,
    #[postgres(name="Diners Club")]
    DinersClub,
    Unknown
}

pub fn as_brand(brand: &str) -> CCBrand {
    use self::CCBrand::*;
    match brand {
        "Visa" => Visa,
        "American Express" => AmericanExpress,
        "MasterCard" => MasterCard,
        "Discover" => Discover,
        "JCB" => JCB,
        "Diners Club" => DinersClub,
        "Unknown" => Unknown,
        _ => panic!("unrecognized brand"),
    }
}

#[derive(Debug, Deserialize)]
pub struct StripeSource {
    pub id: String,
    pub brand: String,
    pub country: String,
    pub customer: String,
    pub last4: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct StripeSourcesEnvelope {
    pub data: Vec<StripeSource>,
}

#[derive(Debug, Deserialize)]
pub struct StripeSubscribedCustomer {
    pub id: String,
    pub default_source: String,
    pub sources: StripeSourcesEnvelope,
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
