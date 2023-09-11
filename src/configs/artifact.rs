use crate::configs::Config;
use serde_json::Value;
use crate::error::BError;

pub struct ArtifactConfig {
    ttype: String, // Optional if not set for the task the default type 'file' is used
    name: String, // The name can be a name for a directory, archive, file or manifest
    source: String, // The source is only used if the type is file 
    dest: String, // The dest is optional
    artifacts: Vec<ArtifactConfig>, // The artifacts can be a composite of multiple artifacts nodes
}

impl Config for ArtifactConfig {
}

impl ArtifactConfig {
    fn get_artifacts(data: &Value) -> Result<Vec<ArtifactConfig>, BError> {
        match data.get("artifacts") {
            Some(value) => {
                if value.is_array() {
                    if let Some(artifact_vec) = value.as_array() {
                        let mut artifacts: Vec<ArtifactConfig> = Vec::new();
                        for artifact_data in artifact_vec.iter() {
                            let artifact: ArtifactConfig = ArtifactConfig::from_value(&artifact_data)?;
                            artifacts.push(artifact);
                        }
                        return Ok(artifacts);
                    }
                    return Err(BError{ code: 0, message: format!("Invalid 'artifacts' format in build config")});
                } else {
                    return Err(BError{ code: 0, message: format!("Invalid 'artifacts' format in build config")});
                }
            }
            None => {
                return Ok(Vec::new());
            }
        }
    }

    pub fn from_str(json_string: &str) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data)
    }

    pub fn from_value(data: &Value) -> Result<Self, BError> {
        let ttype: String = Self::get_str_value("type", &data, Some(String::from("file")))?;
        let name: String = Self::get_str_value("name", &data, Some(String::from("")))?;
        let source: String = Self::get_str_value("source", &data, Some(String::from("")))?;
        let dest: String = Self::get_str_value("dest", &data, Some(String::from("")))?;
        if ttype != "file" && ttype != "directory" && ttype != "archive" && ttype != "manifest" {
            return Err(BError{ code: 0, message: format!("Invalid 'artifact' format in build config. Invalid type '{}'", ttype)});
        }
        if ttype == "file" && source.is_empty() {
            return Err(BError{ code: 0, message: format!("Invalid 'artifact' format in build config. The 'file' type requires a defined 'source'")});
        }
        if ttype == "directory" && name.is_empty() {
            return Err(BError{ code: 0, message: format!("Invalid 'artifact' format in build config. The 'directory' type requires a 'name'")}); 
        }
        if ttype == "archive" && name.is_empty() {
            return Err(BError{ code: 0, message: format!("Invalid 'artifact' format in build config. The 'archive' type requires a 'name'")}); 
        }
        if ttype == "manifest" && name.is_empty() {
            return Err(BError{ code: 0, message: format!("Invalid 'artifact' format in build config. The 'manifest' type requires a 'name'")}); 
        }
        let artifacts: Vec<ArtifactConfig> = Self::get_artifacts(&data)?;
        Ok(ArtifactConfig {
            name,
            ttype,
            source,
            dest,
            artifacts: artifacts,
        })
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn ttype(&self) -> &String {
        &self.ttype
    }
    
    pub fn source(&self) -> &String {
        &self.source
    }

    pub fn dest(&self) -> &String {
        &self.dest
    }

    pub fn artifacts(&self) -> &Vec<ArtifactConfig> {
        &self.artifacts
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::ArtifactConfig;
    use crate::error::BError;

    fn helper_artifact_config_from_str(json_test_str: &str) -> ArtifactConfig {
        let result: Result<ArtifactConfig, BError> = ArtifactConfig::from_str(json_test_str);
        match result {
            Ok(rconfig) => {
                rconfig
            }
            Err(e) => {
                eprintln!("Error parsing artifacts from build config: {}", e);
                panic!();
            } 
        }
    }

    #[test]
    fn test_artifact_config_file_type_simple() {
        let json_test_str = r#"
        {
            "source": "file1.txt"
        }
        "#;
        let config = helper_artifact_config_from_str(json_test_str);
        assert!(config.name().is_empty());
        assert_eq!(config.ttype(), "file");
        assert_eq!(config.source(), "file1.txt");
        assert!(config.dest().is_empty());
    }

    #[test]
    fn test_artifact_config_file_type_dest() {
        let json_test_str = r#"
        {
            "source": "file1.txt",
            "dest": "dest"
        }
        "#;
        let config = helper_artifact_config_from_str(json_test_str);
        assert!(config.name().is_empty());
        assert_eq!(config.ttype(), "file");
        assert_eq!(config.source(), "file1.txt");
        assert_eq!(config.dest(), "dest");
    }

    #[test]
    fn test_artifact_config_file_type() {
        let json_test_str = r#"
        {
            "type": "file",
            "source": "file1.txt",
            "dest": "dest"
        }
        "#;
        let config = helper_artifact_config_from_str(json_test_str);
        assert!(config.name().is_empty());
        assert_eq!(config.ttype(), "file");
        assert_eq!(config.source(), "file1.txt");
        assert_eq!(config.dest(), "dest");
    }

    #[test]
    fn test_artifact_config_dir_type() {
        let json_test_str = r#"
        {
            "type": "directory",
            "name": "dir",
            "artifacts": [
                {
                    "source": "file1.txt"
                }
            ]
        }
        "#;
        let config = helper_artifact_config_from_str(json_test_str);
        assert_eq!(config.ttype(), "directory");
        assert_eq!(config.name(), "dir");
        assert!(!config.artifacts().is_empty());
        let artifacts = config.artifacts();
        assert_eq!(artifacts[0].ttype(), "file");
        assert!(artifacts[0].name().is_empty());
        assert_eq!(artifacts[0].source(), "file1.txt");
        assert!(artifacts[0].dest().is_empty());
    }

    #[test]
    fn test_artifact_config_composite() {
        let json_test_str = r#"
        {
            "type": "archive",
            "name": "test.zip",
            "artifacts": [
                {
                    "type": "directory",
                    "name": "dir-name",
                    "artifacts": [
                        {
                            "source": "file1.txt"
                        },
                        {
                            "source": "file2.txt"
                        }
                    ]
                }
            ]
        }
        "#;
        let config = helper_artifact_config_from_str(json_test_str);
        assert_eq!(config.ttype(), "archive");
        assert_eq!(config.name(), "test.zip");
        assert!(!config.artifacts().is_empty());
        let artifacts = config.artifacts();
        assert_eq!(artifacts[0].ttype(), "directory");
        assert_eq!(artifacts[0].name(), "dir-name");
        assert!(!artifacts[0].artifacts().is_empty());
        let files = artifacts[0].artifacts();
        let mut i = 1;
        for f in files.iter() {
            assert_eq!(f.ttype(), "file");
            assert!(f.name().is_empty());
            assert_eq!(f.source(), &format!("file{}.txt", i));
            assert!(f.dest().is_empty());
            i += 1;
        }
    }

    #[test]
    fn test_artifact_config_error_invalid_type() {
        let json_test_str = r#"
        {
            "type": "invalid",
            "source": "file1.txt",
            "dest": "dest"
        }
        "#;
        let result: Result<ArtifactConfig, BError> = ArtifactConfig::from_str(json_test_str);
        match result {
            Ok(_rconfig) => {
                panic!("We should have recived an error because the type is invalid!");
            }
            Err(e) => {
                assert_eq!(e.message, String::from("Invalid 'artifact' format in build config. Invalid type 'invalid'"));
            } 
        }
    }

    #[test]
    fn test_artifact_config_error_no_dir_name() {
        let json_test_str = r#"
        {
            "type": "directory"
        }
        "#;
        let result: Result<ArtifactConfig, BError> = ArtifactConfig::from_str(json_test_str);
        match result {
            Ok(_rconfig) => {
                panic!("We should have recived an error because the type is invalid!");
            }
            Err(e) => {
                assert_eq!(e.message, String::from("Invalid 'artifact' format in build config. The 'directory' type requires a 'name'"));
            } 
        }
    }

    #[test]
    fn test_artifact_config_error_no_manifest_name() {
        let json_test_str = r#"
        {
            "type": "manifest"
        }
        "#;
        let result: Result<ArtifactConfig, BError> = ArtifactConfig::from_str(json_test_str);
        match result {
            Ok(_rconfig) => {
                panic!("We should have recived an error because the type is invalid!");
            }
            Err(e) => {
                assert_eq!(e.message, String::from("Invalid 'artifact' format in build config. The 'manifest' type requires a 'name'"));
            } 
        }
    }
}