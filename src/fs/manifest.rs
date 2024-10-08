use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::error::BError;

pub struct Manifest {
    path: PathBuf,
    name: String,
    extension: String,
}

impl Manifest {
    pub fn new(path: &PathBuf) -> Result<Self, BError> {
        let name: String = path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .ok_or(BError::ParseManifestError(
                "Manifest file name is not valid UTF-8!".to_string(),
            ))?
            .to_string();

        let suffix: String = path
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or(BError::ParseManifestError(
                "Manifest file extension is not valid UTF-8!".to_string(),
            ))?
            .to_string();

        if suffix != "json" {
            return Err(BError::ParseManifestError(format!(
                "Unsupported manifest extension '{}'!",
                suffix
            )));
        }

        Ok(Manifest {
            path: path.clone(),
            name,
            extension: suffix,
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn extension(&self) -> &str {
        &self.extension
    }

    pub fn write(&self, json_str: &str) -> Result<(), BError> {
        if let Some(parent_dir) = self.path.parent() {
            std::fs::create_dir_all(parent_dir)?;
        }

        let mut file: File = File::create(&self.path)?;

        // Write the JSON string to the file.
        file.write_all(json_str.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::BError;
    use crate::fs::Manifest;

    use std::fs::File;
    use std::io::Read;
    use std::path::{Path, PathBuf};
    use tempdir::TempDir;

    #[test]
    fn test_manifest() {
        let json_test_str = r#"
        {
            TEST1: "value1",
            TEST2: "value2",
            TEST3: "value3",
            "data": {
                TEST4: "value4"
            }
        }
        "#;
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let manifest_path: PathBuf = path.join("test-manifest.json");
        let manifest: Manifest = Manifest::new(&manifest_path).expect("Failed to setup manifest!");

        assert_eq!(manifest.extension(), "json");
        assert_eq!(manifest.name(), "test-manifest.json");
        assert_eq!(manifest.path(), &manifest_path);
        assert!(!manifest.path().exists());
        manifest
            .write(json_test_str)
            .expect("Failed to write manifest file!");
        assert!(manifest.path().exists());

        let mut file: File = File::open(&manifest_path).expect("Failed to open manifest file!");
        let mut contents: String = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read manifes file!");
        assert_eq!(json_test_str, contents);
    }

    #[test]
    fn test_manifest_extension_error() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let manifest_path: PathBuf = path.join("test-manifest.txt");
        let result: Result<Manifest, BError> = Manifest::new(&manifest_path);
        match result {
            Ok(_m) => {
                panic!("We should have recived an error because the extension is not json");
            }
            Err(e) => {
                assert_eq!(e.to_string(), "Invalid 'manifest' node in build config. Unsupported manifest extension 'txt'!");
            }
        }
    }
}
