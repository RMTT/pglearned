use clap::{Parser, Subcommand};
use postgres::{Client, NoTls};
use std::env;

#[derive(Parser)]
#[command(name = "pgl")]
#[command(about = "CLI for pglearned extension interaction")]
struct Cli {
    #[arg(long, env = "DATABASE_URL")]
    db_url: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage query datasets
    Qdataset {
        #[command(subcommand)]
        cmd: QDatasetCommands,
    },
}

#[derive(Subcommand)]
enum QDatasetCommands {
    /// List all datasets
    Ls,
    /// Create a new dataset
    Create {
        /// Name of the dataset
        name: String,
    },
    /// Import queries from a file (server-side path)
    Import {
        /// Name of the dataset
        name: String,
        /// Path to the file on the database server
        file_path: String,
    },
    /// Insert a single query
    Insert {
        /// Name of the dataset
        name: String,
        /// SQL query string
        query: String,
    },
    /// Delete a dataset
    Delete {
        /// Name of the dataset
        name: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let db_url = cli
        .db_url
        .or_else(|| env::var("DATABASE_URL").ok())
        .unwrap_or_else(|| "postgres://postgres:postgres@localhost:5432/postgres".to_string());

    let mut client = Client::connect(&db_url, NoTls)?;

    match cli.command {
        Commands::Qdataset { cmd } => {
            match cmd {
                QDatasetCommands::Ls => {
                    let rows = client.query("SELECT id, dataset_name, current_pos FROM pgl.pgl_qdataset_status ORDER BY id", &[])?;
                    if rows.is_empty() {
                        println!("No datasets found.");
                    } else {
                        println!(
                            "{:<5} | {:<30} | {:<15}",
                            "ID", "Dataset Name", "Current Pos"
                        );
                        println!("{:-<5}-+-{:-<30}-+-{:-<15}", "", "", "");
                        for row in rows {
                            let id: i64 = row.get("id");
                            let name: String = row.get("dataset_name");
                            let pos: i64 = row.get("current_pos");
                            println!("{:<5} | {:<30} | {:<15}", id, name, pos);
                        }
                    }
                }
                QDatasetCommands::Create { name } => {
                    client.execute("SELECT pgl_qdataset_create($1)", &[&name])?;
                    println!("Dataset '{}' created successfully.", name);
                }
                QDatasetCommands::Import { name, file_path } => {
                    client.execute("SELECT pgl_qdataset_import($1, $2)", &[&name, &file_path])?;
                    println!(
                        "Imported queries from '{}' into dataset '{}'.",
                        file_path, name
                    );
                }
                QDatasetCommands::Insert { name, query } => {
                    client.execute("SELECT pgl_qdataset_insert($1, $2)", &[&name, &query])?;
                    println!("Query inserted into dataset '{}'.", name);
                }
                QDatasetCommands::Delete { name } => {
                    client.execute("SELECT pgl_qdataset_delete($1)", &[&name])?;
                    println!("Dataset '{}' deleted successfully.", name);
                }
            }
        }
    }

    Ok(())
}
