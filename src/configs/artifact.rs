use crate::configs::{Config, Context};
use serde_json::Value;
use crate::error::BError;

#[derive(Clone, PartialEq, Debug)]
pub enum AType {
    File,
    Directory,
    Archive,
    Manifest,
}

//TODO: we should consider using IndexSet instead of vector to make sure we
// keep the order from the json file
pub struct ArtifactConfig {
    pub atype: AType, // Optional if not set for the task the default type 'file' is used
    pub name: String, // The name can be a name for a directory, archive, file or manifest
    pub source: String, // The source is only used if the type is file 
    pub dest: String, // The dest is optional
    pub artifacts: Vec<ArtifactConfig>, // The artifacts can be a composite of multiple artifacts nodes
    pub manifest: String, // The manifest content will be a json string that can be put in a file. The manifest can then be used by the CI to collect information from the build
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
        let manifest: String = Self::get_str_manifest("content", &data, Some(String::from("{}")))?;
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

        let enum_ttype: AType;
        match ttype.as_str() {
            "file" => {
                enum_ttype = AType::File;
            },
            "directory" => {
                enum_ttype = AType::Directory;
            },
            "archive" => {
                enum_ttype = AType::Archive;
            },
            "manifest" => {
                enum_ttype = AType::Manifest;
            },
            _ => {
                return Err(BError{ code: 0, message: format!("Invalid 'artifact' format in build config. Invalid type '{}'", ttype)});
            },
        }

        let artifacts: Vec<ArtifactConfig> = Self::get_artifacts(&data)?;
        Ok(ArtifactConfig {
            name,
            atype: enum_ttype,
            source,
            dest,
            artifacts,
            manifest,
        })
    }

    pub fn expand_ctx(&mut self, ctx: &Context) {
        match self.atype {
            AType::File => {
                self.name = ctx.expand_str(&self.name);
                self.source = ctx.expand_str(&self.source);
                self.dest = ctx.expand_str(&self.dest);
            },
            AType::Directory => {
                self.name = ctx.expand_str(&self.name);
                self.artifacts.iter_mut().for_each(|a: &mut ArtifactConfig| a.expand_ctx(ctx));
            },
            AType::Archive => {
                self.name = ctx.expand_str(&self.name);
                self.artifacts.iter_mut().for_each(|a: &mut ArtifactConfig| a.expand_ctx(ctx));
            },
            AType::Manifest => {
                self.name = ctx.expand_str(&self.name);
                self.manifest = ctx.expand_str(&self.manifest);
            },
            _ => {
                panic!("Invalid 'artifact' format in build config. Invalid type '{:?}'", self.atype);
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::{ArtifactConfig, AType, Context};
    use crate::error::BError;

    use indexmap::{IndexMap, indexmap};

    // TODO: we need to add tests so we cover all the different types inclduing specifying the source and dest

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
        assert!(config.name.is_empty());
        assert_eq!(config.atype, AType::File);
        assert_eq!(config.source, "file1.txt");
        assert!(config.dest.is_empty());
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
        assert!(config.name.is_empty());
        assert_eq!(config.atype, AType::File);
        assert_eq!(config.source, "file1.txt");
        assert_eq!(config.dest, "dest");
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
        assert!(config.name.is_empty());
        assert_eq!(config.atype, AType::File);
        assert_eq!(config.source, "file1.txt");
        assert_eq!(config.dest, "dest");
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
        let config: ArtifactConfig = helper_artifact_config_from_str(json_test_str);
        assert_eq!(config.atype, AType::Directory);
        assert_eq!(config.name, "dir");
        assert!(!config.artifacts.is_empty());
        let artifacts: Vec<ArtifactConfig> = config.artifacts;
        assert_eq!(artifacts[0].atype, AType::File);
        assert!(artifacts[0].name.is_empty());
        assert_eq!(artifacts[0].source, "file1.txt");
        assert!(artifacts[0].dest.is_empty());
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
        let config: ArtifactConfig = helper_artifact_config_from_str(json_test_str);
        assert_eq!(config.atype, AType::Archive);
        assert_eq!(config.name, "test.zip");
        assert!(!config.artifacts.is_empty());
        let artifacts: Vec<ArtifactConfig> = config.artifacts;
        assert_eq!(artifacts[0].atype, AType::Directory);
        assert_eq!(artifacts[0].name, "dir-name");
        assert!(!artifacts[0].artifacts.is_empty());
        let files: &Vec<ArtifactConfig> = &artifacts[0].artifacts;
        let mut i = 1;
        for f in files.iter() {
            assert_eq!(f.atype, AType::File);
            assert!(f.name.is_empty());
            assert_eq!(f.source, format!("file{}.txt", i));
            assert!(f.dest.is_empty());
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

    #[test]
    fn test_artifact_config_context() {
        let variables: IndexMap<String, String> = indexmap! {
            "ARCHIVE_FILE".to_string() => "test-archive.zip".to_string(),
            "DIR1".to_string() => "dir1".to_string(),
            "DIR2".to_string() => "dir2".to_string(),
            "DIR3".to_string() => "dir3".to_string()
        };
        let ctx: Context = Context::new(&variables);
        let json_test_str = r#"
        {
            "type": "archive",
            "name": "${ARCHIVE_FILE}",
            "artifacts": [
                {
                    "type": "directory",
                    "name": "${DIR1}/test-dir",
                    "artifacts": [
                        {
                            "source": "${DIR2}/file2.txt"
                        },
                        {
                            "source": "${DIR3}/file3.txt"
                        }
                    ]
                }
            ]
        }
        "#;
        let mut config: ArtifactConfig = helper_artifact_config_from_str(json_test_str);
        config.expand_ctx(&ctx);
        assert_eq!(config.name, "test-archive.zip");
        assert!(!config.artifacts.is_empty());
        let artifacts: &Vec<ArtifactConfig> = &config.artifacts;
        assert_eq!(artifacts[0].atype, AType::Directory);
        assert_eq!(artifacts[0].name, "dir1/test-dir");
        assert!(!artifacts[0].artifacts.is_empty());
        let files: &Vec<ArtifactConfig> = &artifacts[0].artifacts;
        let mut i = 2;
        for f in files.iter() {
            assert_eq!(f.atype, AType::File);
            assert!(f.name.is_empty());
            assert_eq!(f.source, format!("dir{}/file{}.txt", i, i));
            assert!(f.dest.is_empty());
            i += 1;
        }
    }

    #[test]
    fn test_artifact_config_manifest() {
        let variables: IndexMap<String, String> = indexmap! {
            "MANIFEST_FILE".to_string() => "test-manifest.json".to_string(),
            "KEY_CONTEXT1".to_string() => "VAR1".to_string(),
            "KEY_CONTEXT2".to_string() => "VAR2".to_string(),
            "KEY_CONTEXT3".to_string() => "VAR3".to_string(),
            "KEY_CONTEXT4".to_string() => "VAR4".to_string()
        };
        let ctx: Context = Context::new(&variables);
        let json_test_str = r#"
        {
            "type": "manifest",
            "name": "${MANIFEST_FILE}",
            "content": {
                "${KEY_CONTEXT1}": "value1",
                "${KEY_CONTEXT2}": "value2",
                "${KEY_CONTEXT3}": "value3",
                "data": {
                    "${KEY_CONTEXT4}": "value4"
                }
            }
        }
        "#;
        let mut config: ArtifactConfig = helper_artifact_config_from_str(json_test_str);
        config.expand_ctx(&ctx);
        assert_eq!(config.name, "test-manifest.json");
        assert!(!config.manifest.is_empty());
        assert_eq!(config.manifest, "{\"VAR1\":\"value1\",\"VAR2\":\"value2\",\"VAR3\":\"value3\",\"data\":{\"VAR4\":\"value4\"}}");
    }

    #[test]
    fn test_artifact_config_manifest_empty_content() {
        let variables: IndexMap<String, String> = indexmap! {
            "MANIFEST_FILE".to_string() => "test-manifest.json".to_string()
        };
        let ctx: Context = Context::new(&variables);
        let json_test_str = r#"
        {
            "type": "manifest",
            "name": "${MANIFEST_FILE}"
        }
        "#;
        let mut config: ArtifactConfig = helper_artifact_config_from_str(json_test_str);
        config.expand_ctx(&ctx);
        assert_eq!(config.name, "test-manifest.json");
        assert!(!config.manifest.is_empty());
        assert_eq!(config.manifest, "{}");
    }
}