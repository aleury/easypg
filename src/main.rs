mod providers;

use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
};

use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};

use providers::{Local, Provider};

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
        #[clap(short, long, arg_enum, default_value_t = StorageProvider::Local, value_parser)]
        provider: StorageProvider,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum StorageProvider {
    Local,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let backup_src = env!("BACKUP_SOURCE");
    let backup_dest = env!("BACKUP_DESTINATION");

    let provider = match &cli.command {
        Commands::Reset { provider } => match provider {
            StorageProvider::Local => Local::new(backup_src, backup_dest),
        },
    };

    let backup = provider
        .get_latest_backup()
        .expect("failed to get latest backup");
    println!("Copied latest backup to {:?}", backup);

    let backup_file = File::open(&backup)?;
    let backup_file_size = backup_file.metadata()?.len();
    let mut backup_reader = BufReader::new(backup_file);

    let decompressed_filepath = backup.trim_end_matches(".gz");
    let decompressed_file = File::create(decompressed_filepath)?;
    let mut file_writer = BufWriter::new(decompressed_file);
    let mut decompressed_file_writer = flate2::write::GzDecoder::new(&mut file_writer);

    let bar = ProgressBar::new(backup_file_size);
    bar.set_style(
        ProgressStyle::with_template(
            "{msg} {bar:40.cyan/blue} {bytes}/{total_bytes} [{elapsed_precise}]",
        )
        .unwrap()
        .progress_chars("##-"),
    );
    bar.set_message("Decompressing");

    let mut buf = [0u8; 8192];
    loop {
        let num_read = backup_reader.read(&mut buf)?;
        if num_read == 0 {
            break;
        }
        decompressed_file_writer.write_all(&buf[..num_read])?;
        bar.inc(num_read as u64);
    }

    bar.finish();

    Ok(())
}
