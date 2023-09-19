use std::path::{PathBuf, Path};
use std::fs::File;
use std::io::Write;

use crate::error::BError;

pub struct Manifest {
    path: PathBuf,
    name: String,
    extension: String,
}

impl Manifest {
    pub fn new(path: &PathBuf) -> Result<Self, BError> {
        let mut name: String = String::new();
        if let Some(file_name) = path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                let file_name_string = file_name_str.to_string();
                name = file_name_string;
            } else {
                return Err(BError{ code: 0, message: format!("Manifest file name is not valid UTF-8!")});
            }
        } else {
            return Err(BError{ code: 0, message: format!("Manifest file path does not have a valid file name!")});
        }

        let mut suffix: String = String::new();
        if let Some(extension) = path.extension() {
            if let Some(extension_str) = extension.to_str() {
                suffix = extension_str.to_string();
            } else {
                return Err(BError{ code: 0, message: format!("Manifest file name is not valid UTF-8!")});
            }
        } else {
            return Err(BError{ code: 0, message: format!("Manifest must have an extension!")});
        }

        if !suffix.eq("json") {
            return Err(BError{ code: 0, message: format!("Unsupported manifest extension '{}'!", suffix)});
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
            std::fs::create_dir_all(parent_dir).map_err(|err| BError {
                code: 1, // You may set the appropriate error code
                message: format!("Failed to create paranets '{}'", err),
            })?;
        }

        let mut file: File = File::create(&self.path).map_err(|err| BError {
            code: 1, // You may set the appropriate error code
            message: format!("Failed to create manifest file '{}'", err),
        })?;

        // Write the JSON string to the file.
        file.write_all(json_str.as_bytes()).map_err(|err| BError {
            code: 1, // You may set the appropriate error code
            message: format!("Failed to write to manifest '{}'", err),
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::fs::{Manifest, json};
    use crate::error::BError;

    use std::path::{PathBuf, Path};
    use tempdir::TempDir;
    use std::fs::File;
    use std::io::{self, Read};

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
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let manifest_path: PathBuf = path.join("test-manifest.json");
        let manifest: Manifest = Manifest::new(&manifest_path).expect("Failed to setup manifest!");
        
        assert_eq!(manifest.extension(), "json");
        assert_eq!(manifest.name(), "test-manifest.json");
        assert_eq!(manifest.path(), &manifest_path);
        assert!(!manifest.path().exists());
        manifest.write(json_test_str).expect("Failed to write manifest file!");
        assert!(manifest.path().exists());

        let mut file: File = File::open(&manifest_path).expect("Failed to open manifest file!");
        let mut contents: String = String::new();
        file.read_to_string(&mut contents).expect("Failed to read manifes file!");
        assert_eq!(json_test_str, contents);
    }

    #[test]
    fn test_manifest_extension_error() {
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
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let manifest_path: PathBuf = path.join("test-manifest.txt");
        let result: Result<Manifest, BError> = Manifest::new(&manifest_path);
        match result {
            Ok(m) => {
                panic!("We should have recived an error because the extension is not json");
            },
            Err(e) => {
                assert_eq!(e.message, "Unsupported manifest extension 'txt'!");
            }
        }
    }
}