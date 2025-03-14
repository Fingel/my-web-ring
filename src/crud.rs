use crate::models::{NewPage, NewSource, Page, Source};
use diesel::{
    dsl::now,
    prelude::*,
    result::{
        DatabaseErrorKind,
        Error::{DatabaseError, NotFound},
    },
};
use std::env;
use time::PrimitiveDateTime;

pub fn establish_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Failed to establish connection to {}", database_url))
}

pub fn create_source(conn: &mut SqliteConnection, url: &str) -> Source {
    use crate::schema::sources;

    let new_post = NewSource {
        url: url.to_string(),
    };

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

pub fn get_page_by_id(conn: &mut SqliteConnection, page_id: i32) -> Option<Page> {
    use crate::schema::pages::dsl::*;

    match pages
        .filter(id.eq(page_id))
        .select(Page::as_select())
        .first(conn)
    {
        Ok(page) => Some(page),
        Err(err) => match err {
            NotFound => None,
            _ => panic!("Database error: {}", err),
        },
    }
}

pub fn delete_source(conn: &mut SqliteConnection, source_id: i32) -> Result<String, String> {
    if let Some(source) = get_source_by_id(conn, source_id) {
        diesel::delete(&source)
            .execute(conn)
            .expect("Error deleting source");
        Ok(source.url)
    } else {
        Err("Source not found".to_string())
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

pub fn mark_source_synced(
    conn: &mut SqliteConnection,
    marked_source: Source,
    i_last_modified: Option<PrimitiveDateTime>,
    i_etag: Option<String>,
) -> Source {
    use crate::schema::sources::dsl::*;

    diesel::update(&marked_source)
        .set((last_modified.eq(i_last_modified), etag.eq(i_etag)))
        .returning(Source::as_returning())
        .get_result(conn)
        .expect("Error marking source as synced")
}

pub fn create_pages(conn: &mut SqliteConnection, new_pages: Vec<NewPage>) -> usize {
    use crate::schema::pages;

    let mut count = 0;
    new_pages.iter().for_each(|new_page| {
        count += match diesel::insert_into(pages::table)
            .values(new_page)
            .execute(conn)
        {
            Ok(count) => count,
            Err(err) => match err {
                DatabaseError(DatabaseErrorKind::UniqueViolation, _) => 0,
                _ => panic!("Database error: {}", err),
            },
        }
    });
    count
}

pub fn get_pages(conn: &mut SqliteConnection, unread: bool) -> Vec<Page> {
    use crate::schema::pages::dsl::*;

    let query = pages.order(added.desc()).select(Page::as_select());
    if unread {
        query.filter(read.is_null()).get_results(conn)
    } else {
        query.get_results(conn)
    }
    .expect("Error loading pages")
}

pub fn mark_page_read(conn: &mut SqliteConnection, page: &Page) -> Page {
    use crate::schema::pages::dsl::*;
    diesel::update(page)
        .set(read.eq(now))
        .returning(Page::as_returning())
        .get_result(conn)
        .expect("Error setting page read.")
}

pub fn pages_with_source_weight(conn: &mut SqliteConnection) -> Vec<(i32, i32)> {
    use crate::schema::pages::dsl::{id, pages, read};
    use crate::schema::sources::dsl::{sources, weight};

    pages
        .inner_join(sources)
        .filter(read.is_null())
        .select((id, weight))
        .get_results(conn)
        .expect("Error loading pages with source weight")
}
