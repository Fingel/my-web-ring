use crate::crud::mark_page_read;
use crate::select_page;
use diesel::SqliteConnection;
use std::io::{BufRead, BufReader, prelude::*};
use std::net::{TcpListener, TcpStream};

pub fn server(conn: &mut SqliteConnection) {
    let listener = TcpListener::bind("0.0.0.0:8090").unwrap();
    println!("Listening on http://0.0.0.0:8090");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let page = match select_page(conn) {
            Some(page) => {
                mark_page_read(conn, &page);
                page
            }
            None => {
                eprintln!("Failed to select page");
                continue;
            }
        };
        handle_connection(stream, page.url);
    }
}

fn handle_connection(mut stream: TcpStream, redirect: String) {
    let buf_reader = BufReader::new(&stream);
    let _http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();
    let response = format!("HTTP/1.1 302 Found\r\nLocation: {}\r\n\r\n", redirect);
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
