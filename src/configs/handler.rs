use std::path::PathBuf;

use crate::workspace::{WsSettingsHandler, WsBuildConfigHandler};
use crate::fs::ConfigFileReader;
use crate::error::BError;

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
        let default_settings: &str  = r#"
        {
            "version": "5"
        }"#;
        return WsSettingsHandler::from_str(&self.work_dir, default_settings);
    }

    pub fn build_config(self, name: &str, settings: &WsSettingsHandler) -> Result<WsBuildConfigHandler, BError> {
        let mut build_config: PathBuf = PathBuf::from(name);
        build_config.set_extension("json");
        let mut path: PathBuf = settings.work_dir().join(build_config.clone());

        /* We start by looking for the build config in the workspace/work directory */
        if path.exists() {
            let build_config_json: String = ConfigFileReader::new(&path).read_json()?;
            return WsBuildConfigHandler::from_str(&build_config_json, settings);
        }

        /*
         * If we cannot locate the build config in the workspace/work dir we continue to look
         * for it under the configs dir
         */
        path = settings.configs_dir().join(build_config.clone());
        if path.exists() {
            let build_config_json: String = ConfigFileReader::new(&path).read_json()?;
            return WsBuildConfigHandler::from_str(&build_config_json, settings);
        }

        /* TODO: we should remove this and most likely refactor the code so that the sub-commands are responsible for the build config */
        if build_config.display().to_string() == "NA.json".to_string() {
            let dummy_config_json: &str = r#"
                {
                    "version": "5",
                    "name": "all",
                    "description": "Dummy build config to be able to handle 'list' sub-command",
                    "arch": "NA"
                }"#;
            return WsBuildConfigHandler::from_str(&dummy_config_json, settings);
        }

        return Err(BError::ValueError(format!("Build config '{}' missing!", build_config.clone().display())));
    }
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;
    use std::path::PathBuf;
    use std::fs::File;
    use std::io::Write;

    use crate::helper::Helper;
    use crate::configs::WsConfigFileHandler;
    use crate::workspace::{WsSettingsHandler, WsBuildConfigHandler};
    use crate::error::BError;

    fn write_json_conf(path: &PathBuf, json_str: &str) {
        if let Some(parent_dir) = path.parent() {
            std::fs::create_dir_all(parent_dir).expect("Failed create parent dir");
        }

        let mut file: File = File::create(&path).expect("Failed to create file");

        // Write the JSON string to the file.
        file.write_all(json_str.as_bytes()).expect("Failed to write json to file");
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
        let settings: WsSettingsHandler = cfg_handler.ws_settings().expect("Failed parse workspace settings");
        assert_eq!(settings.builds_dir(), work_dir.clone().join("builds"));
        assert_eq!(settings.cache_dir(), work_dir.clone().join(".cache"));
        assert_eq!(settings.artifacts_dir(), work_dir.clone().join("artifacts"));
        assert_eq!(settings.scripts_dir(), work_dir.clone().join("scripts"));
        assert_eq!(settings.docker_dir(), work_dir.clone().join("docker"));
        assert_eq!(settings.configs_dir(), work_dir.clone().join("configs"));
        assert_eq!(settings.include_dir(), work_dir.clone().join("configs/include"));
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
            "version": "5",
            "workspace": {
                "configsdir": "work_dir"
            }
        }"#;
        write_json_conf(&work_dir.clone().join("workspace.json"), ws_settings_1);
        let ws_settings_2: &str = r#"
        {
            "version": "5",
            "workspace": {
                "configsdir": "home_dir"
            }
        }"#;
        write_json_conf(&home_dir.clone().join(".bakery/workspace.json"), ws_settings_2);
        let cfg_handler: WsConfigFileHandler = WsConfigFileHandler::new(&work_dir, &home_dir);
        let settings: WsSettingsHandler = cfg_handler.ws_settings().expect("Failed parse workspace settings");
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
            "version": "5",
            "workspace": {
                "configsdir": "work_dir"
            }
        }"#;
        write_json_conf(&work_dir.clone().join("workspace.json"), ws_settings);
        let cfg_handler: WsConfigFileHandler = WsConfigFileHandler::new(&work_dir, &home_dir);
        let settings: WsSettingsHandler = cfg_handler.ws_settings().expect("Failed parse workspace settings");
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
        let settings: WsSettingsHandler = cfg_handler.ws_settings().expect("Failed parse workspace settings");
        let result: Result<WsBuildConfigHandler, BError> = cfg_handler.build_config("invalid", &settings);
        match result {
            Ok(_build_cfg) => {
                panic!("Was expecting an error!");
            },
            Err(e) => {
                assert_eq!(e.to_string(), String::from("Build config 'invalid.json' missing!"));
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
        let settings: WsSettingsHandler = cfg_handler.ws_settings().expect("Failed parse workspace settings");
        Helper::setup_test_ws_default_dirs(&work_dir);
        let build_conf_ws_root_dir = r#"
        {
            "version": "5",
            "name": "ws-root-build-config",
            "description": "Test Description",
            "arch": "test-arch"
        }"#;
        write_json_conf(&settings.work_dir().join("test.json"),  build_conf_ws_root_dir);
        let build_conf_configs_dir = r#"
        {
            "version": "5",
            "name": "ws-configs-build-config",
            "description": "Test Description",
            "arch": "test-arch"
        }"#;
        write_json_conf(&settings.configs_dir().join("test.json"),  build_conf_configs_dir);
        let config: WsBuildConfigHandler = cfg_handler.build_config("test", &settings).expect("Failed parse build config");
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
        let settings: WsSettingsHandler = cfg_handler.ws_settings().expect("Failed parse workspace settings");
        Helper::setup_test_ws_default_dirs(&work_dir);
        let build_conf_configs_dir = r#"
        {
            "version": "5",
            "name": "ws-configs-build-config",
            "description": "Test Description",
            "arch": "test-arch"
        }"#;
        write_json_conf(&settings.configs_dir().join("test.json"),  build_conf_configs_dir);
        let config: WsBuildConfigHandler = cfg_handler.build_config("test", &settings).expect("Failed parse build config");
        assert_eq!(config.build_data().name(), "ws-configs-build-config");
    }
}