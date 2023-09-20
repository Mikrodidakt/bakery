use std::path::{Path, PathBuf};
use tempdir::TempDir;

use crate::error::BError;

#[derive(Debug)]
pub struct Archiver {
    path: PathBuf,
    name: String,
    extension: String,
    compression: String,
}

impl Archiver {
    pub fn new(path: &PathBuf) -> Result<Self, BError> {
        let archive_name = path.file_name().unwrap_or_default().to_string_lossy();
        // We read out all suffixes by splitting the archive name based on '.'
        let suffixes: Vec<String> = archive_name
            .split('.')
            .skip(1) // Skip the first part (the actual file name)
            .map(|suffix| suffix.to_string())
            .collect();
        let name: String = path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .ok_or(BError {
                code: 0,
                message: "Archive file name is not valid UTF-8!".to_string(),
            })?
            .to_string();
        let mut extensions: Vec<String> = suffixes.clone();
        let mut extension: String = suffixes.get(0).unwrap_or(&String::from("")).clone();
        let mut compression: String = "".to_string();

        println!("name: {}", name);
        suffixes.iter().for_each(|s| { println!("suffix: {}", s)} );
        println!("extension: {}", extension);
        println!("compression: {}", compression);
        if let Some(first_suffix) = suffixes.get(0) {
            println!("first suffix: {}", first_suffix);
            if ["tar", "zip"].contains(&first_suffix.as_str()) {
                if suffixes.len() > 2 {
                    // This is to make sure that we can handle archive names with . in the name
                    // but also to make sure that we can handle tar.gz or any other tar archive
                    // with compression.
                    println!("More than two suffixes we need to take special care");
                    if suffixes.last() == Some(&String::from(".zip")) {
                        // If the last is zip we will automatically assume all other suffixes are
                        // just part of the archive name for example archive.name.zip
                        extensions = vec!["zip".to_string()];
                    } else if suffixes.get(suffixes.len() - 2).cloned() == Some(String::from("tar")) {
                        // We take the length of the suffixes array minuz 2 because we assume there will be at
                        // most two suffixes when the archive is tar. We read out the archive suffixe and the
                        // compression suffixe and store them in the suffixe vector.
                        extensions = suffixes.iter().rev().take(2).cloned().collect();
                    } else if suffixes.get(0) == Some(&String::from("tar")) {
                        return Err(BError{ code: 0, message: "Archive must have an compression!".to_string() });
                    }
                }
            } else {
                return Err(BError{ code: 0, message: format!("Unsupported archive '{}'!", suffixes.get(0).unwrap()) });
            }
        } else {
            return Err(BError{ code: 0, message: "Archive must have an extension!".to_string() });
        }

        extension = extensions.get(0).unwrap().clone();
        if extensions.first() == Some(&String::from("tar")) {
            if extensions.len() < 2 {
                return Err(BError{ code: 0, message: "Archive must have an compression!".to_string() });
            }
            compression = extensions.last().unwrap().clone();
            if !["gz", "bz", "xz"].contains(&compression.as_str()) {
                return Err(BError{ code: 0, message: format!("Unsupported compression '{}'!", compression) });
            }
        }

        Ok(Archiver {
            path: path.clone(),
            name,
            extension,
            compression,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn extension(&self) -> &str {
        &self.extension
    }

    pub fn compression(&self) -> &str {
        &self.compression
    }

    pub fn add_files(&self, files: Vec<PathBuf>, tmp_dir: TempDir) -> Result<(), BError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::BError;
    use crate::fs::{json, Archiver};

    use core::arch;
    use std::fs::File;
    use std::io::{self, Read};
    use std::path::{Path, PathBuf};
    use tempdir::TempDir;

    #[test]
    fn test_archiver_zip() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-archiver.zip");
        let archiver: Archiver = Archiver::new(&archiver_path).expect("Failed to setup archiver!");
        assert_eq!(archiver.name(), "test-archiver.zip");
        assert_eq!(archiver.extension(), "zip");
        assert_eq!(archiver.compression(), "");
    }

    #[test]
    fn test_archiver_tar_gz() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-archiver.tar.gz");
        let archiver: Archiver = Archiver::new(&archiver_path).expect("Failed to setup archiver!");
        assert_eq!(archiver.name(), "test-archiver.tar.gz");
        assert_eq!(archiver.extension(), "tar");
        assert_eq!(archiver.compression(), "gz");
    }

    #[test]
    fn test_archiver_tar_bz() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-archiver.tar.bz");
        let archiver: Archiver = Archiver::new(&archiver_path).expect("Failed to setup archiver!");
        assert_eq!(archiver.name(), "test-archiver.tar.bz");
        assert_eq!(archiver.extension(), "tar");
        assert_eq!(archiver.compression(), "bz");
    }

    #[test]
    fn test_archiver_tar_xz() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-archiver.tar.xz");
        let archiver: Archiver = Archiver::new(&archiver_path).expect("Failed to setup archiver!");
        assert_eq!(archiver.name(), "test-archiver.tar.xz");
        assert_eq!(archiver.extension(), "tar");
        assert_eq!(archiver.compression(), "xz");
    }

    #[test]
    fn test_archiver_error_unsupported_archive() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-archiver.gzip");
        let error: BError = Archiver::new(&archiver_path).expect_err("We are expecting an error but got an Archiver");
        assert_eq!(error.message, "Unsupported archive 'gzip'!".to_string());
    }

    #[test]
    fn test_archiver_error_unsupported_compression() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-archiver.tar.invalid");
        let error: BError = Archiver::new(&archiver_path).expect_err("We are expecting an error but got an Archiver");
        assert_eq!(error.message, "Unsupported compression 'invalid'!".to_string());
    }

    #[test]
    fn test_archiver_error_no_compression() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let archiver_path: PathBuf = path.join("test-archiver.tar");
        let error: BError = Archiver::new(&archiver_path).expect_err("We are expecting an error but got an Archiver");
        assert_eq!(error.message, "Archive must have an compression!".to_string());
    }
}
