use std::fs::DirEntry;
use std::path::PathBuf;
use std::env;
use std::io::Error;
use indexmap::IndexMap;

use crate::fs::JsonFileReader;
use crate::workspace::{WsSettingsHandler, WsBuildConfigHandler, WsTaskConfigHandler};
use crate::configs::{WsSettings, BuildConfig, settings};
use crate::error::BError;

pub struct Workspace {
    settings: WsSettingsHandler,
    config: WsBuildConfigHandler,
    //configs: IndexMap<PathBuf, String>,
}

impl Workspace {
    fn setup_work_directory(work_dir: &Option<PathBuf>) -> PathBuf {
        let path_buf: PathBuf = PathBuf::new();
        match work_dir {
            Some(w_dir) => {
                path_buf.join(w_dir)
            },
            None => {
                let path: Result<PathBuf, Error> = env::current_dir();
                match path {
                    Ok(w_dir) => {
                        path_buf.join(w_dir)
                    },
                    Err(_e) => {
                        panic!("{}", _e.to_string());
                    }
                }
            } 
        }
    }

    fn setup_settings(work_dir: PathBuf, settings: Option<WsSettings>, default: &mut bool) -> WsSettingsHandler {
        match settings {
            Some(ws_settings) => {
                *default = false;
                WsSettingsHandler::new(work_dir.clone(), ws_settings)
            },
            None => {
                // If settings is not supplied we use default
                // settings by supplying a basic settings json
                // string. This string should be valid and never fail
                // so we don't care about error because we assume it
                // will be fine. This should be added as part of one
                // the tests.
                let default_settings: &str  = r#"
                {
                    "version": "4",
                    "builds": {
                        "supported": [
                            "default"
                        ]
                    }
                }"#;
                *default = true;
                WsSettingsHandler::from_str(work_dir.to_str().unwrap(), default_settings).unwrap()
            }
        }
    }

    fn setup_config(settings: &WsSettingsHandler, config: Option<BuildConfig>, default: &mut bool) -> WsBuildConfigHandler {
        match config {
            Some(ws_config) => {
                *default = false;
                WsBuildConfigHandler::new(settings, ws_config)
            },
            None => {
                // If config is not supplied we use default
                // config by supplying a basic build config json
                // string. This string should be valid and never fail
                // so we don't care about error because we assume it
                // will be fine. This should be added as part of one
                // the tests.
                let default_config: &str = r#"
                {                                                                                                                   
                    "version": "4",
                    "name": "default",
                    "description": "Default build config",
                    "arch": "NA",
                    "bb": {}
                }"#;
                *default = true;
                WsBuildConfigHandler::from_str(settings, default_config).unwrap()
            }
        }
    }

    fn setup_list_of_available_configs(settings: &WsSettingsHandler, config: &WsBuildConfigHandler) -> Result<IndexMap<PathBuf, String>, BError> {
        let mut list_of_files: Vec<String> = Vec::new();
        if settings.supported_builds().is_empty() {
            // If the list of supported builds is empty in the settings
            // all build configs are supported
            let config_dir: std::fs::ReadDir = std::fs::read_dir(settings.configs_dir()).map_err(|err| BError {
                code: 1, // You may set the appropriate error code
                message: format!("Failed to read config dir: '{}'", err),
            })?;

            for entry in config_dir {
                let e: DirEntry = entry.map_err(|err| BError {
                    code: 1, // You may set the appropriate error code
                    message: format!("Failed read dir entry: '{}'", err),
                })?;
                let path: PathBuf = e.path();
        
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "json" {
                            if let Some(file_name) = path.file_name() {
                                if let Some(file_name_str) = file_name.to_str() {
                                    list_of_files.push(file_name_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // we could have used map() but in this case it make sense to use
            // for_each()
            settings.supported_builds().iter().for_each(|build| {
                list_of_files.push(format!("{}.json", build));
            });
        }

        let mut build_configs: IndexMap<PathBuf, String> = IndexMap::new();
        for f in list_of_files {
            let config_path: PathBuf = settings.configs_dir().join(f);
            let config_str: String = JsonFileReader::new(config_path.to_string_lossy().to_string()).read_json()?;
            let config: BuildConfig = BuildConfig::from_str(&config_str)?;
            build_configs.insert(config_path, config.description().to_string());
        }

        Ok(build_configs)
    }

    pub fn new(workdir: Option<PathBuf>, ws_config: Option<WsSettings>, build_config: Option<BuildConfig>) -> Result<Self, BError> {
        let mut default_settings: bool = false;
        let mut default_config: bool = false;
        let work_dir: PathBuf = Self::setup_work_directory(&workdir);
        let settings: WsSettingsHandler = Self::setup_settings(work_dir, ws_config, &mut default_settings);
        let config: WsBuildConfigHandler = Self::setup_config(&settings, build_config, &mut default_config);
        //let configs: IndexMap<PathBuf, String> = Self::setup_list_of_available_configs(&settings, &config)?;

        Ok(Workspace {
            settings,
            config,
            //configs,
        })
    }

    pub fn settings(&self) -> &WsSettingsHandler {
        &self.settings
    }

    pub fn config(&self) -> &WsBuildConfigHandler {
        &self.config
    }

    // Returns a dictionary including all build configurations names
    // and their description
    /*pub fn build_configs(&self) -> &IndexMap<PathBuf, String> {
        &self.configs
    }*/


    /*
    pub fn collect(&self, build: &str) -> bool {}

    pub fn extend_build_env(&self, variables: Vec<String>) {}

    pub fn set_recipes(&self, recipes: Vec<String>) {}

    pub fn bb_build_env(&self, build: Option<String>) {}
    
    // Returns a ordered list of the builds
    pub fn builds(&self) -> IndexSet<String> {}

    // Returns true if the config is part of the list
    // of build configs supported by the workspace
    pub fn valid_config(&self, config: &str) -> bool {}
    */
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::executers::DockerImage;
    use crate::helper::Helper;

    use super::Workspace;

    #[test]
    fn test_workspace_default() {
        let test_work_dir: &str = "/test_work_dir";
        let ws: Workspace = Workspace::new(
            Some(PathBuf::from(test_work_dir)),
            None, 
            None).expect("Failed to setup workspace");
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
        assert_eq!(ws.settings().supported_builds(), &vec!["default".to_string()]);
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
        let ws: Workspace = Helper::setup_ws(test_work_dir, json_settings, json_build_config);
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