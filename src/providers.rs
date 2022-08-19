use std::{fs, io};

type ProviderResult<T> = Result<T, Box<dyn std::error::Error>>;

pub trait Provider {
    fn get_latest_backup(&self) -> ProviderResult<String>;
}

pub struct Local<'a> {
    source: &'a str,
    destination: &'a str,
}

impl<'a> Local<'a> {
    pub fn new(source: &'a str, destination: &'a str) -> Self {
        Self {
            source,
            destination,
        }
    }
}

impl Provider for Local<'_> {
    fn get_latest_backup(&self) -> ProviderResult<String> {
        let mut entries = fs::read_dir(&self.source)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        entries.sort();

        let latest_entry = entries.into_iter().filter(|p| p.is_file()).last();

        match latest_entry {
            Some(path) => {
                let filename = path.file_name().unwrap().to_string_lossy().into_owned();
                let destination = format!("{}/{}", self.destination, filename);
                fs::copy(&path, &destination)?;
                Ok(destination)
            }
            None => Err(Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                "no backups found",
            ))),
        }
    }
}
