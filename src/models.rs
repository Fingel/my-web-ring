use crate::schema::{pages, sources};
use diesel::prelude::*;
use time::PrimitiveDateTime;

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq, AsChangeset)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Source {
    pub id: i32,
    pub weight: i32,
    pub url: String,
    pub last_modified: Option<PrimitiveDateTime>,
    pub etag: Option<String>,
    pub added: PrimitiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = sources)]
pub struct NewSource {
    pub url: String,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Source))]
pub struct Page {
    pub id: i32,
    pub source_id: i32,
    pub url: String,
    pub read: Option<PrimitiveDateTime>,
    pub date: PrimitiveDateTime,
    pub added: PrimitiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = pages)]
pub struct NewPage {
    pub source_id: i32,
    pub url: String,
    pub read: Option<PrimitiveDateTime>,
    pub date: Option<PrimitiveDateTime>,
}
