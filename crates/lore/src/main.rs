use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
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

fn scan(reset: bool) {
    let path = ".";
    let lore_paths = lore_core::repo::discover(path).expect("Failed to discover repository");
    if reset {
        lore_core::db::reset(&lore_paths.db_path).expect("Failed to reset database");
        lore_core::db::open(&lore_paths.db_path).expect("Failed to open database");
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Scan { reset }) => scan(*reset),
        None => {}
    }
}
