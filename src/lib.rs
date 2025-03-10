pub mod crud;
pub mod models;
pub mod schema;
use crud::{create_pages, establish_connection};
use models::{NewPage, Source};
use rss::Channel;
use time::{PrimitiveDateTime, format_description::well_known::Rfc2822};

pub fn download_source(url: &str) -> Result<String, ureq::Error> {
    ureq::get(url).call()?.body_mut().read_to_string()
}

pub fn parse_rss(xml: &str) -> Result<Channel, rss::Error> {
    Channel::read_from(xml.as_bytes())
}

pub fn sync_pages(source: Source) -> usize {
    let mut conn = establish_connection();
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
    create_pages(&mut conn, new_pages)
}
