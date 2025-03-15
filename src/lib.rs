pub mod crud;
pub mod models;
pub mod schema;
use std::fmt;

use crud::{
    create_or_reset_page, create_pages, create_source, get_page_by_id, get_sources,
    mark_source_synced, pages_with_source_weight,
};
use diesel::SqliteConnection;
use models::{NewPage, Page, Source, SourceType};
use rand::random_range;
use rss::Channel;
use time::{
    PrimitiveDateTime, UtcOffset, format_description::well_known::Rfc2822,
    macros::format_description,
};

#[derive(Debug)]
pub struct NetworkError {
    message: String,
}

impl From<ureq::Error> for NetworkError {
    fn from(error: ureq::Error) -> Self {
        NetworkError {
            message: error.to_string(),
        }
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Network error: {}", self.message)
    }
}

struct HttpResponse {
    body: String,
    last_modified: Option<PrimitiveDateTime>,
    etag: Option<String>,
}

fn download_source(
    url: &str,
    last_modified: &Option<PrimitiveDateTime>,
    etag: &Option<String>,
) -> Result<HttpResponse, ureq::Error> {
    let mut req = ureq::get(url);
    if let Some(last_modified) = last_modified {
        req = req.header(
            "If-Modified-Since",
            last_modified.assume_utc().format(&Rfc2822).unwrap(),
        );
    }
    if let Some(etag) = etag {
        req = req.header("If-None-Match", etag);
    }
    let mut response = req.call()?;
    let body = response.body_mut().read_to_string()?;

    let last_modified = response
        .headers()
        .get("Last-Modified")
        .and_then(|header| header.to_str().ok()) // as string or none
        .and_then(|date| PrimitiveDateTime::parse(date, &Rfc2822).ok()); // as datetime or none

    let etag = response
        .headers()
        .get("ETag")
        .and_then(|header| header.to_str().ok())
        .map(|etag| etag.to_string());

    Ok(HttpResponse {
        body,
        last_modified,
        etag,
    })
}

struct RssItem {
    link: String,
    date: Option<PrimitiveDateTime>,
}

fn parse_rss(body: &str) -> Result<Vec<RssItem>, rss::Error> {
    let channel = Channel::read_from(body.as_bytes())?;
    Ok(channel
        .items()
        .iter()
        .map(|item| RssItem {
            link: item.link().unwrap_or("").to_string(),
            date: item
                .pub_date()
                .and_then(|date| PrimitiveDateTime::parse(date, &Rfc2822).ok()),
        })
        .collect())
}

fn rss_to_newpages(rss_items: Vec<RssItem>, source_id: i32) -> Vec<NewPage> {
    rss_items
        .into_iter()
        .map(|item| NewPage {
            url: item.link,
            read: None,
            date: item.date,
            source_id,
        })
        .collect()
}
pub fn add_source(conn: &mut SqliteConnection, url: &str) -> Result<Source, NetworkError> {
    let resp = download_source(url, &None::<PrimitiveDateTime>, &None::<String>)?;
    if let Ok(rss_items) = parse_rss(&resp.body) {
        let source = create_source(conn, url, SourceType::Rss);
        let new_pages = rss_to_newpages(rss_items, source.id);
        create_pages(conn, new_pages);
        mark_source_synced(conn, &source, resp.last_modified, resp.etag);
        Ok(source)
    } else {
        let source = create_source(conn, url, SourceType::Website);
        create_or_reset_page(
            conn,
            NewPage {
                url: source.url.clone(),
                read: None,
                date: None,
                source_id: source.id,
            },
        );
        Ok(source)
    }
}
fn sync_source(conn: &mut SqliteConnection, source: &Source) -> usize {
    let mut count = 0;
    match source.s_type {
        SourceType::Rss => {
            let resp = match download_source(&source.url, &source.last_modified, &source.etag) {
                Ok(resp) => resp,
                Err(err) => {
                    println!("Failed to download source {}: {}", source.id, err);
                    return 0;
                }
            };
            if let Ok(rss_items) = parse_rss(&resp.body) {
                let new_pages = rss_to_newpages(rss_items, source.id);
                count += create_pages(conn, new_pages);
                mark_source_synced(conn, source, resp.last_modified, resp.etag);
            }
        }
        SourceType::Website => {
            create_or_reset_page(
                conn,
                NewPage {
                    url: source.url.clone(),
                    read: None,
                    date: None,
                    source_id: source.id,
                },
            );
        }
    }
    count
}
pub fn sync_sources(conn: &mut SqliteConnection) -> usize {
    let sources = get_sources(conn);
    sources
        .into_iter()
        .map(|source| sync_source(conn, &source))
        .sum()
}

pub fn print_source_list(sources: &Vec<Source>) {
    let local_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    println!("{:<5}{:<15}{:<10}URL", "ID", "Last Modified", "Weight");
    for s in sources {
        let format = format_description!("[month]/[day] [hour]:[minute]");
        let formatted = s
            .last_modified
            .map(|date| {
                date.assume_utc()
                    .to_offset(local_offset)
                    .format(format)
                    .unwrap_or("Unknown".to_string())
            })
            .unwrap_or_else(|| "Never".to_string());
        println!("{:<5}{:<15}{:<10}{}", s.id, formatted, s.weight, s.url);
    }
    println!("{} sources.", sources.len());
}

/// Use cumulative sum method to select a page on weighted probability.
pub fn select_page(conn: &mut SqliteConnection) -> Option<Page> {
    let weighted_pages = pages_with_source_weight(conn);
    let mut sum = 0;
    let cum_sum: Vec<(i32, i32)> = weighted_pages
        .iter()
        .map(|(page_id, weight)| {
            sum += weight;
            (*page_id, sum)
        })
        .collect();
    let pick = random_range(0..sum + 1);
    for (page_id, weight) in cum_sum {
        if pick <= weight {
            return get_page_by_id(conn, page_id);
        }
    }
    None
}
