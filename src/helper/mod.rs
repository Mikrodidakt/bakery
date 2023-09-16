use crate::workspace::{WsBuildConfigHandler, WsSettingsHandler, Workspace};
use crate::error::BError;
use crate::configs::{WsSettings, BuildConfig, TaskConfig};

use std::path::PathBuf; 

pub struct Helper;

impl Helper {
    pub fn setup_task_config(json_test_str: &str) -> TaskConfig {
        let result: Result<TaskConfig, BError> = TaskConfig::from_str(json_test_str);
        match result {
            Ok(rconfig) => {
                rconfig
            }
            Err(e) => {
                eprintln!("Error parsing tasks from build config: {}", e);
                panic!();
            } 
        }
    }

    pub fn setup_ws_settings(json_test_str: &str) -> WsSettings {
        let result: Result<WsSettings, BError> = WsSettings::from_str(json_test_str);
        let settings: WsSettings;
        match result {
            Ok(rsettings) => {
                settings = rsettings;
            }
            Err(e) => {
                eprintln!("Error parsing JSON: {}", e);
                panic!();
            } 
        }
        settings
    }

    pub fn setup_build_config(json_test_str: &str) -> BuildConfig {
        let result: Result<BuildConfig, BError> = BuildConfig::from_str(json_test_str);
        match result {
            Ok(rconfig) => {
                rconfig
            }
            Err(e) => {
                eprintln!("Error parsing build config: {}", e);
                panic!();
            } 
        }
    }

    pub fn setup_ws_config_handler(test_work_dir: &str, json_settings: &str, json_build_config: &str) -> WsBuildConfigHandler {
        let work_dir: PathBuf = PathBuf::from(test_work_dir);
        let settings: WsSettingsHandler = WsSettingsHandler::new(
            work_dir,
            Helper::setup_ws_settings(json_settings),
        );
        let result: Result<WsBuildConfigHandler, BError> = WsBuildConfigHandler::from_str(&settings, json_build_config);
        match result {
            Ok(ws_config) => {
                ws_config
            }
            Err(e) => {
                eprintln!("Error parsing build config: {}", e);
                panic!();
            } 
        }
    }

    pub fn setup_ws(test_work_dir: &str, json_settings: &str, json_build_config: &str) -> Workspace {
        let work_dir: PathBuf = PathBuf::from(test_work_dir);
        let ws_config: WsSettings = Self::setup_ws_settings(json_settings);
        let build_config: BuildConfig = Self::setup_build_config(json_build_config);
        Workspace::new(Some(work_dir), ws_config, build_config)
    }
}