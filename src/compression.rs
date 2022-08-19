use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
};

pub fn decompress<F: Fn(u64)>(file: File, destination: &str, callback: F) -> io::Result<()> {
    let mut reader = BufReader::new(file);
    let decompressed_file = File::create(destination.trim_end_matches(".gz"))?;
    let mut writer = BufWriter::new(decompressed_file);
    let mut decoder = flate2::write::GzDecoder::new(&mut writer);

    let mut buf = [0u8; 8192];
    loop {
        let num_read = reader.read(&mut buf)?;
        if num_read == 0 {
            break;
        }
        decoder.write_all(&buf[..num_read])?;
        callback(num_read as u64)
    }

    Ok(())
}
