use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

pub struct JsonFileReader {
    file_path: PathBuf,
}

impl JsonFileReader {
    pub fn new(file_path: String) -> Self {
        JsonFileReader {
            file_path: PathBuf::from(file_path),
        }
    }

    pub fn read_json(&self) -> Result<String, io::Error> {
        let mut file = File::open(&self.file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(contents)
    }
}