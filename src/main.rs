use clap::{Parser, Subcommand};
use diesel::{
    SqliteConnection,
    connection::SimpleConnection,
    r2d2::{ConnectionManager, CustomizeConnection, Error, Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use log::{LevelFilter, debug, info};
use mwr::{
    add_source, data_locations, find_next_page, find_next_page_by_source_id,
    logger::AsyncFileLogger, print_source_list, sync_sources,
};
use mwr::{
    backups::{backup, restore},
    crud::{
        delete_source, get_source_by_id, get_sources, mark_page_read, mark_page_unread,
        mark_source_read, set_source_weight,
    },
    http::server,
};
use std::io::{Write, stdin, stdout};
use std::thread;
use std::time::Duration;

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
    /// Select a page and start the CLI interface (default)
    Run,
    /// Select a page from a specific source
    Open { id: i32 },
    /// Fetch new pages
    Pull,
    /// List all sources
    List,
    /// Add a new source
    Add { url: String, title: Option<String> },
    /// Mark source as read
    MarkRead { id: i32 },
    /// Delete a source
    Delete { id: i32 },
    /// Backup sources and pages to stdout.
    Backup,
    /// Restore sources and pages from stdin.
    Restore,
    /// Start the HTTP server
    Server,
}

fn ui_loop(conn: &mut SqliteConnection) {
    loop {
        let page = match find_next_page(conn) {
            Some(page) => page,
            None => {
                println!("No unread pages found.");
                break;
            }
        };
        if webbrowser::open(&page.url).is_ok() {
            mark_page_read(conn, &page);
        } else {
            println!("Failed to open browser");
        }
        println!("\x1B[1m{} (source {})\x1B[0m", page.title, page.source_id);
        println!("[n]ext - [u]pvote - [d]ownvote - [r] mark unread - [q]uit");
        print!("> ");
        stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        match input.trim() {
            "n" => continue,
            "d" => {
                let (new_weight, url) = set_source_weight(conn, page.source_id, -1);
                println!("👎 {} ({})", url, new_weight);
            }
            "u" => {
                let (new_weight, url) = set_source_weight(conn, page.source_id, 1);
                println!("👍 {} ({})", url, new_weight);
            }
            "r" => {
                mark_page_unread(conn, &page);
                println!("Page {} marked unread", page.url);
            }
            "q" => break,
            _ => println!("Invalid command"),
        }
    }
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn main() {
    let proj_paths = data_locations();
    AsyncFileLogger::init(proj_paths.log, LevelFilter::Info).unwrap();
    let database_url = proj_paths.database;
    debug!("Database url: {:?}", database_url);
    let pool = Pool::builder()
        .max_size(8)
        .connection_customizer(Box::new(ConnectionOptions {
            enable_wal: false,
            enable_foreign_keys: true,
            busy_timeout: Some(Duration::from_secs(30)),
        }))
        .build(ConnectionManager::<SqliteConnection>::new(
            database_url.to_string_lossy(),
        ))
        .expect("Could not build connection pool");

    let conn = &mut pool.get().expect("Failed to get connection");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations, cannot continue.");

    let cli = Cli::parse();
    match cli.command {
        Some(Commands::List) => {
            let sources = get_sources(conn);
            print_source_list(conn, &sources);
        }
        Some(Commands::Add { url, title }) => match add_source(conn, &url, title) {
            Ok(source) => println!("Added source: {}", source.url),
            Err(err) => println!("Failed to add source: {}", err),
        },
        Some(Commands::Pull) => {
            let saved = sync_sources(&pool);
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
                let new_pages = sync_sources(&pool);
                info!("Synced {} new pages", new_pages);
            });
            ui_loop(conn);
            handle.join().unwrap();
        }
        Some(Commands::Open { id }) => {
            let source = get_source_by_id(conn, id).unwrap();
            println!("Opening source: {}", source.title);
            let page = find_next_page_by_source_id(conn, id).unwrap();
            if webbrowser::open(&page.url).is_ok() {
                mark_page_read(conn, &page);
            } else {
                println!("Failed to open browser");
            }
        }
        Some(Commands::MarkRead { id }) => {
            let pages_marked = mark_source_read(conn, id);
            println!("Marked {} pages as read", pages_marked.len());
        }
        Some(Commands::Backup) => {
            backup();
        }
        Some(Commands::Restore) => {
            restore();
        }
        Some(Commands::Server) => {
            server(&pool);
        }
    }
}
