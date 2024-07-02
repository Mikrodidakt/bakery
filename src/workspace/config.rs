use indexmap::IndexMap;
use serde_json::Value;

use crate::workspace::{
    WsSettingsHandler,
    WsTaskHandler,
    WsDeployHandler,
    WsUploadHandler,
    WsTaskCmdHandler,
};
use crate::data::{WsBuildData, WsContextData};
use crate::error::BError;
use crate::fs::JsonFileReader;
use crate::configs::Context;

pub struct WsBuildConfigHandler {
    data: WsBuildData,
    tasks: IndexMap<String, WsTaskHandler>,
    deploy: WsDeployHandler,
    upload: WsUploadHandler,
    setup: WsTaskCmdHandler,
}

impl WsBuildConfigHandler {
    pub fn from_str(json_config: &str, settings: &WsSettingsHandler) -> Result<Self, BError> {
        let data: Value = JsonFileReader::parse(json_config)?;
        Self::new(&data, settings)
    }

    pub fn new(data: &Value, settings: &WsSettingsHandler) -> Result<Self, BError> {
        let build_data: WsBuildData = WsBuildData::new(data, settings)?;
        let tasks: IndexMap<String, WsTaskHandler> = build_data.get_tasks(data)?;
        let deploy: WsDeployHandler = WsDeployHandler::new(data)?;
        let upload: WsUploadHandler = WsUploadHandler::new(data)?;
        let setup: WsTaskCmdHandler = WsTaskCmdHandler::new("setup", data)?;

        if build_data.version() != "5" {
            return Err(BError::InvalidBuildConfigError(build_data.version().to_string()));
        }

        Ok(WsBuildConfigHandler {
            data: build_data,
            deploy,
            upload,
            setup,
            tasks,
        })
    }

    pub fn build_data(&self) -> &WsBuildData {
        &self.data
    }

    pub fn update_ctx(&mut self, context: &Context) {
        self.data.update_ctx(context);
    }

    pub fn expand_ctx(&mut self) -> Result<(), BError> {
        self.data.expand_ctx()?;
        for (_name, task) in self.tasks.iter_mut() {
            task.expand_ctx(self.data.context().ctx())?;
        }
        self.deploy.expand_ctx(self.data.context().ctx())?;
        self.upload.expand_ctx(self.data.context().ctx())?;
        Ok(())
    }

    pub fn ctx(&self) -> Result<IndexMap<String, String>, BError> {
        Ok(self.data.context().ctx().variables().clone())
    }

    pub fn task(&self, task: &str) -> Result<&WsTaskHandler, BError> {
        match self.tasks.get(task) {
            Some(config) => {
                return Ok(config);
            },
            None => {
                return Err(BError::ValueError(format!("Task '{}' does not exists in build config", task)));
            }
        }
    }

    pub fn tasks(&self) -> &IndexMap<String, WsTaskHandler> {
        &self.tasks
    }

    pub fn deploy(&self) -> &WsDeployHandler {
        &self.deploy
    }

    pub fn upload(&self) -> &WsUploadHandler {
        &self.upload
    }

    pub fn setup(&self) -> &WsTaskCmdHandler {
        &self.setup
    }

    pub fn description(&self) -> &str {
        &self.data.product().description()
    }

    //pub fn config_enabled(&self) -> bool {
    //    self.config.enabled()
    //}
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::workspace::{WsSettingsHandler, WsBuildConfigHandler, WsTaskHandler};
    use crate::error::BError;

    #[test]
    fn test_ws_config_default() {
        let json_settings = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {
            "version": "5",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).unwrap();
        let ws_config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut ws_settings).expect("Failed to parse build config");
        assert_eq!(ws_config.build_data().version(), "5".to_string());
        assert_eq!(ws_config.build_data().name(), "test-name".to_string());
        assert_eq!(ws_config.build_data().product().name(), "test-name".to_string());
        assert_eq!(ws_config.build_data().product().arch(), "test-arch".to_string());
        assert_eq!(ws_config.build_data().product().description(), "Test Description".to_string());
        assert_eq!(ws_config.build_data().bitbake().distro(), "NA".to_string());
        assert_eq!(ws_config.build_data().bitbake().machine(), "NA".to_string());
        assert_eq!(ws_config.build_data().bitbake().build_dir(), PathBuf::from("/workspace/builds/test-name"));
        assert_eq!(ws_config.build_data().bitbake().build_config_dir(), PathBuf::from("/workspace/builds/test-name/conf"));
        assert_eq!(ws_config.build_data().bitbake().deploy_dir(), PathBuf::from("/workspace/builds/test-name/tmp/deploy/images"));
        assert_eq!(ws_config.build_data().bitbake().dl_dir(), PathBuf::from("/workspace/.cache/download"));
        assert_eq!(ws_config.build_data().bitbake().sstate_dir(), PathBuf::from("/workspace/.cache/test-arch/sstate-cache"));
        assert_eq!(ws_config.build_data().bitbake().bblayers_conf_path(), PathBuf::from("/workspace/builds/test-name/conf/bblayers.conf"));
        assert!(ws_config.build_data().bitbake().bblayers_conf().is_empty());
        assert_eq!(ws_config.build_data().bitbake().local_conf_path(), PathBuf::from("/workspace/builds/test-name/conf/local.conf"));
        assert!(!ws_config.build_data().bitbake().local_conf().is_empty());
        let mut conf_str: String = String::new();
        conf_str.push_str(&format!("MACHINE ?= \"{}\"\n", ws_config.build_data().bitbake().machine()));
        conf_str.push_str(&format!("PRODUCT_NAME ?= \"{}\"\n", ws_config.build_data().product().name()));
        conf_str.push_str(&format!("DISTRO ?= \"{}\"\n", ws_config.build_data().bitbake().distro()));
        conf_str.push_str(&format!("SSTATE_DIR ?= \"{}\"\n", ws_config.build_data().bitbake().sstate_dir().to_str().unwrap()));
        conf_str.push_str(&format!("DL_DIR ?= \"{}\"\n", ws_config.build_data().bitbake().dl_dir().to_str().unwrap()));
        assert_eq!(ws_config.build_data().bitbake().local_conf(), conf_str);
        assert_eq!(ws_config.build_data().bitbake().docker_image(), "NA".to_string());

    }

    #[test]
    fn test_ws_config_context_docker() {
        let json_settings = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {
            "version": "5",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "context": [
                "DOCKER_REGISTRY=test-registry",
                "DOCKER_TAG=0.1",
                "DOCKER_IMAGE=test-image"
            ],
            "bb": {
                "docker": "$#[DOCKER_REGISTRY]/$#[DOCKER_IMAGE]:$#[DOCKER_TAG]"
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).unwrap();
        let mut ws_config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut ws_settings).expect("Failed to parse build config");
        ws_config.expand_ctx().unwrap();
        assert_eq!(ws_config.build_data().bitbake().docker_image(), "test-registry/test-image:0.1");
    }

    #[test]
    fn test_ws_config_empty_tasks() {
        let json_settings = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {
            "version": "5",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).unwrap();
        let ws_config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut ws_settings).expect("Failed to parse build config");
        assert!(ws_config.tasks().is_empty());
    }

    #[test]
    fn test_ws_task_config_condition() {
        let json_settings = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {
            "version": "5",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "context": [
                "TASK1_CONDITION=1",
                "TASK2_CONDITION=y",
                "TASK3_CONDITION=Y",
                "TASK4_CONDITION=yes",
                "TASK5_CONDITION=Yes",
                "TASK6_CONDITION=YES",
                "TASK7_CONDITION=True",
                "TASK8_CONDITION=TRUE",
                "TASK9_CONDITION=true"
            ],
            "tasks": {
                "task1": {
                    "index": "1",
                    "name": "task1",
                    "type": "non-bitbake",
                    "condition": "$#[TASK1_CONDITION]"
                },
                "task2": {
                    "index": "2",
                    "name": "task2",
                    "type": "non-bitbake",
                    "condition": "$#[TASK2_CONDITION]"
                },
                "task3": {
                    "index": "3",
                    "name": "task3",
                    "type": "non-bitbake",
                    "condition": "$#[TASK3_CONDITION]"
                },
                "task4": {
                    "index": "4",
                    "name": "task4",
                    "type": "non-bitbake",
                    "condition": "$#[TASK4_CONDITION]"
                },
                "task5": {
                    "index": "5",
                    "name": "task5",
                    "type": "non-bitbake",
                    "condition": "$#[TASK5_CONDITION]"
                },
                "task6": {
                    "index": "6",
                    "name": "task6",
                    "type": "non-bitbake",
                    "condition": "$#[TASK6_CONDITION]"
                },
                "task7": {
                    "index": "7",
                    "name": "task7",
                    "type": "non-bitbake",
                    "condition": "$#[TASK7_CONDITION]"
                },
                "task8": {
                    "index": "8",
                    "name": "task8",
                    "type": "non-bitbake",
                    "condition": "$#[TASK8_CONDITION]"
                },
                "task9": {
                    "index": "9",
                    "name": "task9",
                    "type": "non-bitbake",
                    "condition": "$#[TASK8_CONDITION]"
                }
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).unwrap();
        let mut ws_config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut ws_settings).expect("Failed to parse build config");
        ws_config.expand_ctx().unwrap();
        for mut i in 1..9 {
            let result: Result<&WsTaskHandler, BError> = ws_config.task(format!("task{}", i).as_str());
            match result {
                Ok(task) => {
                    if !task.data().condition() {
                        panic!("Failed to evaluate condition nbr {}", i);
                    }
                },
                Err(e) => {
                    panic!("{}", e.to_string());
                }
            }
            i += 1;
        }
    }

    #[test]
    fn test_ws_task_config_build_dir() {
        let json_settings = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {
            "version": "5",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "context": [
                "TASK1_BUILD_DIR=task1/build"
            ],
            "tasks": {
                "task1": {
                    "index": "1",
                    "name": "task1",
                    "type": "non-bitbake",
                    "builddir": "$#[TASK1_BUILD_DIR]/dir/"
                },
                "task2": {
                    "index": "2",
                    "name": "task2",
                    "type": "bitbake",
                    "recipes": [
                        "test-image"
                    ]
                }
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).unwrap();
        let mut ws_config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut ws_settings).expect("Failed to parse build config");
        ws_config.expand_ctx().unwrap();
        assert_eq!(ws_config.task("task1").unwrap().data().build_dir(), &PathBuf::from("/workspace/task1/build/dir"));
        assert_eq!(ws_config.task("task2").unwrap().data().build_dir(), &PathBuf::from("/workspace/builds/test-name"));
    }

    #[test]
    fn test_ws_config_context_task_build_dir() {
        let json_settings = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {
            "version": "5",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "context": [
                "TASK1_BUILD_DIR=task1/build/dir"
            ],
            "tasks": {
                "task1": {
                    "index": "0",
                    "name": "task1-name",
                    "type": "non-bitbake",
                    "builddir": "test/$#[TASK1_BUILD_DIR]",
                    "build": "build-cmd",
                    "clean": "clean-cmd",
                    "artifacts": []
                },
                "task2": {
                    "index": "1",
                    "name": "task2-name",
                    "type": "bitbake",
                    "recipes": [
                        "image-recipe"
                    ],
                    "artifacts": []
                }
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).unwrap();
        let mut ws_config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut ws_settings).expect("Failed to parse build config");
        ws_config.expand_ctx().unwrap();
        assert_eq!(ws_config.task("task1").unwrap().data().build_dir(), &PathBuf::from("/workspace/test/task1/build/dir"));
        assert_eq!(ws_config.task("task2").unwrap().data().build_dir(), &PathBuf::from("/workspace/builds/test-name"));
        {
            let result: Result<&WsTaskHandler, BError> = ws_config.task("task3");
            match result {
                Ok(_task) => {
                    panic!("We should have recived an error because we have no task3 defined!");
                },
                Err(e) => {
                    assert_eq!(e.to_string(), "Task 'task3' does not exists in build config".to_string());
                }
            }
        }
    }

    #[test]
    fn test_ws_config_tasks() {
        let json_settings = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {
            "version": "5",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "tasks": {
                "task0": {
                    "index": "0",
                    "name": "task0",
                    "type": "non-bitbake",
                    "builddir": "test/task0",
                    "build": "cmd0",
                    "clean": "clean0",
                    "artifacts": [
                        {
                            "source": "test/file0-1.txt"
                        }
                    ]
                },
                "task1": {
                    "index": "1",
                    "name": "task1",
                    "type": "non-bitbake",
                    "builddir": "test/task1",
                    "build": "cmd1",
                    "clean": "clean1",
                    "artifacts": [
                        {
                            "source": "test/file1-1.txt"
                        }
                    ]
                },
                "task2": {
                    "index": "2",
                    "name": "task2",
                    "type": "non-bitbake",
                    "builddir": "test/task2",
                    "build": "cmd2",
                    "clean": "clean2",
                    "artifacts": [
                        {
                            "source": "test/file2-1.txt"
                        }
                    ]
                }
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).unwrap();
        let ws_config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut ws_settings).expect("Failed to parse build config");

        let mut i: i32 = 0;
        ws_config.tasks().iter().for_each(|(name, task)| {
            assert_eq!(name, &format!("task{}", i));
            assert_eq!(task.data().build_dir(), &PathBuf::from(format!("/workspace/test/task{}", i)));
            assert_eq!(task.data().build_cmd(), &format!("cmd{}", i));
            assert_eq!(task.data().clean_cmd(), &format!("clean{}", i));
            task.artifacts().iter().for_each(|a| {
                assert_eq!(a.data().source(), &format!("test/file{}-1.txt", i));
            });
            i += 1;
        });
    }

    #[test]
    fn test_ws_config_incompatible_version() {
        let json_settings = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).unwrap();
        let result: Result<WsBuildConfigHandler, BError> = WsBuildConfigHandler::from_str(json_build_config, &mut ws_settings);
        match result {
            Ok(_cfg) => {
                assert!(false, "Expected an error");
            }
            Err(err) => {
                assert_eq!("The build config version '4' is not compatible with current bakery version. \
                    Update config to match the format of version '5'", err.to_string());
            }
        }
    }
}