use clap::{Parser, Subcommand};
use diesel::{RunQueryDsl, sql_query};
use mwr::crud::{
    create_source, delete_source, establish_connection, get_pages, get_sources, mark_page_read,
};
use mwr::{print_source_list, select_page, sync_sources};
use std::thread;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync all Sources
    Reload,
    /// List all sources
    List,
    /// Add a new source
    Add { url: String },
    /// Delete a source
    Delete { id: i32 },
}

fn main() {
    let conn = &mut establish_connection();
    sql_query("PRAGMA foreign_keys = ON")
        .execute(conn)
        .expect("Failed to enable foreign keys");
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::List) => {
            let sources = get_sources(conn);
            print_source_list(&sources);
        }
        Some(Commands::Add { url }) => {
            let source = create_source(conn, &url);
            println!("Added {:?}", source);
        }
        Some(Commands::Reload) => {
            let saved = sync_sources(conn);
            println!("Saved {} pages", saved);
        }
        Some(Commands::Delete { id }) => {
            if let Ok(deleted) = delete_source(conn, id) {
                println!("Deleted {}", deleted);
            } else {
                println!("No source with that ID found.");
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
                if let Some(page) = select_page(conn) {
                    println!("Page selected: id: {}, url: {}", page.id, page.url);
                    if webbrowser::open(&page.url).is_ok() {
                        mark_page_read(conn, &page);
                    } else {
                        println!("Failed to open browser");
                    }
                } else {
                    println!("Could not find a page.")
                }
            }
            handle.join().unwrap();
        }
    }
}
