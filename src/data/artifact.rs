use serde_json::Value;

use crate::configs::Context;
use crate::error::BError;
use crate::configs::Config;

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub enum AType {
    File,
    Directory,
    Archive,
    Manifest,
    Link,
}

// TODO: we should consider using IndexSet instead of vector to make sure we
// keep the order from the json file
pub struct WsArtifactData {
    pub atype: AType, // Optional if not set for the task the default type 'file' is used
    pub name: String, // The name can be a name for a directory, archive, file or manifest
    pub source: String, // The source is only used if the type is file
    pub dest: String, // The dest is optional
    pub manifest: String, // The manifest content will be a json string that can be put in a file. The manifest can then be used by the CI to collect information from the build
}

impl Config for WsArtifactData {
}

impl WsArtifactData {
    pub fn from_str(json_string: &str) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data)
    }

    pub fn from_value(data: &Value) -> Result<Self, BError> {
        Self::new(data)
    }

    pub fn new(data: &Value) -> Result<Self, BError> {
        let ttype: String = Self::get_str_value("type", &data, Some(String::from("file")))?;
        let name: String = Self::get_str_value("name", &data, Some(String::from("")))?;
        let source: String = Self::get_str_value("source", &data, Some(String::from("")))?;
        let dest: String = Self::get_str_value("dest", &data, Some(String::from("")))?;
        let manifest: String = Self::get_str_manifest("content", &data, Some(String::from("{}")))?;

        if ttype != "file" && ttype != "directory" && ttype != "archive" && ttype != "manifest" && ttype != "link" {
            return Err(BError::ParseArtifactsError(format!("Invalid type '{}'", ttype)));
        }
        if ttype == "file" && source.is_empty() {
            return Err(BError::ParseArtifactsError(format!("The 'file' type requires a 'source'")));
        }
        if ttype == "directory" && name.is_empty() {
            return Err(BError::ParseArtifactsError(format!("The 'directory' type requires a 'name'")));
        }
        if ttype == "archive" && name.is_empty() {
            return Err(BError::ParseArtifactsError(format!("The 'archive' type requires a 'name'")));
        }
        if ttype == "manifest" && name.is_empty() {
            return Err(BError::ParseArtifactsError(format!("The 'manifest' type requires a 'name'")));
        }
        if ttype == "link" && (name.is_empty() || source.is_empty()) {
            return Err(BError::ParseArtifactsError(format!("The 'link' type requires a 'name' and 'source'")));
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
            "link" => {
                enum_ttype = AType::Link;
            },
            _ => {
                return Err(BError::ParseArtifactsError(format!("Invalid type '{}'", ttype)));
            },
        }

        //let source: PathBuf = task_build_dir.clone().join(PathBuf::from(source_str));
        //let dest: PathBuf = artifacts_dir.clone().join(PathBuf::from(dest_str));

        Ok(WsArtifactData {
            name,
            atype: enum_ttype,
            source,
            dest,
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
            },
            AType::Archive => {
                self.name = ctx.expand_str(&self.name);
            },
            AType::Manifest => {
                self.name = ctx.expand_str(&self.name);
                self.manifest = ctx.expand_str(&self.manifest);
            },
            AType::Link => {
                self.name = ctx.expand_str(&self.name);
                self.source = ctx.expand_str(&self.source);
            },
            _ => {
                panic!("Invalid 'artifact' format in build config. Invalid type '{:?}'", self.atype);
            },
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn atype(&self) -> &AType {
        &self.atype
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn dest(&self) -> &str {
        &self.dest
    }

    pub fn manifest(&self) -> &str {
        &self.manifest
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use indexmap::{IndexMap, indexmap};

    use crate::error::BError;
    use crate::helper::Helper;
    use crate::data::{
        WsArtifactData,
        AType,
    };
    use crate::configs::Context;

    #[test]
    fn test_ws_artifact_data_source() {
        let json_artifact_config: &str = r#"
        {
            "source": "file1.txt"
        }
        "#;
        let value: Value = Helper::parse(json_artifact_config).expect("Failed to parse artifact config");
        let data: WsArtifactData = WsArtifactData::new(&value).expect("Failed to parse artifact data");
        assert!(data.name().is_empty());
        assert_eq!(data.atype(), &AType::File);
        assert_eq!(data.source(), "file1.txt");
        assert_eq!(data.dest(), "");
    }

    #[test]
    fn test_ws_artifact_data_dest() {
        let json_artifact_config: &str = r#"
        {
            "source": "file1.txt",
            "dest": "dest"
        }
        "#;
        let value: Value = Helper::parse(json_artifact_config).expect("Failed to parse artifact config");
        let data: WsArtifactData = WsArtifactData::new(&value).expect("Failed to parse artifact data");
        assert!(data.name.is_empty());
        assert_eq!(data.atype(), &AType::File);
        assert_eq!(data.source(), "file1.txt");
        assert_eq!(data.dest(), "dest");
    }

    #[test]
    fn test_ws_artifact_data_file_type() {
        let json_artifact_config: &str  = r#"
        {
            "type": "file",
            "source": "file1.txt",
            "dest": "dest"
        }
        "#;
        let value: Value = Helper::parse(json_artifact_config).expect("Failed to parse artifact config");
        let data: WsArtifactData = WsArtifactData::new(&value).expect("Failed to parse artifact data");
        assert!(data.name().is_empty());
        assert_eq!(data.atype(), &AType::File);
        assert_eq!(data.source(), "file1.txt");
        assert_eq!(data.dest(), "dest");
    }

    #[test]
    fn test_ws_artifact_data_dir_type() {
        let json_artifact_config: &str = r#"
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
        let value: Value = Helper::parse(json_artifact_config).expect("Failed to parse artifact config");
        let data: WsArtifactData = WsArtifactData::new(&value).expect("Failed to parse artifact data");
        assert_eq!(data.atype(), &AType::Directory);
        assert_eq!(data.name(), "dir");
    }

    #[test]
    fn test_ws_artifact_data_error_invalid_type() {
        let json_artifact_config: &str = r#"
        {
            "type": "invalid",
            "source": "file1.txt",
            "dest": "dest"
        }
        "#;
        let value: Value = Helper::parse(json_artifact_config).expect("Failed to parse artifact config");
        let result: Result<WsArtifactData, BError> = WsArtifactData::new(&value);
        match result {
            Ok(_data) => {
                panic!("We should have recived an error because the type is invalid!");
            }
            Err(e) => {
                assert_eq!(e.to_string(), String::from("Invalid 'artifact' node in build config. Invalid type 'invalid'"));
            }
        }
    }

    #[test]
    fn test_ws_artifact_data_error_no_dir_name() {
        let json_artifact_config: &str = r#"
        {
            "type": "directory"
        }
        "#;
        let value: Value = Helper::parse(json_artifact_config).expect("Failed to parse artifact config");
        let result: Result<WsArtifactData, BError> = WsArtifactData::new(&value);
        match result {
            Ok(_rconfig) => {
                panic!("We should have recived an error because the type is invalid!");
            }
            Err(e) => {
                assert_eq!(e.to_string(), String::from("Invalid 'artifact' node in build config. The 'directory' type requires a 'name'"));
            }
        }
    }

    #[test]
    fn test_ws_artifact_data_error_no_manifest_name() {
        let json_artifact_config: &str = r#"
        {
            "type": "manifest"
        }
        "#;
        let value: Value = Helper::parse(json_artifact_config).expect("Failed to parse artifact config");
        let result: Result<WsArtifactData, BError> = WsArtifactData::new(&value);
        match result {
            Ok(_rconfig) => {
                panic!("We should have recived an error because the type is invalid!");
            }
            Err(e) => {
                assert_eq!(e.to_string(), String::from("Invalid 'artifact' node in build config. The 'manifest' type requires a 'name'"));
            }
        }
    }

    #[test]
    fn test_ws_artifact_data_context() {
        let ctx_variables: IndexMap<String, String> = indexmap! {
            "ARCHIVE_FILE".to_string() => "test-archive.zip".to_string(),
            "DIR1".to_string() => "dir1".to_string(),
            "DIR2".to_string() => "dir2".to_string(),
            "DIR3".to_string() => "dir3".to_string()
        };
        let json_artifact_config: &str = r#"
        {
            "type": "archive",
            "name": "$#[ARCHIVE_FILE]",
            "artifacts": [
                {
                    "type": "directory",
                    "name": "$#[DIR1]/test-dir",
                    "artifacts": [
                        {
                            "source": "$#[DIR2]/file2.txt"
                        },
                        {
                            "source": "$#[DIR3]/file3.txt"
                        }
                    ]
                }
            ]
        }
        "#;
        let context: Context = Context::new(&ctx_variables);
        let value: Value = Helper::parse(json_artifact_config).expect("Failed to parse artifact config");
        let mut data: WsArtifactData = WsArtifactData::new(&value).expect("Failed to parse artifact data");
        data.expand_ctx(&context);
        assert_eq!(data.name(), "test-archive.zip");
    }

    #[test]
    fn test_ws_artifact_data_manifest() {
        let ctx_variables: IndexMap<String, String> = indexmap! {
            "MANIFEST_FILE".to_string() => "test-manifest.json".to_string(),
            "KEY_CONTEXT1".to_string() => "VAR1".to_string(),
            "KEY_CONTEXT2".to_string() => "VAR2".to_string(),
            "KEY_CONTEXT3".to_string() => "VAR3".to_string(),
            "KEY_CONTEXT4".to_string() => "VAR4".to_string()
        };
        let json_artifact_config: &str = r#"
        {
            "type": "manifest",
            "name": "$#[MANIFEST_FILE]",
            "content": {
                "$#[KEY_CONTEXT1]": "value1",
                "$#[KEY_CONTEXT2]": "value2",
                "$#[KEY_CONTEXT3]": "value3",
                "data": {
                    "$#[KEY_CONTEXT4]": "value4"
                }
            }
        }
        "#;
        let context: Context = Context::new(&ctx_variables);
        let value: Value = Helper::parse(json_artifact_config).expect("Failed to parse artifact config");
        let mut data: WsArtifactData = WsArtifactData::new(&value).expect("Failed to parse artifact data");
        data.expand_ctx(&context);
        assert_eq!(data.name(), "test-manifest.json");
        assert!(!data.manifest().is_empty());
        assert_eq!(data.manifest(), "{\"VAR1\":\"value1\",\"VAR2\":\"value2\",\"VAR3\":\"value3\",\"data\":{\"VAR4\":\"value4\"}}");
    }

    #[test]
    fn test_ws_artifact_data_manifest_empty_content() {
        let ctx_variables: IndexMap<String, String> = indexmap! {
            "MANIFEST_FILE".to_string() => "test-manifest.json".to_string()
        };
        let json_artifact_config: &str = r#"
        {
            "type": "manifest",
            "name": "$#[MANIFEST_FILE]"
        }
        "#;
        let context: Context = Context::new(&ctx_variables);
        let value: Value = Helper::parse(json_artifact_config).expect("Failed to parse artifact config");
        let mut data: WsArtifactData = WsArtifactData::new(&value).expect("Failed to parse artifact data");
        data.expand_ctx(&context);
        assert_eq!(data.name(), "test-manifest.json");
        assert!(!data.manifest().is_empty());
        assert_eq!(data.manifest(), "{}");
    }

    #[test]
    fn test_ws_artifact_data_link_type() {
        let json_artifact_config: &str = r#"
        {
            "type": "link",
            "name": "link.txt",
            "source": "file.txt"
        }
        "#;
        let value: Value = Helper::parse(json_artifact_config).expect("Failed to parse artifact config");
        let data: WsArtifactData = WsArtifactData::new(&value).expect("Failed to parse artifact data");
        assert!(data.dest().is_empty());
        assert_eq!(data.atype(), &AType::Link);
        assert_eq!(data.source(), "file.txt");
        assert_eq!(data.name(), "link.txt");
    }

    #[test]
    fn test_ws_artifact_data_link_ctx() {
        let ctx_variables: IndexMap<String, String> = indexmap! {
            "LINK_NAME".to_string() => "ctx-link.txt".to_string()
        };
        let json_artifact_config: &str = r#"
        {
            "type": "link",
            "name": "$#[LINK_NAME]",
            "source": "file.txt"
        }
        "#;
        let context: Context = Context::new(&ctx_variables);
        let value: Value = Helper::parse(json_artifact_config).expect("Failed to parse artifact config");
        let mut data: WsArtifactData = WsArtifactData::new(&value).expect("Failed to parse artifact data");
        data.expand_ctx(&context);
        assert!(data.dest().is_empty());
        assert_eq!(data.atype(), &AType::Link);
        assert_eq!(data.source(), "file.txt");
        assert_eq!(data.name(), "ctx-link.txt");
    }

    #[test]
    fn test_ws_artifact_data_error_no_link_name() {
        let json_artifact_config: &str = r#"
        {
            "type": "link"
        }
        "#;
        let value: Value = Helper::parse(json_artifact_config).expect("Failed to parse artifact config");
        let result: Result<WsArtifactData, BError> = WsArtifactData::new(&value);
        match result {
            Ok(_rconfig) => {
                panic!("We should have recived an error because the type is invalid!");
            }
            Err(e) => {
                assert_eq!(e.to_string(), String::from("Invalid 'artifact' node in build config. The 'link' type requires a 'name' and 'source'"));
            }
        }
    }

    #[test]
    fn test_ws_artifact_data_error_no_link_source() {
        let json_artifact_config: &str = r#"
        {
            "type": "link",
            "name": "link.txt"
        }
        "#;
        let value: Value = Helper::parse(json_artifact_config).expect("Failed to parse artifact config");
        let result: Result<WsArtifactData, BError> = WsArtifactData::new(&value);
        match result {
            Ok(_rconfig) => {
                panic!("We should have recived an error because the type is invalid!");
            }
            Err(e) => {
                assert_eq!(e.to_string(), String::from("Invalid 'artifact' node in build config. The 'link' type requires a 'name' and 'source'"));
            }
        }
    }
}
