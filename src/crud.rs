use crate::models::{NewPage, NewSource, Page, Source};
use diesel::prelude::*;
use std::env;
use time::PrimitiveDateTime;

pub fn establish_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Failed to establish connection to {}", database_url))
}

pub fn create_source(conn: &mut SqliteConnection, url: &str) -> Source {
    use crate::schema::sources;

    let new_post = NewSource { url };

    diesel::insert_into(sources::table)
        .values(&new_post)
        .returning(Source::as_returning())
        .get_result(conn)
        .expect("Error saving source")
}

pub fn get_source(conn: &mut SqliteConnection, source_id: i32) -> Source {
    use crate::schema::sources::dsl::*;

    sources
        .filter(id.eq(source_id))
        .select(Source::as_select())
        .first(conn)
        .expect("Error loading source")
}

pub fn get_sources(conn: &mut SqliteConnection) -> Vec<Source> {
    use crate::schema::sources::dsl::*;

    sources
        .order(added.desc())
        .select(Source::as_select())
        .load(conn)
        .expect("Error loading sources")
}

pub fn create_page(
    conn: &mut SqliteConnection,
    source_id: i32,
    url: &str,
    read: Option<PrimitiveDateTime>,
    date: Option<PrimitiveDateTime>,
) -> Page {
    use crate::schema::pages;

    let new_page = NewPage {
        source_id,
        url,
        read,
        date,
    };

    diesel::insert_into(pages::table)
        .values(&new_page)
        .returning(Page::as_returning())
        .get_result(conn)
        .expect("Error saving page")
}

pub fn create_pages(conn: &mut SqliteConnection, new_pages: Vec<NewPage>) -> usize {
    use crate::schema::pages;

    diesel::insert_into(pages::table)
        .values(&new_pages)
        .execute(conn)
        .expect("Error saving pages")
}
