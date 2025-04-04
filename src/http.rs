use crate::crud::mark_page_read;
use crate::{select_page, sync_sources};
use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use log::{error, info};
use std::io::{BufRead, BufReader, prelude::*};
use std::net::{TcpListener, TcpStream};

pub fn server(pool: &Pool<ConnectionManager<SqliteConnection>>) {
    let conn = &mut pool.get().expect("Failed to get connection");
    let listener = TcpListener::bind("0.0.0.0:8090").unwrap();
    println!("Listening on http://0.0.0.0:8090");

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to accept connection. Https?: {}", e);
                continue;
            }
        };
        let page = match select_page(conn) {
            Some(page) => {
                mark_page_read(conn, &page);
                page
            }
            None => {
                error!("Failed to select page");
                continue;
            }
        };
        handle_connection(pool, stream, page.url);
    }
}

fn handle_connection(
    pool: &Pool<ConnectionManager<SqliteConnection>>,
    mut stream: TcpStream,
    redirect: String,
) {
    info!("Handling http request.");
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap_or_default())
        .take_while(|line| !line.is_empty())
        .collect();
    if !http_request.is_empty() && http_request[0].contains("reload") {
        info!("Reloading sources...");
        // If the path contains "reload", do a sync
        let count = sync_sources(pool);
        info!("Reloaded {} sources", count);
    }
    let response = format!("HTTP/1.1 302 Found\r\nLocation: {}\r\n\r\n", redirect);
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
