use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scans commits in the current repository and stores them in the database
    Scan {
        /// Reset the database before scanning (deletes all existing data)
        #[arg(long)]
        reset: bool,
    },
}

fn scan(reset: bool) -> anyhow::Result<()> {
    let lore_paths = lore_core::repo::discover(".")?;

    if reset {
        lore_core::db::reset(&lore_paths.db_path)?;
    }

    let _conn = lore_core::db::open(&lore_paths.db_path)?;

    println!("Repository: {}", lore_paths.root.display());
    println!("Database:   {}", lore_paths.db_path.display());
    println!("Scan not implemented yet.");

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { reset } => scan(reset)?,
    }

    Ok(())
}
