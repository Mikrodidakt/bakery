use std::path::PathBuf;
use std::env;
use std::io::Error;

use crate::workspace::{WsSettingsHandler, WsConfigHandler};
use crate::configs::{WsSettings, BuildConfig, Context};
pub struct Workspace {
    settings: WsSettingsHandler,
    config: WsConfigHandler,
}

impl Workspace {
    fn setup_work_directory(workdir: &Option<PathBuf>) -> PathBuf {
        let mut work_dir: PathBuf = PathBuf::new();
        match workdir {
            Some(w_dir) => {
                work_dir.join(w_dir)
            },
            None => {
                let path: Result<PathBuf, Error> = env::current_dir();
                match path {
                    Ok(w_dir) => {
                        work_dir.join(w_dir)
                    },
                    Err(_e) => {
                        panic!("{}", _e.to_string());
                    }
                }
            } 
        }
    }

    pub fn new(workdir: Option<PathBuf>, ws_config: WsSettings, build_config: BuildConfig) -> Self {
        let work_dir: PathBuf = Self::setup_work_directory(&workdir);
        let settings: WsSettingsHandler = WsSettingsHandler::new(work_dir.clone(), ws_config);
        let config: WsConfigHandler = WsConfigHandler::new(&settings, build_config);

        Workspace {
            settings,
            config,
        }
    }

    pub fn settings(&self) -> &WsSettingsHandler {
        &self.settings
    }

    pub fn config(&self) -> &WsConfigHandler {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::configs::{WsSettings, BuildConfig};
    use crate::executers::DockerImage;
    use crate::error::BError;

    use super::Workspace;

    fn helper_setup_ws_config(json_test_str: &str) -> WsSettings {
        let result: Result<WsSettings, BError> = WsSettings::from_str(json_test_str);
        let settings: WsSettings;
        match result {
            Ok(rsettings) => {
                settings = rsettings;
            }
            Err(e) => {
                eprintln!("Error parsing workspace settings: {}", e);
                panic!();
            } 
        }
        settings
    }

    fn helper_setup_build_config(json_test_str: &str) -> BuildConfig {
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

    fn helper_setup_workspace(test_work_dir: &str, json_settings: &str, json_build_config: &str) -> Workspace {
        let work_dir: PathBuf = PathBuf::from(test_work_dir);
        let ws_config: WsSettings = helper_setup_ws_config(json_settings);
        let build_config: BuildConfig = helper_setup_build_config(json_build_config);
        Workspace::new(Some(work_dir), ws_config, build_config)
    }

    #[test]
    fn test_workspace_default_settings() {
        let test_work_dir: &str = "/test_work_dir";
        let json_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_build_config: &str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {}
        }"#;
        let ws: Workspace = helper_setup_workspace(test_work_dir, json_settings, json_build_config);
        assert_eq!(ws.settings().builds_dir(), PathBuf::from("/test_work_dir/builds"));
        assert_eq!(ws.settings().cache_dir(), PathBuf::from("/test_work_dir/.cache"));
        assert_eq!(ws.settings().artifacts_dir(), PathBuf::from("/test_work_dir/artifacts"));
        assert_eq!(ws.settings().scripts_dir(), PathBuf::from("/test_work_dir/scripts"));
        assert_eq!(ws.settings().docker_dir(), PathBuf::from("/test_work_dir/docker"));
        assert_eq!(ws.settings().configs_dir(), PathBuf::from("/test_work_dir/configs"));
        assert_eq!(ws.settings().include_dir(), PathBuf::from("/test_work_dir/configs/include"));
        let docker_image: &DockerImage = ws.settings().docker_image();
        assert_eq!(format!("{}", docker_image), "strixos/bakery-workspace:0.68");
        assert_eq!(ws.settings().docker_args(), &vec!["--rm=true".to_string(), "-t".to_string()]);
        assert!(ws.settings().supported_builds().is_empty());
    }
}