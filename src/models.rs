use crate::schema::sources;
use diesel::prelude::*;
use time::PrimitiveDateTime;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::sources)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Source {
    pub id: i32,
    pub weight: i32,
    pub url: String,
    pub added: PrimitiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = sources)]
pub struct NewSource<'a> {
    pub url: &'a str,
}
