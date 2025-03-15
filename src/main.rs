use clap::{Parser, Subcommand};
use diesel::{
    SqliteConnection,
    connection::SimpleConnection,
    r2d2::{ConnectionManager, CustomizeConnection, Error, Pool},
};
use mwr::crud::{delete_source, get_pages, get_sources, mark_page_read, set_source_weight};
use mwr::{add_source, print_source_list, select_page, sync_sources};
use std::thread;
use std::time::Duration;
use std::{
    env,
    io::{Write, stdin, stdout},
};

#[derive(Debug)]
struct ConnectionOptions {
    enable_wal: bool,
    enable_foreign_keys: bool,
    busy_timeout: Option<Duration>,
}

impl CustomizeConnection<SqliteConnection, Error> for ConnectionOptions {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), Error> {
        if self.enable_wal {
            conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")
                .map_err(Error::QueryError)?;
        }
        if self.enable_foreign_keys {
            conn.batch_execute("PRAGMA foreign_keys = ON;")
                .map_err(Error::QueryError)?;
        }
        if let Some(d) = self.busy_timeout {
            conn.batch_execute(&format!("PRAGMA busy_timeout = {};", d.as_millis()))
                .map_err(Error::QueryError)?;
        }
        Ok(())
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Open a page and start the CLI interface (default)
    Run,
    /// Sync all Sources
    Reload,
    /// List all sources
    List,
    /// Add a new source
    Add { url: String },
    /// Delete a source
    Delete { id: i32 },
}

fn ui_loop(conn: &mut SqliteConnection) {
    let pages = get_pages(conn, true);
    if pages.is_empty() {
        println!("No pages found, add a source first.");
        return;
    }
    println!("{} unread pages", pages.len());

    loop {
        let page = select_page(conn).unwrap();
        if webbrowser::open(&page.url).is_ok() {
            mark_page_read(conn, &page);
        } else {
            println!("Failed to open browser");
        }
        println!("Current page: {} (source {})", page.url, page.source_id);
        println!("[n]ext - [u]n-weight - [i]ncrease-weight - [q]uit");
        stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "n" => continue,
            "u" => {
                let (new_weight, url) = set_source_weight(conn, page.source_id, -1);
                println!("Source {} weight updated to {}", url, new_weight);
            }
            "i" => {
                let (new_weight, url) = set_source_weight(conn, page.source_id, 1);
                println!("Source {} weight updated to {}", url, new_weight);
            }
            "q" => break,
            _ => println!("Invalid command"),
        }
    }
}

fn main() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = Pool::builder()
        .max_size(8)
        .connection_customizer(Box::new(ConnectionOptions {
            enable_wal: true,
            enable_foreign_keys: true,
            busy_timeout: Some(Duration::from_secs(30)),
        }))
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .expect("Could not build connection pool");
    let conn = &mut pool.get().expect("Failed to get connection");
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::List) => {
            let sources = get_sources(conn);
            print_source_list(&sources);
        }
        Some(Commands::Add { url }) => match add_source(conn, &url) {
            Ok(source) => println!("Added source: {}", source.url),
            Err(err) => println!("Failed to add source: {}", err),
        },
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
        Some(Commands::Run) | None => {
            let handle = thread::spawn(move || {
                let sync_conn = &mut pool.get().expect("Failed to get connection");
                println!("Syncing sources...");
                let count = sync_sources(sync_conn);
                println!("Done syncing sources. {} new pages", count);
            });
            ui_loop(conn);
            handle.join().unwrap();
        }
    }
}
