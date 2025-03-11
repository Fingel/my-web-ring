pub mod crud;
pub mod models;
pub mod schema;
use crud::{create_pages, get_sources, mark_source_synced};
use diesel::SqliteConnection;
use models::{NewPage, Source};
use rss::Channel;
use time::{
    PrimitiveDateTime, UtcOffset, format_description::well_known::Rfc2822,
    macros::format_description,
};

pub fn download_source(url: &str) -> Result<String, ureq::Error> {
    ureq::get(url).call()?.body_mut().read_to_string()
}

pub fn parse_rss(xml: &str) -> Result<Channel, rss::Error> {
    Channel::read_from(xml.as_bytes())
}

pub fn sync_pages(conn: &mut SqliteConnection, source: &Source) -> usize {
    let channel = parse_rss(&download_source(source.url.as_str()).unwrap()).unwrap();
    let new_pages: Vec<NewPage> = channel
        .items()
        .iter()
        .map(|item| NewPage {
            source_id: source.id,
            url: item.link().unwrap_or(""),
            read: None,
            date: item
                .pub_date()
                .and_then(|date| PrimitiveDateTime::parse(date, &Rfc2822).ok()),
        })
        .collect();
    create_pages(conn, new_pages)
}

pub fn sync_sources(conn: &mut SqliteConnection) -> usize {
    let sources = get_sources(conn);
    let mut count = 0;
    for source in sources {
        count += sync_pages(conn, &source);
        mark_source_synced(conn, source);
    }
    count
}

pub fn print_source_list(sources: &Vec<Source>) {
    let local_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    println!("{:<5}{:<15}{:<10}URL", "ID", "Last Sync", "Weight");
    for s in sources {
        let format = format_description!("[month]/[day] [hour]:[minute]");
        let formatted = match s.last_synced {
            Some(date) => date
                .assume_utc()
                .to_offset(local_offset)
                .format(format)
                .unwrap(),
            None => "Never".to_string(),
        };
        println!("{:<5}{:<15}{:<10}{}", s.id, formatted, s.weight, s.url);
    }
    println!("{} sources.", sources.len());
}
