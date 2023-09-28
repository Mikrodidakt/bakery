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
        match serde_json::from_str(json_string) {
            Ok(data) => {
                Ok(data) 
            },
            Err(err) => {
                let error_message = format!("Failed to parse JSON: {}", err);
                Err(BError { code: 1, message: error_message })
            }
        }
    }

    pub fn new(file_path: String) -> Self {
        JsonFileReader {
            file_path: PathBuf::from(file_path),
        }
    }

    pub fn read_json(&self) -> Result<String, BError> {
        let mut file = File::open(&self.file_path).map_err(|err| BError {
            code: 1, // You may set the appropriate error code
            message: format!("Failed to open file: '{}'", err),
        })?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|err| BError {
            code: 1, // You may set the appropriate error code
            message: format!("Failed to parse json: '{}'", err),
        })?;

        Ok(contents)
    }
}