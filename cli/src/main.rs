use clap::{Parser, Subcommand};
use log::info;
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
    /// Run a dataset and save the output
    Run {
        /// Name of the dataset
        name: String,
        /// Continue from the last position (offset -1). Default is false (offset 0).
        #[arg(long)]
        r#continue: bool,
        /// Output file path
        #[arg(long)]
        out: String,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
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
                    info!("Dataset '{}' created successfully.", name);
                }
                QDatasetCommands::Import { name, file_path } => {
                    client.execute("SELECT pgl_qdataset_import($1, $2)", &[&name, &file_path])?;
                    info!(
                        "Imported queries from '{}' into dataset '{}'.",
                        file_path, name
                    );
                }
                QDatasetCommands::Insert { name, query } => {
                    client.execute("SELECT pgl_qdataset_insert($1, $2)", &[&name, &query])?;
                    info!("Query inserted into dataset '{}'.", name);
                }
                QDatasetCommands::Delete { name } => {
                    client.execute("SELECT pgl_qdataset_delete($1)", &[&name])?;
                    info!("Dataset '{}' deleted successfully.", name);
                }
                QDatasetCommands::Run {
                    name,
                    r#continue,
                    out,
                } => {
                    let mut offset: i64 = if r#continue { -1 } else { 0 };
                    let limit: i64 = 20;
                    let mut all_results = Vec::new();

                    info!("Running dataset '{}'...", name);
                    if r#continue {
                        info!("Continuing from last position.");
                    } else {
                        info!("Starting from the beginning (offset 0).");
                    }

                    loop {
                        let rows = client.query(
                            "SELECT plan FROM pgl_qdataset_collect($1, $2, $3)",
                            &[&name, &offset, &limit],
                        )?;

                        let batch_size = rows.len();
                        if batch_size == 0 {
                            break;
                        }

                        for row in rows {
                            let plan: serde_json::Value = row.get("plan");
                            all_results.push(plan);
                        }

                        if batch_size < limit as usize {
                            break;
                        }

                        // Use offset -1 for subsequent batches
                        offset = -1;
                    }

                    let file = std::fs::File::create(&out)?;
                    serde_json::to_writer_pretty(file, &all_results)?;
                    info!("Saved {} results to '{}'.", all_results.len(), out);
                }
            }
        }
    }

    Ok(())
}
