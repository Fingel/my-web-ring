use crate::schema::{pages, sources};
use chrono::NaiveDateTime;
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    prelude::*,
    serialize::{self, Output, ToSql},
    sql_types::Integer,
};

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Integer)]
pub enum SourceType {
    Rss = 1,
    Website = 2,
}

impl<DB> FromSql<Integer, DB> for SourceType
where
    DB: Backend,
    i32: FromSql<Integer, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            1 => Ok(SourceType::Rss),
            2 => Ok(SourceType::Website),
            x => Err(format!("Invalid source type {}", x).into()),
        }
    }
}

impl<DB> ToSql<Integer, DB> for SourceType
where
    DB: Backend,
    i32: ToSql<Integer, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
        match self {
            SourceType::Rss => 1.to_sql(out),
            SourceType::Website => 2.to_sql(out),
        }
    }
}

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq, AsChangeset)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Source {
    pub id: i32,
    pub s_type: SourceType,
    pub weight: i32,
    pub url: String,
    pub last_modified: Option<NaiveDateTime>,
    pub etag: Option<String>,
    pub added: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = sources)]
pub struct NewSource {
    pub url: String,
    pub s_type: SourceType,
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(belongs_to(Source))]
pub struct Page {
    pub id: i32,
    pub source_id: i32,
    pub url: String,
    pub title: String,
    pub read: Option<NaiveDateTime>,
    pub date: NaiveDateTime,
    pub added: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = pages)]
pub struct NewPage {
    pub source_id: i32,
    pub url: String,
    pub title: String,
    pub read: Option<NaiveDateTime>,
    pub date: Option<NaiveDateTime>,
}
