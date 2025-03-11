use crate::models::{NewPage, NewSource, Source};
use diesel::{
    prelude::*,
    result::{
        DatabaseErrorKind,
        Error::{DatabaseError, NotFound},
    },
};
use std::env;

pub fn establish_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Failed to establish connection to {}", database_url))
}

pub fn create_source(conn: &mut SqliteConnection, url: &str) -> Source {
    use crate::schema::sources;

    let new_post = NewSource { url };

    match diesel::insert_into(sources::table)
        .values(&new_post)
        .returning(Source::as_returning())
        .get_result(conn)
    {
        Ok(source) => source,
        Err(err) => match err {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => get_source_by_url(conn, url)
                .expect("Unique violation, but could not find matching source"),
            _ => panic!("Database error: {}", err),
        },
    }
}

fn get_source_by_url(conn: &mut SqliteConnection, source_url: &str) -> Option<Source> {
    use crate::schema::sources::dsl::*;

    match sources
        .filter(url.eq(source_url))
        .select(Source::as_select())
        .first(conn)
    {
        Ok(source) => Some(source),
        Err(err) => match err {
            NotFound => None,
            _ => panic!("Database error: {}", err),
        },
    }
}

pub fn get_source_by_id(conn: &mut SqliteConnection, source_id: i32) -> Option<Source> {
    use crate::schema::sources::dsl::*;

    match sources
        .filter(id.eq(source_id))
        .select(Source::as_select())
        .first(conn)
    {
        Ok(source) => Some(source),
        Err(err) => match err {
            NotFound => None,
            _ => panic!("Database error: {}", err),
        },
    }
}

pub fn get_sources(conn: &mut SqliteConnection) -> Vec<Source> {
    use crate::schema::sources::dsl::*;

    sources
        .order(added.desc())
        .select(Source::as_select())
        .load(conn)
        .expect("Error loading sources")
}

pub fn create_pages(conn: &mut SqliteConnection, new_pages: Vec<NewPage>) -> usize {
    use crate::schema::pages;

    match diesel::insert_into(pages::table)
        .values(&new_pages)
        .execute(conn)
    {
        Ok(count) => count,
        Err(err) => match err {
            DatabaseError(DatabaseErrorKind::UniqueViolation, _) => 0,
            _ => panic!("Database error: {}", err),
        },
    }
}
