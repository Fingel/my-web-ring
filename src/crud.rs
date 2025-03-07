use crate::models::{NewSource, Source};
use diesel::prelude::*;
use std::env;

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

pub fn get_sources(conn: &mut SqliteConnection) -> Vec<Source> {
    use crate::schema::sources::dsl::*;

    sources
        .order(added.desc())
        .select(Source::as_select())
        .load(conn)
        .expect("Error loading sources")
}
