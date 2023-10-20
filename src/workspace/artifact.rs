use crate::configs::{AType, ArtifactConfig, Context, task};
use crate::workspace::{WsSettingsHandler, WsBuildData};
use crate::error::BError;
use crate::fs::JsonFileReader;

use std::path::PathBuf;
use serde_json::Value;

pub struct WsArtifactsHandler {
    config: ArtifactConfig,
    build_dir: PathBuf,
    artifacts_dir: PathBuf,
    artifacts: Vec<WsArtifactsHandler>,
}

impl WsArtifactsHandler {
    pub fn from_str(json_config: &str, task_build_dir: &PathBuf, build_data: &WsBuildData) -> Result<Self, BError> {
        let data: Value = JsonFileReader::parse(json_config)?;
        Self::new(&data, task_build_dir, build_data)
    }

    pub fn new(data: &Value, task_build_dir: &PathBuf, build_data: &WsBuildData) -> Result<Self, BError> {
        let mut config: ArtifactConfig = ArtifactConfig::from_value(data)?;
        config.expand_ctx(build_data.context().ctx());
        let artifacts: Vec<WsArtifactsHandler> = build_data.get_artifacts(data, task_build_dir)?;
        Ok(WsArtifactsHandler {
            config,
            build_dir: task_build_dir.to_owned(),
            artifacts_dir: build_data.settings().artifacts_dir(),
            artifacts,
        })
    }

    pub fn expand_ctx(&mut self, ctx: &Context) {
        self.config.expand_ctx(ctx);
        self.build_dir = ctx.expand_path(&self.build_dir);
        self.artifacts.iter_mut().for_each(|artifact| {
            artifact.expand_ctx(ctx);
        });
    }

    pub fn name(&self) -> &str {
        &self.config.name
    }

    pub fn atype(&self) -> &AType {
        &self.config.atype
    }

    pub fn source(&self) -> PathBuf {
        let mut path_buf: PathBuf = self.build_dir.clone();
        path_buf.join(&self.config.source)
    }

    pub fn dest(&self) -> PathBuf {
        let mut path_buf: PathBuf = self.artifacts_dir.clone();
        path_buf.join(&self.config.dest)
    }

    pub fn manifest(&self) -> &str {
        &self.config.manifest
    }

    pub fn artifacts(&self) -> &Vec<WsArtifactsHandler> {
        &self.artifacts
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use indexmap::{IndexMap, indexmap};
    use crate::workspace::{WsSettingsHandler, WsArtifactsHandler, WsBuildData};
    use crate::configs::AType;
    use crate::helper::Helper;

    #[test]
    fn test_ws_artifacts_file_source() {
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let json_artifacts_config: &str = r#"
        {
            "source": "test/file0-1.txt"
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = WsArtifactsHandler::from_str(
            json_artifacts_config,
            &task_build_dir,
            &build_data
        ).expect("Failed to parse config");
        assert_eq!(artifacts.atype(), &AType::File);
        assert_eq!(artifacts.source(), PathBuf::from("/workspace/task/dir/test/file0-1.txt"));
        assert!(artifacts.artifacts().is_empty());
    }

    #[test]
    fn test_ws_artifacts_file_dest() {
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let json_artifacts_config: &str = r#"
        {
            "source": "test/file0-1.txt",
            "dest": "test/renamed-file0-1.txt"
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = WsArtifactsHandler::from_str(
            json_artifacts_config,
            &task_build_dir,
            &build_data
        ).expect("Failed to parse config");
        assert_eq!(artifacts.atype(), &AType::File);
        assert_eq!(artifacts.source(), PathBuf::from("/workspace/task/dir/test/file0-1.txt"));
        assert_eq!(artifacts.dest(), PathBuf::from("/workspace/artifacts/test/renamed-file0-1.txt"));
        assert!(artifacts.artifacts().is_empty());
    }

    #[test]
    fn test_artifacts_dir_type() {
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let json_artifacts_config: &str = r#"
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
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = WsArtifactsHandler::from_str(
            json_artifacts_config,
            &task_build_dir,
            &build_data
        ).expect("Failed to parse config");
        assert_eq!(artifacts.atype(), &AType::Directory);
        assert_eq!(artifacts.name(), "dir");
        assert!(!artifacts.artifacts().is_empty());
        let dir_artifacts: &Vec<WsArtifactsHandler> = artifacts.artifacts();
        assert_eq!(dir_artifacts.get(0).unwrap().atype(), &AType::File);
        assert_eq!(dir_artifacts.get(0).unwrap().dest(), PathBuf::from("/workspace/artifacts/"));
        assert_eq!(dir_artifacts.get(0).unwrap().source(), PathBuf::from("/workspace/task/dir/file1.txt"));
    }

    #[test]
    fn test_artifact_archive() {
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let json_artifacts_config: &str = r#"
        {
            "type": "archive",
            "name": "test.zip",
            "artifacts": [
                {
                    "source": "file3.txt",
                    "dest": "file4.txt"
                }
            ]
        }
        "#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir,None, None);
        let artifacts: WsArtifactsHandler = WsArtifactsHandler::from_str(
            json_artifacts_config,
            &task_build_dir,
            &build_data
        ).expect("Failed to parse config");
        assert_eq!(artifacts.atype(), &AType::Archive);
        assert_eq!(artifacts.name(), "test.zip");
        assert!(!artifacts.artifacts().is_empty());
        let dir_artifacts: &Vec<WsArtifactsHandler> = artifacts.artifacts();
        assert_eq!(dir_artifacts.get(0).unwrap().atype(), &AType::File);
        assert_eq!(dir_artifacts.get(0).unwrap().source(), PathBuf::from("/workspace/task/dir/file3.txt"));
        assert_eq!(dir_artifacts.get(0).unwrap().dest(), PathBuf::from("/workspace/artifacts/file4.txt"));
    }

    #[test]
    fn test_artifact_manifest() {
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let json_artifacts_config: &str = r#"
        {
            "type": "manifest",
            "name": "test-manifest.json",
            "content": {
                "VAR1": "value1",
                "VAR2": "value2",
                "VAR3": "value3",
                "data": {
                    "VAR4": "value4"
                }
            }
        }
        "#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = WsArtifactsHandler::from_str(
            json_artifacts_config,
            &task_build_dir,
            &build_data
        ).expect("Failed to parse config");
        assert_eq!(artifacts.name(), "test-manifest.json");
        assert!(!artifacts.manifest().is_empty());
        assert_eq!(artifacts.manifest(), "{\"VAR1\":\"value1\",\"VAR2\":\"value2\",\"VAR3\":\"value3\",\"data\":{\"VAR4\":\"value4\"}}");
    }

    #[test]
    fn test_artifact_composite() {
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let json_artifacts_config: &str = r#"
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
        "#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let artifacts: WsArtifactsHandler = WsArtifactsHandler::from_str(
            json_artifacts_config,
            &task_build_dir,
            &build_data
        ).expect("Failed to parse config");
        assert_eq!(artifacts.atype(), &AType::Archive);
        assert_eq!(artifacts.name(), "test.zip");
        assert!(!artifacts.artifacts().is_empty());
        let archive_artifacts: &Vec<WsArtifactsHandler> = artifacts.artifacts();
        assert_eq!(archive_artifacts.get(0).unwrap().atype(), &AType::File);
        assert_eq!(archive_artifacts.get(0).unwrap().source(), PathBuf::from("/workspace/task/dir/file3.txt"));
        assert_eq!(archive_artifacts.get(0).unwrap().dest(), PathBuf::from("/workspace/artifacts/file4.txt"));
        assert_eq!(archive_artifacts.get(1).unwrap().atype(), &AType::Directory);
        assert_eq!(archive_artifacts.get(1).unwrap().name(), "dir-name");
        let dir_artifacts: &Vec<WsArtifactsHandler> = archive_artifacts.get(1).unwrap().artifacts();
        let mut i: usize = 1;
        dir_artifacts.iter().for_each(|f| {
            assert_eq!(f.atype(), &AType::File);
            assert_eq!(f.source(), PathBuf::from(format!("/workspace/task/dir/file{}.txt", i)));
            assert_eq!(f.dest(), PathBuf::from("/workspace/artifacts/"));
            i += 1;
        });
    }

    #[test]
    fn test_artifact_expand_ctx() {
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let json_build_config: &str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "context": [
                "ARCHIVE_NAME=test.zip",
                "DIR_NAME=dir-name",
                "FILE_NAME=file2.txt",
                "DEST_NAME=file-dest.txt",
                "DEST_FILE_NAME=${DEST_NAME}",
                "MANIFEST_FILE=test-manifest.json",
                "KEY_CONTEXT1=VAR1",
                "KEY_CONTEXT2=VAR2",
                "KEY_CONTEXT3=VAR3",
                "KEY_CONTEXT4=VAR4"
            ]
        }"#;
        let json_artifacts_config: &str = r#"
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
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let artifacts: WsArtifactsHandler = WsArtifactsHandler::from_str(
            json_artifacts_config,
            &task_build_dir,
            &build_data
        ).expect("Failed to parse config");
        assert_eq!(artifacts.atype(), &AType::Archive);
        assert_eq!(artifacts.name(), "test.zip");
        assert!(!artifacts.artifacts().is_empty());
        let archive_artifacts: &Vec<WsArtifactsHandler> = artifacts.artifacts();
        assert_eq!(archive_artifacts.get(0).unwrap().atype(), &AType::File);
        assert_eq!(archive_artifacts.get(0).unwrap().source(), PathBuf::from("/workspace/task/dir/file3.txt"));
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
            assert_eq!(f.source(), PathBuf::from(format!("/workspace/task/dir/file{}.txt", i)));
            assert_eq!(f.dest(), PathBuf::from("/workspace/artifacts/file-dest.txt"));
            i += 1;
        });
    }
}