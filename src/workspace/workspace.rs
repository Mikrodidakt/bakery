use std::path::PathBuf;
use std::env;
use std::io::Error;

use crate::workspace::{WsSettingsHandler, WsBuildConfigHandler, WsTaskConfigHandler};
use crate::configs::{WsSettings, BuildConfig};

pub struct Workspace {
    settings: WsSettingsHandler,
    config: WsBuildConfigHandler,
}

impl Workspace {
    fn setup_work_directory(workdir: &Option<PathBuf>) -> PathBuf {
        let work_dir: PathBuf = PathBuf::new();
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
        let config: WsBuildConfigHandler = WsBuildConfigHandler::new(&settings, build_config);

        Workspace {
            settings,
            config,
        }
    }

    pub fn settings(&self) -> &WsSettingsHandler {
        &self.settings
    }

    pub fn config(&self) -> &WsBuildConfigHandler {
        &self.config
    }

    /*
    pub fn collect(&self, build: &str) -> bool {}

    pub fn extend_build_env(&self, variables: Vec<String>) {}

    pub fn set_recipes(&self, recipes: Vec<String>) {}

    pub fn bb_build_env(&self, build: Option<String>) {}
    
    // Returns a dictionary including all build configurations names
    // and there description.
    pub fn build_configs(&self) -> IndexMap<String, String> {}

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