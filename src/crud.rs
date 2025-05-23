use crate::models::{NewPage, NewSource, Page, Source, SourceType};
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::{
    dsl::now,
    prelude::*,
    result::{
        DatabaseErrorKind,
        Error::{DatabaseError, NotFound},
    },
};
use std::cmp;

pub fn create_source(
    conn: &mut SqliteConnection,
    url: &str,
    s_type: SourceType,
    title: String,
) -> Source {
    use crate::schema::sources;

    let new_post = NewSource {
        url: url.to_string(),
        s_type,
        title,
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
    marked_source: &Source,
    i_last_modified: Option<NaiveDateTime>,
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

/// Creates a page but if the url exists, set it as unread and the date
/// as 5 days ago so they don't get buried.
pub fn create_or_reset_page(conn: &mut SqliteConnection, new_page: NewPage) -> usize {
    use crate::schema::pages::dsl::*;

    diesel::insert_into(pages)
        .values(&new_page)
        .on_conflict(url)
        .do_update()
        .set((
            read.eq(Option::<NaiveDateTime>::None),
            date.eq((Utc::now() - Duration::days(5)).naive_utc()),
        ))
        .execute(conn)
        .expect("Unexpected database error create_single_page")
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

pub fn get_unread_pages_by_source(conn: &mut SqliteConnection, i_source_id: i32) -> Vec<Page> {
    use crate::schema::pages::dsl::*;

    pages
        .filter(source_id.eq(i_source_id))
        .filter(read.is_null())
        .select(Page::as_select())
        .order(added.desc())
        .get_results(conn)
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

pub fn mark_source_read(conn: &mut SqliteConnection, i_source_id: i32) -> Vec<Page> {
    use crate::schema::pages::dsl::*;
    diesel::update(pages)
        .filter(source_id.eq(i_source_id))
        .set(read.eq(now))
        .returning(Page::as_returning())
        .get_results(conn)
        .expect("Error setting pages read.")
}

pub fn mark_page_unread(conn: &mut SqliteConnection, page: &Page) -> Page {
    use crate::schema::pages::dsl::*;
    diesel::update(page)
        .set(read.eq(None::<NaiveDateTime>))
        .returning(Page::as_returning())
        .get_result(conn)
        .expect("Error setting page read.")
}

pub fn read_status_for_source(
    conn: &mut SqliteConnection,
    i_source_id: i32,
) -> Vec<Option<NaiveDateTime>> {
    use crate::schema::pages::dsl::{pages, read, source_id};
    use crate::schema::sources::dsl::sources;

    pages
        .inner_join(sources)
        .filter(source_id.eq(i_source_id))
        .select(read)
        .get_results(conn)
        .expect("Error loading pages for source")
}

pub fn pages_with_source_weight(conn: &mut SqliteConnection) -> Vec<(i32, i32)> {
    use crate::schema::pages::dsl::{date, id, pages, read};
    use crate::schema::sources::dsl::{sources, weight};

    pages
        .inner_join(sources)
        .filter(read.is_null())
        .select((id, weight))
        .order(date.asc())
        .get_results(conn)
        .expect("Error loading pages with source weight")
}

pub fn set_source_weight(
    conn: &mut SqliteConnection,
    source_id: i32,
    i_weight: i32,
) -> (i32, String) {
    use crate::schema::sources::dsl::{id, sources, url, weight};

    let source = get_source_by_id(conn, source_id).expect("Source not found");
    let new_weight = cmp::max(0, source.weight + i_weight);
    diesel::update(sources.filter(id.eq(source_id)))
        .set(weight.eq(new_weight))
        .returning((weight, url))
        .get_result(conn)
        .expect("Error setting source weight.")
}
