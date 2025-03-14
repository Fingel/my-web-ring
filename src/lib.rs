pub mod crud;
pub mod models;
pub mod schema;
use crud::{
    create_pages, get_page_by_id, get_sources, mark_source_synced, pages_with_source_weight,
};
use diesel::SqliteConnection;
use models::{NewPage, Page, Source};
use rand::random_range;
use rss::Channel;
use time::{
    PrimitiveDateTime, UtcOffset, format_description::well_known::Rfc2822,
    macros::format_description,
};

pub struct RssResponse {
    body: String,
    status: u16,
    last_modified: Option<PrimitiveDateTime>,
    etag: Option<String>,
}

pub fn download_source(source: &Source) -> Result<RssResponse, ureq::Error> {
    let mut req = ureq::get(&source.url);
    if let Some(last_modified) = &source.last_modified {
        req = req.header(
            "If-Modified-Since",
            last_modified.assume_utc().format(&Rfc2822).unwrap(),
        );
    }
    if let Some(etag) = &source.etag {
        req = req.header("If-None-Match", etag);
    }
    let mut response = req.call()?;
    let body = response.body_mut().read_to_string()?;
    let status = response.status().into();

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

    Ok(RssResponse {
        body,
        status,
        last_modified,
        etag,
    })
}

pub fn parse_rss(xml: &str, source_id: i32) -> Result<Vec<NewPage>, rss::Error> {
    let channel = Channel::read_from(xml.as_bytes())?;
    let new_pages: Vec<NewPage> = channel
        .items()
        .iter()
        .map(|item| NewPage {
            url: item.link().unwrap_or("").to_string(),
            read: None,
            date: item
                .pub_date()
                .and_then(|date| PrimitiveDateTime::parse(date, &Rfc2822).ok()),
            source_id,
        })
        .collect::<Vec<NewPage>>();

    Ok(new_pages)
}

pub fn sync_sources(conn: &mut SqliteConnection) -> usize {
    let sources = get_sources(conn);
    let mut count = 0;
    for source in sources {
        let Ok(rss_resp) = download_source(&source) else {
            println!("Failed to download source {}", source.id);
            continue;
        };
        if rss_resp.status != 200 {
            continue;
        }
        let new_pages = parse_rss(&rss_resp.body, source.id);
        if let Ok(new_pages) = new_pages {
            count += create_pages(conn, new_pages);
        } else {
            println!("Failed to parse RSS for source {}", source.id);
            continue;
        }
        mark_source_synced(conn, source, rss_resp.last_modified, rss_resp.etag);
    }
    count
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
    let pick = random_range(0..sum);
    for (page_id, weight) in cum_sum {
        if pick < weight {
            return get_page_by_id(conn, page_id);
        }
    }
    None
}
