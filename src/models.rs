use uuid::Uuid;
use chrono::naive::datetime::NaiveDateTime;

use super::schema::{people, sessions};


#[derive(Queryable, Clone, Debug)]
pub struct Person {
    pub id: Uuid,
    pub created: NaiveDateTime,
    pub email: String,
    pub activated: bool,
    pub disabled: bool,
}

#[derive(Insertable)]
#[table_name="people"]
pub struct NewPerson<'a> {
    pub email: &'a str,
}


#[derive(Queryable, Associations, Identifiable, AsChangeset, Insertable, Clone, Debug)]
#[table_name="sessions"]
#[belongs_to(Person)]
#[primary_key(login_key)]
pub struct Session {
    pub login_key: Uuid,
    pub created: NaiveDateTime,
    pub account: Uuid,
    pub session_id: Option<Uuid>,
    pub accessed: Option<NaiveDateTime>,
}


impl Session {
    pub fn login(&self) -> Session {
        let mut new = self.clone();
        new.session_id = Some(Uuid::new_v4());
        new
    }
}


#[derive(Insertable)]
#[table_name="sessions"]
pub struct NewSession {
    pub account: Uuid,
    // pub login_ua: &'a str,
}
