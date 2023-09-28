use crate::configs::{TType, TaskConfig, Context, task};
use crate::workspace::{WsArtifactsHandler, WsSettingsHandler, WsBuildData};
use crate::error::BError;
use crate::fs::JsonFileReader;

use std::path::PathBuf;
use serde_json::Value;
use indexmap::IndexMap;

use super::WsBuildConfigHandler;

pub struct WsTaskHandler {
    name: String,
    config: TaskConfig,
    build_dir: PathBuf,
    work_dir: PathBuf,
    artifacts_dir: PathBuf,
    artifacts: Vec<WsArtifactsHandler>,
}

impl WsTaskHandler {
    fn determine_build_dir(config: &TaskConfig, data: &WsBuildData) -> PathBuf {
        if config.ttype == TType::Bitbake {
            let task_build_dir: &str = &config.builddir;
            if task_build_dir.is_empty() {
                return data.bb_build_dir()
            }
        }

        data.settings().work_dir().join(PathBuf::from(&config.builddir))
    }

    pub fn from_str(json_config: &str, build_data: &WsBuildData) -> Result<Self, BError> {
        let data: Value = JsonFileReader::parse(json_config)?;
        Self::new(&data, build_data)
    }

    pub fn new(data: &Value, build_data: &WsBuildData) -> Result<Self, BError> {
        let mut config: TaskConfig = TaskConfig::from_value(data)?;
        config.expand_ctx(build_data.context());
        let task_build_dir: PathBuf = Self::determine_build_dir(&config, build_data);
        let artifacts: Vec<WsArtifactsHandler> = build_data.get_artifacts(data, &task_build_dir)?;
        Ok(WsTaskHandler {
            name: config.name.to_string(),
            config,
            work_dir: build_data.settings().work_dir(),
            build_dir: task_build_dir,
            artifacts_dir: build_data.settings().artifacts_dir(),
            artifacts,
        })
    }

    pub fn expand_ctx(&mut self, ctx: &Context) {
        self.config.expand_ctx(ctx);
        self.build_dir = ctx.expand_path(&self.build_dir);
    }

    pub fn build_dir(&self) -> PathBuf {
        self.build_dir.clone()
    }

    pub fn artifacts_dir(&self) -> PathBuf {
        self.artifacts_dir.clone()
    }

    pub fn ttype(&self) -> &TType {
        &self.config.ttype
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn build_cmd(&self) -> &str {
        &self.config.build
    }

    pub fn clean_cmd(&self) -> &str {
        &self.config.clean
    }

    pub fn docker(&self) -> &str {
        &self.config.docker
    }

    pub fn disabled(&self) -> bool {
        if self.config.disabled == "true" {
            return true;
        }
        return false;
    }

    pub fn recipes(&self) -> &Vec<String> {
        &self.config.recipes
    }

    pub fn condition(&self) -> bool {
        let condition: &str = &self.config.condition;

        if condition.is_empty() {
            return true;
        }

        match condition {
            "1" | "yes" | "y" | "Y" | "true" | "YES" | "TRUE" | "True" | "Yes" => return true,
            _ => return false,
        }
    }
    
    pub fn artifacts(&self) -> &Vec<WsArtifactsHandler> {
        &self.artifacts
    }
}

#[cfg(test)]
mod tests {
    use std::path::{PathBuf};
    use indexmap::{IndexMap, indexmap};

    use crate::commands::build;
    use crate::helper::Helper;
    use crate::workspace::{WsTaskHandler, WsSettingsHandler, WsArtifactsHandler, WsBuildData};
    use crate::configs::{TType, AType, Context};

    #[test]
    fn test_ws_task_nonbitbake() {
        let json_task_str: &str = r#"
        { 
            "index": "0",
            "name": "task-name",
            "type": "non-bitbake",
            "disabled": "false",
            "condition": "true",
            "builddir": "task/dir",
            "build": "build-cmd",
            "clean": "clean-cmd",
            "artifacts": []
        }"#;
        let default_settings: &str  = r#"
        {
            "version": "4"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let build_data: WsBuildData = WsBuildData::new("", "tmp/deploy/", IndexMap::new(), &ws_settings).expect("Failed to setup build data");
        let task: WsTaskHandler = WsTaskHandler::from_str(json_task_str, &build_data).expect("Failed to parse Task config");
        assert_eq!(task.build_dir(), PathBuf::from("/workspace/task/dir"));
        assert!(task.condition());
        assert_eq!(task.name(), "task-name");
        assert_eq!(task.build_cmd(), "build-cmd");
        assert_eq!(task.clean_cmd(), "clean-cmd");
        assert_eq!(task.ttype(), &TType::NonBitbake);
        assert!(!task.disabled());
    }

    #[test]
    fn test_ws_task_bitbake() {
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "bitbake",
            "recipes": [
                "test-image"
            ]
        }"#;
        let default_settings: &str  = r#"
        {
            "version": "4"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let build_data: WsBuildData = WsBuildData::new("", "tmp/deploy/", IndexMap::new(), &ws_settings).expect("Failed to setup build data");
        let task: WsTaskHandler = WsTaskHandler::from_str(json_task_str, &build_data).expect("Failed to parse Task config");
        assert_eq!(task.build_dir(), PathBuf::from("/workspace/builds"));
        assert!(task.condition());
        assert_eq!(task.name(), "task-name");
        assert_eq!(task.ttype(), &TType::Bitbake);
        assert_eq!(task.recipes(), &vec!["test-image".to_string()]);
        assert!(!task.disabled());
    }

    #[test]
    fn test_ws_task_bb_build_dir() {
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "bitbake",
            "recipes": [
                "test-image"
            ]
        }"#;
        let default_settings: &str  = r#"
        {
            "version": "4"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let build_data: WsBuildData = WsBuildData::new("", "tmp/deploy/", IndexMap::new(), &ws_settings).expect("Failed to setup build data");
        let task: WsTaskHandler = WsTaskHandler::from_str(json_task_str, &build_data).expect("Failed to parse Task config");
        assert_eq!(task.build_dir(), PathBuf::from("/workspace/builds"));
        assert!(task.condition());
        assert_eq!(task.name(), "task-name");
        assert_eq!(task.ttype(), &TType::Bitbake);
        assert_eq!(task.recipes(), &vec!["test-image".to_string()]);
        assert!(!task.disabled());
    }

    #[test]
    fn test_ws_task_nonbitbake_artifacts() {
        let default_settings: &str  = r#"
        {
            "version": "4"
        }"#;
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "non-bitbake",
            "builddir": "task/build/dir",
            "build": "build-cmd",
            "clean": "clean-cmd",
            "artifacts": [
                {
                    "type": "archive",
                    "name": "test.zip",
                    "artifacts": [
                        {
                            "source": "file3.txt",
                            "dest": "file4.txt"
                        },
                        {
                            "type": "directory",
                            "name": "dir-name",
                            "artifacts": [
                                {
                                    "source": "file1.txt"
                                },
                                {
                                    "source": "file2.txt"
                                },
                                {
                                    "source": "file3.txt"
                                }
                            ]
                        }
                    ]
                }
            ]
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let build_data: WsBuildData = WsBuildData::new("", "tmp/deploy/", IndexMap::new(), &ws_settings).expect("Failed to setup build data");
        let task: WsTaskHandler = WsTaskHandler::from_str(json_task_str, &build_data).expect("Failed to parse Task config");
        assert_eq!(task.build_dir(), PathBuf::from("/workspace/task/build/dir"));
        assert!(task.condition());
        assert_eq!(task.name(), "task-name");
        assert_eq!(task.ttype(), &TType::NonBitbake);
        assert_eq!(task.build_cmd(), "build-cmd");
        assert_eq!(task.clean_cmd(), "clean-cmd");
        assert!(!task.disabled());
        let artifacts: &WsArtifactsHandler = task.artifacts().first().unwrap();
        assert_eq!(artifacts.atype(), &AType::Archive);
        assert_eq!(artifacts.name(), "test.zip");
        assert!(!artifacts.artifacts().is_empty());
        let archive_artifacts: &Vec<WsArtifactsHandler> = artifacts.artifacts();
        assert_eq!(archive_artifacts.get(0).unwrap().atype(), &AType::File);
        assert_eq!(archive_artifacts.get(0).unwrap().source(), PathBuf::from("/workspace/task/build/dir/file3.txt"));
        assert_eq!(archive_artifacts.get(0).unwrap().dest(), PathBuf::from("/workspace/artifacts/file4.txt"));
        assert_eq!(archive_artifacts.get(1).unwrap().atype(), &AType::Directory);
        assert_eq!(archive_artifacts.get(1).unwrap().name(), "dir-name");
        let dir_artifacts: &Vec<WsArtifactsHandler> = archive_artifacts.get(1).unwrap().artifacts();
        let mut i: usize = 1;
        dir_artifacts.iter().for_each(|f| {
            assert_eq!(f.atype(), &AType::File);
            assert_eq!(f.source(), PathBuf::from(format!("/workspace/task/build/dir/file{}.txt", i)));
            assert_eq!(f.dest(), PathBuf::from("/workspace/artifacts/"));
            i += 1;
        });
    }

    #[test]
    fn test_ws_task_expand_ctx() {
        let variables: IndexMap<String, String> = indexmap! {
            "ARCHIVE_NAME".to_string() => "test.zip".to_string(),
            "DIR_NAME".to_string() => "dir-name".to_string(),
            "FILE_NAME".to_string() => "file2.txt".to_string(),
            "BITBAKE_IMAGE".to_string() => "test-image".to_string(),
            "DEST_NAME".to_string() => "file-dest.txt".to_string(),
            "DEST_FILE_NAME".to_string() => "${DEST_NAME}".to_string(),
            "MANIFEST_FILE".to_string() => "test-manifest.json".to_string(),
            "KEY_CONTEXT1".to_string() => "VAR1".to_string(),
            "KEY_CONTEXT2".to_string() => "VAR2".to_string(),
            "KEY_CONTEXT3".to_string() => "VAR3".to_string(),
            "KEY_CONTEXT4".to_string() => "VAR4".to_string()
        };
        let ctx: Context = Context::new(&variables);
        let default_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "bitbake",
            "recipes": [
                "${BITBAKE_IMAGE}"
            ],
            "artifacts": [
                {
                    "type": "archive",
                    "name": "${ARCHIVE_NAME}",
                    "artifacts": [
                        {
                            "source": "file3.txt",
                            "dest": "file4.txt"
                        },
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
                        },
                        {
                            "type": "directory",
                            "name": "${DIR_NAME}",
                            "artifacts": [
                                {
                                    "source": "file1.txt",
                                    "dest": "${DEST_NAME}"
                                },
                                {
                                    "source": "${FILE_NAME}",
                                    "dest": "${DEST_NAME}"
                                },
                                {
                                    "source": "file3.txt",
                                    "dest": "${DEST_NAME}"
                                }
                            ]
                        }
                    ]
                }
            ]
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let build_data: WsBuildData = WsBuildData::new("", "tmp/deploy/", variables, &ws_settings).expect("Failed to setup build data");
        let mut task: WsTaskHandler = WsTaskHandler::from_str(json_task_str, &build_data).expect("Failed to parse Task config");
        task.expand_ctx(build_data.context());
        assert_eq!(task.build_dir(), PathBuf::from("/workspace/builds"));
        assert!(task.condition());
        assert_eq!(task.name(), "task-name");
        assert_eq!(task.ttype(), &TType::Bitbake);
        assert_eq!(task.recipes(), &vec!["test-image".to_string()]);
        assert!(!task.disabled());
        let artifacts: &WsArtifactsHandler = task.artifacts().first().unwrap();
        assert_eq!(artifacts.atype(), &AType::Archive);
        assert_eq!(artifacts.name(), "test.zip");
        assert!(!artifacts.artifacts().is_empty());
        let archive_artifacts: &Vec<WsArtifactsHandler> = artifacts.artifacts();
        assert_eq!(archive_artifacts.get(0).unwrap().atype(), &AType::File);
        assert_eq!(archive_artifacts.get(0).unwrap().source(), PathBuf::from("/workspace/builds/file3.txt"));
        assert_eq!(archive_artifacts.get(0).unwrap().dest(), PathBuf::from("/workspace/artifacts/file4.txt"));
        assert_eq!(archive_artifacts.get(1).unwrap().name(), "test-manifest.json");
        assert!(!archive_artifacts.get(1).unwrap().manifest().is_empty());
        assert_eq!(archive_artifacts.get(1).unwrap().manifest(), "{\"VAR1\":\"value1\",\"VAR2\":\"value2\",\"VAR3\":\"value3\",\"data\":{\"VAR4\":\"value4\"}}");
        assert_eq!(archive_artifacts.get(2).unwrap().atype(), &AType::Directory);
        assert_eq!(archive_artifacts.get(2).unwrap().name(), "dir-name");
        let dir_artifacts: &Vec<WsArtifactsHandler> = archive_artifacts.get(2).unwrap().artifacts();
        let mut i: usize = 1;
        dir_artifacts.iter().for_each(|f| {
            assert_eq!(f.atype(), &AType::File);
            assert_eq!(f.source(), PathBuf::from(format!("/workspace/builds/file{}.txt", i)));
            assert_eq!(f.dest(), PathBuf::from("/workspace/artifacts/file-dest.txt"));
            i += 1;
        });
    }
}
