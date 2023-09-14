use std::path::PathBuf;
use std::env;
use std::io::Error;
use indexmap::{IndexMap, indexmap};

use crate::workspace::WsSettingsHandler;
use crate::configs::{WsSettings, BuildConfig, Context};
pub struct Workspace {
    settings: WsSettingsHandler,
    build_config: BuildConfig,
    ctx: Context,
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

        let ctx_variables: IndexMap<String, String> = indexmap! {
            "MACHINE".to_string() => build_config.bitbake().machine().to_string(),
            "ARCH".to_string() => build_config.arch().to_string(),
            "DISTRO".to_string() => build_config.bitbake().distro().to_string(),
            "VARIATN".to_string() => "".to_string(),
            "PRODUCT_NAME".to_string() => build_config.name().to_string(),
            "BB_BUILD_DIR".to_string() => "".to_string(), // TODO: specify a default value
            "BB_DEPLOY_DIR".to_string() => "".to_string(), // TODO: specify a default value
            "ARTIFACTS_DIR".to_string() => settings.artifacts_dir().to_str().unwrap().to_string(),
            "WORK_DIR".to_string() => settings.work_dir().to_str().unwrap().to_string(),
            "PLATFORM_VERSION".to_string() => "0.0.0".to_string(),
            "BUILD_NUMBER".to_string() => "0".to_string(),                                                                                 
            "PLATFORM_RELEASE".to_string() => "0.0.0-0".to_string(),                                                                       
            "BUILD_SHA".to_string() => "dev".to_string(),                                                                                  
            "RELEASE_BUILD".to_string() => "0".to_string(),
            "BUILD_VARIANT".to_string() => "dev".to_string(),                                                                              
            "ARCHIVER".to_string() => "0".to_string(), 
            "DEBUG_SYMBOLS".to_string() => "0".to_string(),
        };
        let mut ctx: Context = Context::new(&ctx_variables);
        ctx.update(build_config.context());

        Workspace {
            settings,
            build_config,
            ctx,
        }
    }

    pub fn settings(&self) -> &WsSettingsHandler {
        &self.settings
    }

    //pub fn bbsettings(&self) -> &BBSettings {
    //    &self.bbsettings
    //}
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