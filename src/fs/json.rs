use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use crate::error::BError;
use serde_json::Value;

pub struct JsonFileReader {
    file_path: PathBuf,
}

impl JsonFileReader {
    pub fn parse(json_string: &str) -> Result<Value, BError> {
        let value: serde_json::Value = serde_json::from_str(json_string)?;
        Ok(value)
    }

    pub fn new(file_path: String) -> Self {
        JsonFileReader {
            file_path: PathBuf::from(file_path),
        }
    }

    pub fn read_json(&self) -> Result<String, BError> {
        let mut file: File = File::open(&self.file_path)?;
        let mut contents: String = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }
}