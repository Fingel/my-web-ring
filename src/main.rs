use clap::{Parser, Subcommand};
use mwr::crud::{create_source, establish_connection, get_source, get_sources};
use mwr::{download_source, parse_rss, sync_pages};
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
            let source = get_source(conn, id);
            let resp = download_source(&source.url).expect("Failed to download source");
            let channel = parse_rss(&resp).expect("Could not parse RSS");
            println!("{:?}", channel);
            println!("{:?}", channel.items);
            let saved = sync_pages(source);
            println!("Saved {} pages", saved);
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
            println!("No command provided, opening a new source");
        }
    }
}
