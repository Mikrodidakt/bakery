use crate::configs::Context;
use crate::error::BError;
use crate::fs::JsonFileReader;
use crate::data::{
    WsArtifactData,
    WsBuildData,
};

use std::path::PathBuf;
use serde_json::Value;

pub struct WsArtifactsHandler {
    data: WsArtifactData,
    children: Vec<WsArtifactsHandler>,
}

impl WsArtifactsHandler {
    pub fn from_str(json_config: &str, task_build_dir: &PathBuf, build_data: &WsBuildData) -> Result<Self, BError> {
        let data: Value = JsonFileReader::parse(json_config)?;
        Self::new(&data, task_build_dir, build_data)
    }

    pub fn new(data: &Value, task_build_dir: &PathBuf, build_data: &WsBuildData) -> Result<Self, BError> {
        let artifact_data: WsArtifactData = WsArtifactData::from_value(data)?;
        let children: Vec<WsArtifactsHandler> = build_data.get_artifacts(data, task_build_dir)?;
        Ok(WsArtifactsHandler {
            data: artifact_data,
            children,
        })
    }

    pub fn expand_ctx(&mut self, ctx: &Context) {
        self.data.expand_ctx(ctx);
        self.children.iter_mut().for_each(|artifact| {
            artifact.expand_ctx(ctx);
        });
    }

    pub fn data(&self) -> &WsArtifactData {
        &self.data
    }

    pub fn children(&self) -> &Vec<WsArtifactsHandler> {
        &self.children
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::workspace::WsArtifactsHandler;
    use crate::data::WsBuildData;
    use crate::helper::Helper;
    use crate::data::AType;

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
        assert_eq!(artifacts.data().atype(), &AType::File);
        assert_eq!(artifacts.data().source(), "test/file0-1.txt");
        assert!(artifacts.children().is_empty());
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
        assert_eq!(artifacts.data().atype(), &AType::File);
        assert_eq!(artifacts.data().source(), "test/file0-1.txt");
        assert_eq!(artifacts.data().dest(), "test/renamed-file0-1.txt");
        assert!(artifacts.children().is_empty());
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
        assert_eq!(artifacts.data().atype(), &AType::Directory);
        assert_eq!(artifacts.data().name(), "dir");
        assert!(!artifacts.children().is_empty());
        let dir_artifacts: &Vec<WsArtifactsHandler> = artifacts.children();
        assert_eq!(dir_artifacts.get(0).unwrap().data().atype(), &AType::File);
        assert_eq!(dir_artifacts.get(0).unwrap().data().dest(), "");
        assert_eq!(dir_artifacts.get(0).unwrap().data().source(), "file1.txt");
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
        assert_eq!(artifacts.data().atype(), &AType::Archive);
        assert_eq!(artifacts.data().name(), "test.zip");
        assert!(!artifacts.children().is_empty());
        let dir_artifacts: &Vec<WsArtifactsHandler> = artifacts.children();
        assert_eq!(dir_artifacts.get(0).unwrap().data().atype(), &AType::File);
        assert_eq!(dir_artifacts.get(0).unwrap().data().source(), "file3.txt");
        assert_eq!(dir_artifacts.get(0).unwrap().data().dest(), "file4.txt");
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
        assert_eq!(artifacts.data().name(), "test-manifest.json");
        assert!(!artifacts.data().manifest().is_empty());
        assert_eq!(artifacts.data().manifest(), "{\"VAR1\":\"value1\",\"VAR2\":\"value2\",\"VAR3\":\"value3\",\"data\":{\"VAR4\":\"value4\"}}");
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
        assert_eq!(artifacts.data().atype(), &AType::Archive);
        assert_eq!(artifacts.data().name(), "test.zip");
        assert!(!artifacts.children().is_empty());
        let archive_artifacts: &Vec<WsArtifactsHandler> = artifacts.children();
        assert_eq!(archive_artifacts.get(0).unwrap().data().atype(), &AType::File);
        assert_eq!(archive_artifacts.get(0).unwrap().data().source(), "file3.txt");
        assert_eq!(archive_artifacts.get(0).unwrap().data().dest(), "file4.txt");
        assert_eq!(archive_artifacts.get(1).unwrap().data().atype(), &AType::Directory);
        assert_eq!(archive_artifacts.get(1).unwrap().data().name(), "dir-name");
        let dir_artifacts: &Vec<WsArtifactsHandler> = archive_artifacts.get(1).unwrap().children();
        let mut i: usize = 1;
        dir_artifacts.iter().for_each(|f| {
            assert_eq!(f.data().atype(), &AType::File);
            assert_eq!(f.data().source(), &format!("file{}.txt", i));
            assert_eq!(f.data().dest(), "");
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
        let mut artifacts: WsArtifactsHandler = WsArtifactsHandler::from_str(
            json_artifacts_config,
            &task_build_dir,
            &build_data
        ).expect("Failed to parse config");
        artifacts.expand_ctx(build_data.context().ctx());
        assert_eq!(artifacts.data().atype(), &AType::Archive);
        assert_eq!(artifacts.data().name(), "test.zip");
        assert!(!artifacts.children().is_empty());
        let archive_artifacts: &Vec<WsArtifactsHandler> = artifacts.children();
        assert_eq!(archive_artifacts.get(0).unwrap().data().atype(), &AType::File);
        assert_eq!(archive_artifacts.get(0).unwrap().data().source(), "file3.txt");
        assert_eq!(archive_artifacts.get(0).unwrap().data().dest(), "file4.txt");
        assert_eq!(archive_artifacts.get(1).unwrap().data().name(), "test-manifest.json");
        assert!(!archive_artifacts.get(1).unwrap().data().manifest().is_empty());
        assert_eq!(archive_artifacts.get(1).unwrap().data().manifest(), "{\"VAR1\":\"value1\",\"VAR2\":\"value2\",\"VAR3\":\"value3\",\"data\":{\"VAR4\":\"value4\"}}");
        assert_eq!(archive_artifacts.get(2).unwrap().data().atype(), &AType::Directory);
        assert_eq!(archive_artifacts.get(2).unwrap().data().name(), "dir-name");
        let dir_artifacts: &Vec<WsArtifactsHandler> = archive_artifacts.get(2).unwrap().children();
        let mut i: usize = 1;
        dir_artifacts.iter().for_each(|f| {
            assert_eq!(f.data().atype(), &AType::File);
            assert_eq!(f.data().source(), &format!("file{}.txt", i));
            assert_eq!(f.data().dest(), "file-dest.txt");
            i += 1;
        });
    }
}