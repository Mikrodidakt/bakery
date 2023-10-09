use indexmap::{IndexMap, indexmap};
use std::path::PathBuf;
use serde_json::Value;

use crate::configs::{Context, BuildConfig, TType};
use crate::workspace::{WsSettingsHandler, WsBuildData, WsTaskHandler};
use crate::error::BError;
use crate::fs::JsonFileReader;

pub struct WsBuildConfigHandler {
    data: WsBuildData,
    config: BuildConfig,
    work_dir: PathBuf,
    build_dir: PathBuf,
    cache_dir: PathBuf,
    artifacts_dir: PathBuf,
    tasks: IndexMap<String, WsTaskHandler>,
}

impl WsBuildConfigHandler {
    pub fn from_str(json_config: &str, settings: &WsSettingsHandler) -> Result<Self, BError> {
        let data: Value = JsonFileReader::parse(json_config)?;
        Self::new(&data, settings)
    }

    pub fn new(data: &Value, settings: &WsSettingsHandler) -> Result<Self, BError> {
        let mut config: BuildConfig = BuildConfig::from_value(&data)?;
        // Define the context variables that is only defined in the build config
        let ctx_variables: IndexMap<String, String> = indexmap! {
            "MACHINE".to_string() => config.bitbake.machine.to_string(),
            "ARCH".to_string() => config.arch.to_string(),
            "DISTRO".to_string() => config.bitbake.distro.to_string(),
            "VARIANT".to_string() => "".to_string()
        };
        let mut ws_build_data: WsBuildData = WsBuildData::new(
            &config.name,
            &config.bitbake.deploy_dir,
            ctx_variables,
            settings
        )?;
        // Update the context with variables defined in the build config
        ws_build_data.update_context(&config.context);
        // Expand all context variables in the main build config
        config.expand_ctx(ws_build_data.context());
        let tasks: IndexMap<String, WsTaskHandler> = ws_build_data.get_tasks(&data)?;

        Ok(WsBuildConfigHandler {
            config,
            data: ws_build_data,
            work_dir: settings.work_dir().clone(),
            build_dir: settings.builds_dir().clone(),
            cache_dir: settings.cache_dir().clone(),
            artifacts_dir: settings.artifacts_dir().clone(),
            tasks,
        })
    }

    pub fn work_dir(&self) -> PathBuf {
        self.work_dir.clone()
    }

    pub fn artifacts_dir(&self) -> PathBuf {
        self.artifacts_dir.clone()
    }

    pub fn context(&self) -> &Context {
        self.data.context()
    }

    pub fn task(&self, task: &str) -> Result<&WsTaskHandler, BError> {
        match self.tasks.get(task) {
            Some(config) => {
                return Ok(config);
            },
            None => {
                return Err(BError{ code: 0, message: format!("Task '{}' does not exists in build config", task)});
            }
        }
    }

    pub fn tasks(&self) -> &IndexMap<String, WsTaskHandler> {
        &self.tasks
    }

    pub fn extend_ctx(&self, ctx: &Context) {}

    pub fn description(&self) -> &str {
        &self.config.description
    }

    pub fn product_name(&self) -> &str {
        // Currently the product name is the
        // same as config name but this might not
        // be the case in the future so therefore
        // I have added a specific getter
        self.config_name()
    }

    pub fn config_name(&self) -> &str {
        &self.config.name
    }

    //pub fn config_enabled(&self) -> bool {
    //    self.config.enabled()
    //}

    pub fn version(&self) -> &str {
        &self.config.version
    }

    pub fn arch(&self) -> &str {
        &self.config.arch
    }

    pub fn bb_layers_conf(&self) -> &Vec<String> {
        &self.config.bitbake.bblayers_conf
    }

    pub fn bb_local_conf(&self) -> Vec<String> {
        let mut local_conf: Vec<String> = self.config.bitbake.local_conf.clone();
        local_conf.push(format!("MACHINE ?= {}", self.bb_machine()));
        // TODO: we need to handle VARIANT correctly but this is good enough for now
        local_conf.push(format!("VARIANT ?= {}", "dev".to_string()));
        // TODO: we should define a method product_name() call that instead
        local_conf.push(format!("PRODUCT_NAME ?= {}", self.config.name));
        local_conf.push(format!("DISTRO ?= {}", self.bb_distro()));
        local_conf.push(format!("SSTATE_DIR ?= {}", self.bb_sstate_dir().to_str().unwrap()));
        local_conf.push(format!("DL_DIR ?= {}", self.bb_dl_dir().to_str().unwrap()));
        //local_conf.push(format!("PLATFORM_VERSION ?= {}", self.platform_version()));
        //local_conf.push(format!("BUILD_NUMBER ?= {}", self.build_number()));
        //local_conf.push(format!("BUILD_SHA ?= {}", self.build_sha()));
        //local_conf.push(format!("RELASE_BUILD ?= {}", self.release_build()));
        //local_conf.push(format!("BUILD_VARIANT ?= {}", self.build_variant()));
        local_conf
    }

    pub fn bb_machine(&self) -> &str {
        &self.config.bitbake.machine
    }

    //pub fn variant(&self) -> &str {
    //    self.config.variant()
    //}

    pub fn bb_distro(&self) -> &str {
        &self.config.bitbake.distro
    }

    pub fn bb_build_dir(&self) -> PathBuf {
        self.data.bb_build_dir()
    }

    pub fn bb_docker_image(&self) -> String {
        self.config.bitbake.docker.to_string()
    }

    pub fn bb_build_config_dir(&self) -> PathBuf {
        self.bb_build_dir().join("conf".to_string())
    }

    pub fn bb_local_config(&self) -> PathBuf {
        self.bb_build_config_dir().join("local.conf".to_string())
    }

    pub fn bb_layers_config(&self) -> PathBuf {
        self.bb_build_config_dir().join("bblayers.conf")
    }

    pub fn bb_deploy_dir(&self) -> PathBuf {
        self.data.bb_deploy_dir()
    }    

    pub fn bb_sstate_dir(&self) -> PathBuf {
        let mut path_buf = self.cache_dir.clone();
        path_buf.join(&self.config.arch).join("sstate-cache".to_string())
    }

    pub fn bb_dl_dir(&self) -> PathBuf {
        let mut path_buf = self.cache_dir.clone();
        path_buf.join("download".to_string())
    }        

    pub fn poky_dir(&self) -> PathBuf {
        // TODO: not sure about this we should not lock the bakery into using poky
        // we only need this to be able to determine the where to find the OE init file.
        // I think the solution is to add a entry in the build config file in the bb-node
        // where you can specify a path for the init file to source. The default could be
        // layers/poky/oe-init-build-env. Potentially we should also add an entry in the
        // Workspace settings file where you can specify the layers directory
        let mut path_buf = self.work_dir.clone();
        path_buf.join("layers".to_string()).join("poky".to_string())
    }

    pub fn oe_init_file(&self) -> PathBuf {
        // TODO: we should probably setup an option to configure what OE init script
        // to source to setup the env.
        self.poky_dir().join("oe-init-build-env")
    }   
}

#[cfg(test)]
mod tests {
    use std::path::{PathBuf, Path};

    use crate::workspace::{WsSettingsHandler, WsBuildConfigHandler, WsTaskHandler};
    use crate::helper::Helper;
    use crate::error::BError;

    #[test]
    fn test_ws_config_default() {
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
            "bb": {}
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).unwrap();
        let ws_config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut ws_settings).expect("Failed to parse build config");
        assert_eq!(ws_config.version(), "4".to_string());
        assert_eq!(ws_config.arch(), "test-arch".to_string());
        assert_eq!(ws_config.description(), "Test Description".to_string());
        assert_eq!(ws_config.config_name(), "test-name".to_string());
        assert_eq!(ws_config.product_name(), "test-name".to_string());
        assert_eq!(ws_config.bb_distro(), "".to_string());
        assert_eq!(ws_config.bb_machine(), "".to_string());
        assert_eq!(ws_config.bb_build_dir(), PathBuf::from("/workspace/builds/test-name"));
        assert_eq!(ws_config.bb_build_config_dir(), PathBuf::from("/workspace/builds/test-name/conf"));
        assert_eq!(ws_config.bb_deploy_dir(), PathBuf::from("/workspace/builds/test-name/tmp/deploy/images"));
        assert_eq!(ws_config.bb_dl_dir(), PathBuf::from("/workspace/.cache/download"));
        assert_eq!(ws_config.bb_sstate_dir(), PathBuf::from("/workspace/.cache/test-arch/sstate-cache"));
        assert_eq!(ws_config.bb_layers_config(), PathBuf::from("/workspace/builds/test-name/conf/bblayers.conf"));
        assert!(ws_config.bb_layers_conf().is_empty());
        assert_eq!(ws_config.bb_local_config(), PathBuf::from("/workspace/builds/test-name/conf/local.conf"));
        assert!(!ws_config.bb_local_conf().is_empty());
        let local_conf: Vec<String> = vec![
            format!("MACHINE ?= {}", ws_config.bb_machine()),
            "VARIANT ?= dev".to_string(),
            format!("PRODUCT_NAME ?= {}", ws_config.product_name()),
            format!("DISTRO ?= {}", ws_config.bb_distro()),
            format!("SSTATE_DIR ?= {}", ws_config.bb_sstate_dir().to_str().unwrap()),
            format!("DL_DIR ?= {}", ws_config.bb_dl_dir().to_str().unwrap()),
            //format!("PLATFORM_VERSION ?= {}", ws_config.platform_version()),
            //format!("BUILD_NUMBER ?= {}", ws_config.build_number()),
            //format!("BUILD_SHA ?= {}", ws_config.build_sha()),
            //format!("RELASE_BUILD ?= {}", ws_config.release_build()),
            //format!("BUILD_VARIANT ?= {}", ws_config.build_variant()),
        ];
        assert_eq!(ws_config.bb_local_conf(), local_conf);
        assert_eq!(ws_config.bb_docker_image(), "".to_string());

    }

    #[test]
    fn test_ws_config_context_docker() {
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
                "DOCKER_REGISTRY=test-registry",
                "DOCKER_TAG=0.1",
                "DOCKER_IMAGE=test-image"
            ],
            "bb": {
                "docker": "${DOCKER_REGISTRY}/${DOCKER_IMAGE}:${DOCKER_TAG}"
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).unwrap();
        let ws_config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut ws_settings).expect("Failed to parse build config");
        assert_eq!(ws_config.bb_docker_image(), "test-registry/test-image:0.1");
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
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).unwrap();
        let ws_config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut ws_settings).expect("Failed to parse build config");
        for mut i in 1..9 {
            let result: Result<&WsTaskHandler, BError> = ws_config.task(format!("task{}", i).as_str());
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
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let mut ws_settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).unwrap();
        let ws_config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut ws_settings).expect("Failed to parse build config");
        assert_eq!(ws_config.task("task1").unwrap().build_dir(), PathBuf::from("/workspace/task1/build/dir"));
        assert_eq!(ws_config.task("task2").unwrap().build_dir(), PathBuf::from("/workspace/builds/test-name"));
    }

    #[test]
    fn test_ws_config_context_task_build_dir() {
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
                "TASK1_BUILD_DIR=task1/build/dir"
            ],
            "bb": {
            },
            "tasks": { 
                "task1": {
                    "index": "0",
                    "name": "task1-name",
                    "type": "non-bitbake",
                    "builddir": "test/${TASK1_BUILD_DIR}",
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
        let ws_config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut ws_settings).expect("Failed to parse build config");
        assert_eq!(ws_config.task("task1").unwrap().build_dir(), PathBuf::from("/workspace/test/task1/build/dir"));
        assert_eq!(ws_config.task("task2").unwrap().build_dir(), PathBuf::from("/workspace/builds/test-name"));
        {
            let result: Result<&WsTaskHandler, BError> = ws_config.task("task3");
            match result {
                Ok(_task) => {
                    panic!("We should have recived an error because we have no task3 defined!");
                },
                Err(e) => {
                    assert_eq!(e.message, "Task 'task3' does not exists in build config".to_string());
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
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {},
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
            assert_eq!(task.build_dir(), PathBuf::from(format!("/workspace/test/task{}", i)));
            assert_eq!(task.build_cmd(), &format!("cmd{}", i));
            assert_eq!(task.clean_cmd(), &format!("clean{}", i));
            task.artifacts().iter().for_each(|a| {
                assert_eq!(a.source().as_path(), task.build_dir().join(format!("test/file{}-1.txt", i)));
            });
            i += 1;
        });
    }
}