use mwr::crud::{create_source, establish_connection, get_sources};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <new_url>", args[0]);
        std::process::exit(1);
    }

    let new_url = &args[1];

    let connection = &mut establish_connection();

    create_source(connection, new_url);

    let results = get_sources(connection);

    println!("Displaying {} sources", results.len());

    for source in results.iter() {
        println!("id: {}", source.id);
        println!("url: {}", source.url);
        println!("timestamp: {}", source.added);
    }
}
