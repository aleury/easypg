use std::io::{Read, Write};
use std::process::Command;

fn main() -> std::io::Result<()> {
    let backup_source = env!("BACKUP_SOURCE");
    let backup_destination = env!("BACKUP_DESTINATION");

    // Find the filename of latest dump
    let ls_output = Command::new("aws")
        .args(["s3", "ls", format!("{backup_source}/").as_str()])
        .output()
        .expect("failed to execute ls");

    let ls_result = std::str::from_utf8(&ls_output.stdout).expect("failed convert stdout to str");

    let latest_filename = ls_result
        .lines()
        .flat_map(|line| line.split_whitespace().last())
        .last()
        .expect("failed to find the latest backup");
    println!("Latest backup is {latest_filename}. Downloading now...");

    // Download the latest dump found
    let mut cp_child = Command::new("aws")
        .args([
            "s3",
            "cp",
            format!("{backup_source}/{latest_filename}").as_str(),
            format!("{backup_destination}/").as_str(),
        ])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to download backup");
    let mut cp_stdout =
        std::mem::take(&mut cp_child.stdout).expect("couldn't attach to downloader stdout");

    let mut buf = [0u8; 1024];
    loop {
        let num_read = cp_stdout.read(&mut buf)?;
        if num_read == 0 {
            break;
        }
        let _ = std::io::stdout().write(&buf[..num_read])?;
    }
    let ecode = cp_child.wait().expect("failed to wait on downloader");
    assert!(ecode.success());

    // Decompress backup: .db.gz -> .db
    let mut gunzip_child = Command::new("gunzip")
        .arg(format!("{backup_destination}/{latest_filename}"))
        .spawn()
        .expect("failed to decompress backup");
    let ecode = gunzip_child
        .wait()
        .expect("failed to wait for decompression to finish");
    assert!(ecode.success());

    // Reset database
    // 1. dropdb db_name
    // 2. createdb -T template0 db_name
    // 3. psql $DATABASE_URL < ./backups/backup.db

    Ok(())
}
