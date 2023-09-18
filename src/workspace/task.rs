use crate::configs::{TType, TaskConfig, ArtifactConfig};
use crate::workspace::WsBuildConfigHandler;

use std::path::{Path, PathBuf};

pub struct WsTaskConfigHandler<'a> {
    name: String,
    task_config: &'a TaskConfig,
    ws_config: &'a WsBuildConfigHandler,
}

impl<'a> WsTaskConfigHandler<'a> {
    pub fn new(task_config: &'a TaskConfig, ws_config: &'a WsBuildConfigHandler) -> Self {
        WsTaskConfigHandler {
            name: task_config.name().to_string(),
            task_config,
            ws_config,
        }
    }

    pub fn build_dir(&self) -> PathBuf {
        if self.task_config.ttype() == TType::Bitbake {
            let task_build_dir: &str = self.task_config.builddir();
            if task_build_dir.is_empty() {
                return self.ws_config.bb_build_dir();
            }
        }

        self.ws_config
                .work_dir()
                .join(PathBuf::from(self.task_config.builddir()))

        /*
        return self.ws_config.context().expand_path(
            &self
                .ws_config
                .work_dir()
                .join(PathBuf::from(self.task_config.builddir())),
        );
         */
    }

    pub fn ttype(&self) -> TType {
        self.task_config.ttype()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn build_cmd(&self) -> &str {
        self.task_config.build()
        /*
        self.ws_config
            .context()
            .expand_str(self.task_config.build())
        */
    }

    pub fn clean_cmd(&self) -> &str {
        self.task_config.clean()
        /*
        self.ws_config
            .context()
            .expand_str(self.task_config.clean())
        */
    }

    pub fn docker(&self) -> &str {
        self.task_config.docker()
        /*
        self.ws_config
            .context()
            .expand_str(self.task_config.docker())
        */
    }

    pub fn disabled(&self) -> bool {
        if self.task_config.disabled() == "true" {
            return true;
        }
        return false;
    }

    pub fn recipes(&self) -> &Vec<String> {
        self.task_config.recipes()
        //self.ws_config.context().expand_vec(self.task_config.recipes())
    }

    pub fn condition(&self) -> bool {
        let condition: &str = self.task_config.condition();

        if condition.is_empty() {
            return true;
        }

        //match self.ws_config.context().expand_str(condition).as_str() {
        match condition {
            "1" | "yes" | "y" | "Y" | "true" | "YES" | "TRUE" | "True" | "Yes" => return true,
            _ => return false,
        }
    }
    
    pub fn artifacts(&self) -> &Vec<ArtifactConfig> {
        self.task_config.artifacts()
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::error::BError;
    use crate::helper::Helper;
    use crate::workspace::{WsBuildConfigHandler, WsTaskConfigHandler};
    use crate::configs::TType;

    #[test]
    fn test_ws_task_config() {
        let json_settings = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "context": [
                "TASK_BUILD_DIR=task/build/dir",
                "TASK_CONDITION=1"
            ],
            "bb": {
            },
            "tasks": { 
                "task": {
                    "index": "0",
                    "name": "task-name",
                    "type": "non-bitbake",
                    "disabled": "false",
                    "condition": "${TASK_CONDITION}",
                    "builddir": "test/${TASK_BUILD_DIR}",
                    "build": "build-cmd",
                    "clean": "clean-cmd",
                    "artifacts": []
                }
            }
        }"#;
        let ws_config: WsBuildConfigHandler = Helper::setup_ws_config_handler("/workspace", json_settings, json_build_config);
        {
            let result: Result<WsTaskConfigHandler, BError> = ws_config.task("task");
            match result {
                Ok(task) => {
                    assert_eq!(task.build_dir(), PathBuf::from("/workspace/test/task/build/dir"));
                    assert!(task.condition());
                    assert_eq!(task.name(), "task-name");
                    assert_eq!(task.build_cmd(), "build-cmd");
                    assert_eq!(task.clean_cmd(), "clean-cmd");
                    assert_eq!(task.ttype(), TType::NonBitbake);
                    assert!(!task.disabled());
                },
                Err(e) => {
                    panic!("{}", e.message);
                }
            }
        }
    }

    #[test]
    fn test_ws_task_config_condition() {
        let json_settings = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {                                                                                                                   
            "version": "4",
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
            "bb": {
            },
            "tasks": { 
                "task1": {
                    "index": "1",
                    "name": "task1",
                    "type": "non-bitbake",
                    "condition": "${TASK1_CONDITION}"
                },
                "task2": {
                    "index": "2",
                    "name": "task2",
                    "type": "non-bitbake",
                    "condition": "${TASK2_CONDITION}"
                },
                "task3": {
                    "index": "3",
                    "name": "task3",
                    "type": "non-bitbake",
                    "condition": "${TASK3_CONDITION}"
                },
                "task4": {
                    "index": "4",
                    "name": "task4",
                    "type": "non-bitbake",
                    "condition": "${TASK4_CONDITION}"
                },
                "task5": {
                    "index": "5",
                    "name": "task5",
                    "type": "non-bitbake",
                    "condition": "${TASK5_CONDITION}"
                },
                "task6": {
                    "index": "6",
                    "name": "task6",
                    "type": "non-bitbake",
                    "condition": "${TASK6_CONDITION}"
                },
                "task7": {
                    "index": "7",
                    "name": "task7",
                    "type": "non-bitbake",
                    "condition": "${TASK7_CONDITION}"
                },
                "task8": {
                    "index": "8",
                    "name": "task8",
                    "type": "non-bitbake",
                    "condition": "${TASK8_CONDITION}"
                },
                "task9": {
                    "index": "9",
                    "name": "task9",
                    "type": "non-bitbake",
                    "condition": "${TASK8_CONDITION}"
                }
            }
        }"#;
        let ws_config: WsBuildConfigHandler = Helper::setup_ws_config_handler("/workspace", json_settings, json_build_config);
        for mut i in 1..9 {
            let result: Result<WsTaskConfigHandler, BError> = ws_config.task(format!("task{}", i).as_str());
            match result {
                Ok(task) => {
                    if !task.condition() {
                        panic!("Failed to evaluate condition nbr {}", i);
                    }
                },
                Err(e) => {
                    panic!("{}", e.message);
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
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "context": [
                "TASK1_BUILD_DIR=task1/build"
            ],
            "bb": {
            },
            "tasks": { 
                "task1": {
                    "index": "1",
                    "name": "task1",
                    "type": "non-bitbake",
                    "builddir": "${TASK1_BUILD_DIR}/dir/"
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
        let ws_config: WsBuildConfigHandler = Helper::setup_ws_config_handler("/workspace", json_settings, json_build_config);
        {
            let result: Result<WsTaskConfigHandler, BError> = ws_config.task("task1");
            match result {
                Ok(task) => {
                    assert_eq!(task.build_dir(), PathBuf::from("/workspace/task1/build/dir/"));
                },
                Err(e) => {
                    panic!("{}", e.message);
                }
            }
        }
        {
            let result: Result<WsTaskConfigHandler, BError> = ws_config.task("task2");
            match result {
                Ok(task) => {
                    assert_eq!(task.build_dir(), PathBuf::from("/workspace/builds/test-name"));
                },
                Err(e) => {
                    panic!("{}", e.message);
                }
            }
        }
    }
}
