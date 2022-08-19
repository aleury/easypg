use std::{fs::File, io};

use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};

use easypg::compression::decompress;
use easypg::providers::{Local, Provider};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Resets database using the latest backup obtained from the provider
    Reset {
        /// What provider to fetch the latest backup from
        #[clap(short, long, arg_enum, default_value_t = Providers::Local, value_parser)]
        provider: Providers,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Providers {
    Local,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let backup_src = env!("BACKUP_SOURCE");
    let backup_dest = env!("BACKUP_DESTINATION");

    let provider = match &cli.command {
        Commands::Reset { provider } => match provider {
            Providers::Local => Local::new(backup_src, backup_dest),
        },
    };

    let backup = provider
        .get_latest_backup()
        .expect("failed to get latest backup");
    println!("Copied latest backup to {:?}", backup);

    // Decompress backup file
    let file = File::open(&backup)?;
    let size = file.metadata()?.len();
    let bar = ProgressBar::new(size);
    bar.set_style(
        ProgressStyle::with_template(
            "{msg} {bar:40.cyan/blue} {bytes}/{total_bytes} [{elapsed_precise}]",
        )
        .unwrap()
        .progress_chars("##-"),
    );
    bar.set_message("Decompressing");
    decompress(file, &backup, |n| bar.inc(n))?;
    bar.finish();

    Ok(())
}
