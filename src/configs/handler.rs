use std::path::PathBuf;

use crate::error::BError;
use crate::fs::ConfigFileReader;
use crate::workspace::{WsBuildConfigHandler, WsSettingsHandler};

const WORKSPACE_SETTINGS: &str = "workspace.json";

pub struct WsConfigFileHandler {
    work_dir: PathBuf,
    bakery_dir: PathBuf,
}

impl WsConfigFileHandler {
    pub fn new(work_dir: &PathBuf, home_dir: &PathBuf) -> Self {
        let bakery_dir: PathBuf = home_dir.clone().join(".bakery");
        WsConfigFileHandler {
            work_dir: work_dir.clone(),
            bakery_dir,
        }
    }

    pub fn ws_settings(&self) -> Result<WsSettingsHandler, BError> {
        let mut path: PathBuf = self.bakery_dir.clone().join(WORKSPACE_SETTINGS);

        /*
         * The workspace settings file workspace.json can be placed under ${HOME}/.bakery/workspace.json
         * if available that file will be used for any workspace that is used by the bakery. This can be
         * use if for some reason a baker would like to overwrite the workspace settings that are defined
         * in the repo for the product that is going to be baked.
         */
        if path.exists() {
            let settings_str: String = ConfigFileReader::new(&path).read_json()?;
            return WsSettingsHandler::from_str(&self.work_dir, &settings_str);
        }

        /*
         * The default location for the workspace settings is the current directory from where bakery is executed
         * normally this file is part of the repo that have been cloned containing the meta data to build the product
         */
        path = self.work_dir.clone().join(WORKSPACE_SETTINGS);
        if path.exists() {
            let settings_str: String = ConfigFileReader::new(&path).read_json()?;
            return WsSettingsHandler::from_str(&self.work_dir, &settings_str);
        }

        /*
         * Return default settings the only thing required is the version the rest
         * be defined by the settings handler if it is not defined in the json
         */
        let default_settings: &str = r#"
        {
            "version": "6"
        }"#;
        return WsSettingsHandler::from_str(&self.work_dir, default_settings);
    }

    fn config_header(&self, config: &WsBuildConfigHandler) -> String {
        let cfg_bitbake_json: String = config.build_data().bitbake().to_string();
        let cfg_product_json: String = config.build_data().product().to_string();
        let cfg_header_json: String = format!("{},{}", cfg_product_json, cfg_bitbake_json);
        cfg_header_json.clone()
    }

    pub fn setup_build_config(
        &self,
        path: &PathBuf,
        settings: &WsSettingsHandler,
    ) -> Result<WsBuildConfigHandler, BError> {
        let build_config_json: String = ConfigFileReader::new(&path).read_json()?;
        let mut main_config: WsBuildConfigHandler =
            WsBuildConfigHandler::from_str(&build_config_json, settings)?;
        let cfg_header_json: String = self.config_header(&main_config);

        /*
         * Iterate over any included build config and extend the main build config with the included
         * build configs. Currently the included build configs will only extend the main config with
         * the tasks and any of the built-in sub-commands sync, setup, upload, deploy
         */
        for config in main_config.build_data().included_configs().iter() {
            let cfg_include_json: String = ConfigFileReader::new(config).read_json()?;
            /*
             * The included build config does not and should not contain anything but the tasks and custom sub commands but because
             * each task is handling it's own build dir which is setup by the bb segment we need to inject the bb to the WsBuildConfigHandler
             * string.
             */
            let cfg_json: String = format!(
                "{{{},{}}}",
                cfg_header_json,
                cfg_include_json
                    .trim_start()
                    .trim_start_matches('{')
                    .trim_end()
                    .trim_end_matches('}')
            );
            let mut cfg: WsBuildConfigHandler =
                WsBuildConfigHandler::from_str(&cfg_json, settings)?;
            main_config.merge(&mut cfg);
        }

        return Ok(main_config);
    }

    pub fn build_config(
        &self,
        name: &str,
        settings: &WsSettingsHandler,
    ) -> Result<WsBuildConfigHandler, BError> {
        let mut build_config: PathBuf = PathBuf::from(name);
        build_config.set_extension("json");
        let mut path: PathBuf = settings.work_dir().join(build_config.clone());

        /* We start by looking for the build config in the workspace/work directory */
        if path.exists() {
            return self.setup_build_config(&path, settings);
        }

        /*
         * If we cannot locate the build config in the workspace/work dir we continue to look
         * for it under the configs dir
         */
        path = settings.configs_dir().join(build_config.clone());
        if path.exists() {
            return self.setup_build_config(&path, settings);
        }

        /* TODO: we should remove this and most likely refactor the code so that the sub-commands are responsible for the build config */
        if build_config.display().to_string() == "NA.json".to_string() {
            let dummy_config_json: &str = r#"
                {
                    "version": "6",
                    "name": "all",
                    "description": "Dummy build config to be able to handle 'list' sub-command",
                    "arch": "NA"
                }"#;
            return WsBuildConfigHandler::from_str(&dummy_config_json, settings);
        }

        return Err(BError::ValueError(format!(
            "Build config '{}' missing!",
            build_config.clone().display()
        )));
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempdir::TempDir;

    use crate::configs::WsConfigFileHandler;
    use crate::error::BError;
    use crate::helper::Helper;
    use crate::workspace::{
        WsBuildConfigHandler, WsCustomSubCmdHandler, WsSettingsHandler, WsTaskHandler,
    };

    fn write_json_conf(path: &PathBuf, json_str: &str) {
        if let Some(parent_dir) = path.parent() {
            std::fs::create_dir_all(parent_dir).expect("Failed create parent dir");
        }

        let mut file: File = File::create(&path).expect("Failed to create file");

        // Write the JSON string to the file.
        file.write_all(json_str.as_bytes())
            .expect("Failed to write json to file");
    }

    /*
     * Test that if no workspace settings file is available the default is used.
     * All the directories should be the default once
     */
    #[test]
    fn test_cfg_handler_settings_default() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path()).join("workspace");
        let home_dir: PathBuf = PathBuf::from(temp_dir.path()).join("home");
        Helper::setup_test_ws_default_dirs(&work_dir);
        let cfg_handler: WsConfigFileHandler = WsConfigFileHandler::new(&work_dir, &home_dir);
        let settings: WsSettingsHandler = cfg_handler
            .ws_settings()
            .expect("Failed parse workspace settings");
        assert_eq!(settings.builds_dir(), work_dir.clone().join("builds"));
        assert_eq!(settings.cache_dir(), work_dir.clone().join(".cache"));
        assert_eq!(settings.artifacts_dir(), work_dir.clone().join("artifacts"));
        assert_eq!(settings.scripts_dir(), work_dir.clone().join("scripts"));
        assert_eq!(settings.docker_dir(), work_dir.clone().join("docker"));
        assert_eq!(settings.configs_dir(), work_dir.clone().join("configs"));
        assert_eq!(
            settings.include_dir(),
            work_dir.clone().join("configs/include")
        );
    }

    /*
     * Test that the workspace settings file in the home bakery config dir is used instead
     * of the one in the root of the workspace/work dir
     */
    #[test]
    fn test_cfg_handler_settings_home_dir() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path()).join("workspace");
        let home_dir: PathBuf = PathBuf::from(temp_dir.path()).join("home");
        Helper::setup_test_ws_default_dirs(&work_dir);
        let ws_settings_1: &str = r#"
        {
            "version": "6",
            "workspace": {
                "configsdir": "work_dir"
            }
        }"#;
        write_json_conf(&work_dir.clone().join("workspace.json"), ws_settings_1);
        let ws_settings_2: &str = r#"
        {
            "version": "6",
            "workspace": {
                "configsdir": "home_dir"
            }
        }"#;
        write_json_conf(
            &home_dir.clone().join(".bakery/workspace.json"),
            ws_settings_2,
        );
        let cfg_handler: WsConfigFileHandler = WsConfigFileHandler::new(&work_dir, &home_dir);
        let settings: WsSettingsHandler = cfg_handler
            .ws_settings()
            .expect("Failed parse workspace settings");
        assert_eq!(settings.configs_dir(), work_dir.clone().join("home_dir"));
    }

    /*
     * Test that the workspace settings file workspace/work dir is used
     */
    #[test]
    fn test_cfg_handler_settings_work_dir() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path()).join("workspace");
        let home_dir: PathBuf = PathBuf::from(temp_dir.path()).join("home");
        Helper::setup_test_ws_default_dirs(&work_dir);
        let ws_settings: &str = r#"
        {
            "version": "6",
            "workspace": {
                "configsdir": "work_dir"
            }
        }"#;
        write_json_conf(&work_dir.clone().join("workspace.json"), ws_settings);
        let cfg_handler: WsConfigFileHandler = WsConfigFileHandler::new(&work_dir, &home_dir);
        let settings: WsSettingsHandler = cfg_handler
            .ws_settings()
            .expect("Failed parse workspace settings");
        assert_eq!(settings.configs_dir(), work_dir.join("work_dir"));
    }

    /*
     * Test that what happens if no build config an Error should be returned
     */
    #[test]
    fn test_cfg_handler_build_config() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path()).join("workspace");
        let home_dir: PathBuf = PathBuf::from(temp_dir.path()).join("home");
        Helper::setup_test_ws_default_dirs(&work_dir);
        let cfg_handler: WsConfigFileHandler = WsConfigFileHandler::new(&work_dir, &home_dir);
        let settings: WsSettingsHandler = cfg_handler
            .ws_settings()
            .expect("Failed parse workspace settings");
        let result: Result<WsBuildConfigHandler, BError> =
            cfg_handler.build_config("invalid", &settings);
        match result {
            Ok(_build_cfg) => {
                panic!("Was expecting an error!");
            }
            Err(e) => {
                assert_eq!(
                    e.to_string(),
                    String::from("Build config 'invalid.json' missing!")
                );
            }
        }
    }

    /*
     * Test that if there exists a build config in the workspace/work dir then that is picked up
     */
    #[test]
    fn test_cfg_handler_ws_root_build_config() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path()).join("workspace");
        let home_dir: PathBuf = PathBuf::from(temp_dir.path()).join("home");
        let cfg_handler: WsConfigFileHandler = WsConfigFileHandler::new(&work_dir, &home_dir);
        let settings: WsSettingsHandler = cfg_handler
            .ws_settings()
            .expect("Failed parse workspace settings");
        Helper::setup_test_ws_default_dirs(&work_dir);
        let build_conf_ws_root_dir = r#"
        {
            "version": "6",
            "name": "ws-root-build-config",
            "description": "Test Description",
            "arch": "test-arch"
        }"#;
        write_json_conf(
            &settings.work_dir().join("test.json"),
            build_conf_ws_root_dir,
        );
        let build_conf_configs_dir = r#"
        {
            "version": "6",
            "name": "ws-configs-build-config",
            "description": "Test Description",
            "arch": "test-arch"
        }"#;
        write_json_conf(
            &settings.configs_dir().join("test.json"),
            build_conf_configs_dir,
        );
        let config: WsBuildConfigHandler = cfg_handler
            .build_config("test", &settings)
            .expect("Failed parse build config");
        assert_eq!(config.build_data().name(), "ws-root-build-config");
    }

    /*
     * Test that the build config is picked up from the configs dir
     */
    #[test]
    fn test_cfg_handler_ws_configs_build_config() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path()).join("workspace");
        let home_dir: PathBuf = PathBuf::from(temp_dir.path()).join("home");
        let cfg_handler: WsConfigFileHandler = WsConfigFileHandler::new(&work_dir, &home_dir);
        let settings: WsSettingsHandler = cfg_handler
            .ws_settings()
            .expect("Failed parse workspace settings");
        Helper::setup_test_ws_default_dirs(&work_dir);
        let build_conf_configs_dir = r#"
        {
            "version": "6",
            "name": "ws-configs-build-config",
            "description": "Test Description",
            "arch": "test-arch"
        }"#;
        write_json_conf(
            &settings.configs_dir().join("test.json"),
            build_conf_configs_dir,
        );
        let config: WsBuildConfigHandler = cfg_handler
            .build_config("test", &settings)
            .expect("Failed parse build config");
        assert_eq!(config.build_data().name(), "ws-configs-build-config");
    }

    #[test]
    fn test_cfg_handler_ws_include_configs() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path()).join("workspace");
        let home_dir: PathBuf = PathBuf::from(temp_dir.path()).join("home");
        let cfg_handler: WsConfigFileHandler = WsConfigFileHandler::new(&work_dir, &home_dir);
        let settings: WsSettingsHandler = cfg_handler
            .ws_settings()
            .expect("Failed parse workspace settings");
        Helper::setup_test_ws_default_dirs(&work_dir);
        let main_build_config = r#"
        {
            "version": "6",
            "name": "test-product",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {
                "machine": "test-machine",
                "distro": "test-distro",
                "deploydir": "tmp/test/deploy",
                "docker": "test-registry/test-image:0.1",
                "initenv": "layers/test/oe-my-init-env",
                "bblayersconf": [
                    "BB_LAYERS_CONF_TEST_LINE_1",
                    "BB_LAYERS_CONF_TEST_LINE_2",
                    "BB_LAYERS_CONF_TEST_LINE_3"
                ],
                "localconf": [
                    "BB_LOCAL_CONF_TEST_LINE_1",
                    "BB_LOCAL_CONF_TEST_LINE_2",
                    "BB_LOCAL_CONF_TEST_LINE_3"
                ]
            },
            "include": [
                "config1",
                "config2"
            ],
            "tasks": {
                "task0": {
                    "index": "0",
                    "name": "task0",
                    "type": "non-bitbake",
                    "builddir": "test/main",
                    "build": "main",
                    "clean": "main",
                    "artifacts": [
                        {
                            "source": "test/main-file.txt"
                        }
                    ]
                }
            },
            "setup": {
                "cmd": "main"
            }
        }"#;
        write_json_conf(&settings.work_dir().join("main.json"), main_build_config);
        let build_config1 = r#"
        {
            "version": "6",
            "tasks": {
                "task0": {
                    "index": "0",
                    "name": "task0",
                    "type": "non-bitbake",
                    "builddir": "test/config1",
                    "build": "config1",
                    "clean": "config1",
                    "artifacts": [
                        {
                            "source": "test/config.txt"
                        }
                    ]
                },
                "task1": {
                    "index": "1",
                    "name": "task1",
                    "recipes": [
                        "test"
                    ],
                    "artifacts": [
                        {
                            "source": "test/config.txt"
                        }
                    ]
                }
            },
            "setup": {
                "cmd": "config1"
            },
            "sync": {
                "cmd": "config1"
            }
        }"#;
        write_json_conf(&settings.include_dir().join("config1.json"), build_config1);
        let build_config2 = r#"
        {
            "version": "6",
            "tasks": {
                "task2": {
                    "index": "2",
                    "name": "task2",
                    "type": "non-bitbake",
                    "builddir": "test/config2",
                    "build": "config2",
                    "clean": "config2",
                    "artifacts": [
                        {
                            "source": "test/config.txt"
                        }
                    ]
                }
            },
            "upload": {
                "cmd": "config2"
            }
        }"#;
        write_json_conf(&settings.include_dir().join("config2.json"), build_config2);
        let config: WsBuildConfigHandler = cfg_handler
            .build_config("main", &settings)
            .expect("Failed parse build config");
        assert_eq!(config.build_data().name(), "test-product");
        let t0: &WsTaskHandler = config.tasks().get("task0").unwrap();
        assert_eq!(t0.data().build_cmd(), "main");
        assert_eq!(
            t0.data().build_dir(),
            &settings.work_dir().join("test/main")
        );
        let t1: &WsTaskHandler = config.tasks().get("task1").unwrap();
        assert_eq!(t1.data().recipes(), &vec!["test"]);
        assert_eq!(
            t1.data().build_dir(),
            &settings.work_dir().join("builds/test-product")
        );
        let t2: &WsTaskHandler = config.tasks().get("task2").unwrap();
        assert_eq!(t2.data().build_cmd(), "config2");
        assert_eq!(
            t2.data().build_dir(),
            &settings.work_dir().join("test/config2")
        );
        let setup: &WsCustomSubCmdHandler = config.subcmds().get("setup").unwrap();
        assert_eq!(setup.data().cmd(), "main");
        let sync: &WsCustomSubCmdHandler = config.subcmds().get("sync").unwrap();
        assert_eq!(sync.data().cmd(), "config1");
        let upload: &WsCustomSubCmdHandler = config.subcmds().get("upload").unwrap();
        assert_eq!(upload.data().cmd(), "config2");
    }
}
