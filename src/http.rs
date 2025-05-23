use crate::crud::mark_page_read;
use crate::{find_next_page, sync_sources};
use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use http::{Response, StatusCode};
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
        let page = match find_next_page(conn) {
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

    let response: Response<String> = Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header("Location", redirect)
        .body(String::new())
        .unwrap();
    let serialized = serialize_response_to_bytes(&response).unwrap();

    stream.write_all(&serialized).unwrap();
    stream.flush().unwrap();
}

fn serialize_response_to_bytes(response: &Response<String>) -> std::io::Result<Vec<u8>> {
    // Create a buffer to hold the serialized response
    let mut buffer = Vec::new();

    write_status_line(response, &mut buffer)?;
    write_headers(response.headers(), &mut buffer)?;
    writeln!(&mut buffer)?; // newline
    write_body(response.body(), &mut buffer);

    Ok(buffer)
}

fn write_status_line(response: &Response<String>, buf: &mut Vec<u8>) -> std::io::Result<()> {
    writeln!(buf, "HTTP/1.1 {}", response.status())?;
    Ok(())
}

fn write_headers(headers: &http::HeaderMap, buf: &mut Vec<u8>) -> std::io::Result<()> {
    for (name, value) in headers.iter() {
        writeln!(buf, "{}: {}", name.as_str(), value.to_str().unwrap_or(""))?;
    }
    Ok(())
}

fn write_body(body: &str, buf: &mut Vec<u8>) {
    buf.extend_from_slice(body.as_bytes());
}
