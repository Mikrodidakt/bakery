use std::path::{PathBuf, Path};
use std::env;
use std::io::Error;

use crate::workspace::Settings;
use crate::configs::SettingsConfig;
pub struct Workspace {
    pub work_dir: PathBuf,
    settings: Settings,
}

impl<'a> Workspace {
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

    pub fn new(workdir: Option<PathBuf>, config: SettingsConfig) -> Self {
        let work_dir: PathBuf = Self::setup_work_directory(&workdir);
        let settings: Settings = Settings::new(work_dir.clone(), config);
        Workspace {
            work_dir,
            settings,
        }
    }

    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::workspace::Settings;
    use crate::configs::{SettingsConfig, BuildConfig};
    use crate::error::BError;

    use super::Workspace;

    fn helper_setup_ws_config(json_test_str: &str) -> SettingsConfig {
        let result: Result<SettingsConfig, BError> = SettingsConfig::from_str(json_test_str);
        let settings: SettingsConfig;
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

    //fn helper_setup_workspace(json_settings, json_build_config, work_dir)

    #[test]
    fn test_workspace() {
        let test_work_dir: String = String::from("/test_work_dir");
        let json_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let _json_build_config: &str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {}
        }"#;
        let work_dir: PathBuf = PathBuf::from(test_work_dir);
        let ws_config: SettingsConfig = helper_setup_ws_config(json_settings);
        //let build_config: BuildConfig = helper_setup_build_config(json_build_config);
        let ws: Workspace = Workspace::new(Some(work_dir), ws_config);
        let settings: &Settings = ws.get_settings();
        assert_eq!(settings.builds_dir(), PathBuf::from("/test_work_dir/builds"));
    }
}