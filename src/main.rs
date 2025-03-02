use anyhow::{Result, Context};
use clap::{Parser, Subcommand};
use studfinder::{Config, StudFinder, ScanQuality, ExportFormat, ProcessorType};
use std::path::PathBuf;
use tracing::{error, info, debug};

#[derive(Parser)]
#[command(name = "studfinder")]
#[command(about = "Vision-based LEGO piece identifier and cataloging tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Scan and identify LEGO pieces")]
    Scan {
        #[arg(help = "Path to image file")]
        path: PathBuf,

        #[arg(short, long, help = "Process entire directory")]
        batch: bool,
    },

    #[command(about = "Initialize database and configuration")]
    Init,

    #[command(about = "Reset database (warning: destroys all data)")]
    Reset {
        #[arg(short, long, help = "Skip confirmation prompt")]
        force: bool,
    },

    #[command(about = "Manage piece inventory")]
    Inventory {
        #[command(subcommand)]
        action: InventoryCommands,
    },
}

#[derive(Subcommand)]
enum InventoryCommands {
    #[command(about = "List all pieces")]
    List,

    #[command(about = "Export inventory to file")]
    Export {
        #[arg(help = "Path to export file")]
        path: PathBuf,
    },

    #[command(about = "Import inventory from file")]
    Import {
        #[arg(help = "Path to import file")]
        path: PathBuf,
    },
}

fn setup_logging(verbose: bool) -> Result<()> {
    if verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    setup_logging(cli.verbose)?;

    let config = get_default_config()
        .context("Failed to get default configuration")?;

    let studfinder = StudFinder::new(config)
        .context("Failed to initialize StudFinder")?;

    match cli.command {
        Commands::Init => {
            info!("Initializing studfinder...");
            studfinder.init()
                .context("Failed to initialize")?;
            info!("Initialization complete");
        }
        Commands::Reset { force } => {
            if !force {
                println!("WARNING: This will delete all stored data. Are you sure? [y/N]");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Reset cancelled");
                    return Ok(());
                }
            }
            info!("Resetting database...");
            studfinder.reset()
                .context("Failed to reset database")?;
            info!("Reset complete");
        }
        Commands::Scan { path, batch } => {
            if batch {
                info!("Processing directory: {}", path.display());
                process_directory(&studfinder, path).await
                    .context("Failed to process directory")?;
            } else {
                info!("Processing image: {}", path.display());
                process_single_image(&studfinder, path).await
                    .context("Failed to process image")?;
            }
        }
        Commands::Inventory { action } => match action {
            InventoryCommands::List => {
                let pieces = studfinder.list_inventory()
                    .context("Failed to list inventory")?;
                if pieces.is_empty() {
                    println!("No pieces in inventory");
                } else {
                    println!("\nInventory:");
                    println!("{:<36} {:<8} {:<10} {:<8} {:<10}", "ID", "PART#", "COLOR", "QTY", "CONFIDENCE");
                    println!("{}", "-".repeat(75));
                    for piece in pieces {
                        println!("{:<36} {:<8} {:<10} {:<8} {:.1}%",
                            piece.id,
                            piece.part_number,
                            piece.color,
                            piece.quantity,
                            piece.confidence * 100.0
                        );
                    }
                    println!();
                }
            }
            InventoryCommands::Export { path } => {
                info!("Exporting inventory to: {}", path.display());
                studfinder.export_inventory(path)
                    .context("Failed to export inventory")?;
                info!("Export complete");
            }
            InventoryCommands::Import { path } => {
                info!("Importing inventory from: {}", path.display());
                studfinder.import_inventory(path)
                    .context("Failed to import inventory")?;
                info!("Import complete");
            }
        },
    }

    Ok(())
}

fn get_default_config() -> Result<Config> {
    let dirs = directories::ProjectDirs::from("com", "studfinder", "studfinder")
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    let data_dir = dirs.data_dir();
    std::fs::create_dir_all(data_dir)
        .context("Failed to create data directory")?;

    Ok(Config {
        database_path: data_dir.join("studfinder.db"),
        export_format: ExportFormat::Json,
        scan_quality: ScanQuality::Balanced,
        processor_type: ProcessorType::Scanner,
        confidence_threshold: 0.8,
    })
}

async fn process_directory(studfinder: &StudFinder, dir: PathBuf) -> Result<()> {
    let mut successful = 0;
    let mut failed = 0;

    for entry in std::fs::read_dir(&dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            match process_single_image(studfinder, path.clone()).await {
                Ok(()) => {
                    successful += 1;
                    debug!("Successfully processed: {}", path.display());
                },
                Err(e) => {
                    failed += 1;
                    error!("Failed to process {}: {}", path.display(), e);
                }
            }
        }
    }

    info!(
        "Batch processing complete. Successful: {}, Failed: {}",
        successful, failed
    );
    Ok(())
}

async fn process_single_image(studfinder: &StudFinder, path: PathBuf) -> Result<()> {
    info!("Processing image: {}", path.display());

    let piece = studfinder.scan_image(path)
        .await
        .context("Failed to scan image")?;

    info!("Detected: {} {} {} (confidence: {:.1}%)",
        piece.color,
        piece.category,
        piece.part_number,
        piece.confidence * 100.0
    );

    studfinder.add_piece(piece)
        .context("Failed to add piece to inventory")?;

    Ok(())
}
