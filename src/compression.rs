use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
};

pub fn decompress<F>(file: File, destination: &str, callback: F) -> io::Result<()>
where
    F: Fn(u64) -> (),
{
    let mut backup_reader = BufReader::new(file);
    let decompressed_filepath = destination.trim_end_matches(".gz");
    let decompressed_file = File::create(decompressed_filepath)?;
    let mut file_writer = BufWriter::new(decompressed_file);
    let mut decompressed_file_writer = flate2::write::GzDecoder::new(&mut file_writer);

    let mut buf = [0u8; 8192];
    loop {
        let num_read = backup_reader.read(&mut buf)?;
        if num_read == 0 {
            break;
        }
        decompressed_file_writer.write_all(&buf[..num_read])?;
        callback(num_read as u64)
    }

    Ok(())
}
