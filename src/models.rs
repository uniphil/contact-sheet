use uuid::Uuid;
use chrono::naive::datetime::NaiveDateTime;

use super::schema::people;


#[derive(Queryable, Debug)]
pub struct Hi {
    pub id: i32,
}


#[derive(Queryable, Clone, Debug)]
pub struct Person {
    id: Uuid,
    created: NaiveDateTime,
    email: String,
    activated: bool,
    disabled: bool,
}

#[derive(Insertable)]
#[table_name="people"]
pub struct NewPerson<'a> {
    pub email: &'a str,
}
