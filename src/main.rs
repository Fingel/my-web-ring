use clap::{Parser, Subcommand};
use mwr::crud::{
    create_source, establish_connection, get_pages, get_source_by_id, get_sources, mark_page_read,
};
use mwr::{sync_pages, sync_sources};
use rand::prelude::*;
use std::thread;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Reload a source
    Reload { id: i32 },
    /// List all sources
    List,
    /// Add a new source
    Add { url: String },
    /// Edit or delete a source
    Edit {
        /// ID of the source to edit
        id: u32,
        /// New URL for the source
        #[arg(short, long)]
        url: Option<String>,
        /// New weight for the source
        #[arg(short, long)]
        weight: Option<u32>,
        /// Delete the source
        #[arg(short, long)]
        delete: bool,
    },
}

fn main() {
    let conn = &mut establish_connection();
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::List) => {
            let results = get_sources(conn);

            println!("Displaying {} sources", results.len());

            for source in results.iter() {
                println!("id: {}", source.id);
                println!("weight: {}", source.weight);
                println!("url: {}", source.url);
                println!("timestamp: {}", source.added);
            }
        }
        Some(Commands::Add { url }) => {
            let source = create_source(conn, &url);
            println!("Added {:?}", source);
        }
        Some(Commands::Reload { id }) => {
            if let Some(source) = get_source_by_id(conn, id) {
                let saved = sync_pages(conn, source);
                println!("Saved {} pages", saved);
            } else {
                println!("Source not found");
            }
        }
        Some(Commands::Edit {
            id,
            url,
            weight,
            delete,
        }) => {
            if delete {
                println!("Deleting source with ID: {}", id);
            } else {
                println!("Editing source with ID: {}", id);
                if let Some(url) = url {
                    println!("New URL: {}", url);
                }
                if let Some(weight) = weight {
                    println!("New weight: {}", weight);
                }
            }
        }
        None => {
            let handle = thread::spawn(|| {
                let sync_conn = &mut establish_connection();
                println!("Syncing sources...");
                let count = sync_sources(sync_conn);
                println!("Done syncing sources. {} new pages", count);
            });
            let pages = get_pages(conn, true);
            if pages.is_empty() {
                println!("No pages available. Add a source first.");
            } else {
                println!("{} unread pages", pages.len());
                let mut rng = rand::rng();
                let page = pages.choose(&mut rng).unwrap();
                println!("Page selected: id: {}, url: {}", page.id, page.url);
                if webbrowser::open(&page.url).is_ok() {
                    mark_page_read(conn, page);
                } else {
                    println!("Failed to open browser");
                }
            }
            handle.join().unwrap();
        }
    }
}
