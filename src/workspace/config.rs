use indexmap::{IndexMap, indexmap};
use std::path::PathBuf;


use crate::configs::{Context, BuildConfig};
use crate::workspace::WsSettingsHandler;

pub struct WsConfigHandler {
    ctx: Context,
    config: BuildConfig,
    work_dir: PathBuf,
}

impl WsConfigHandler {
    pub fn new(settings: &WsSettingsHandler, config: BuildConfig) -> Self {
        let ctx_variables: IndexMap<String, String> = indexmap! {
            "MACHINE".to_string() => config.bitbake().machine().to_string(),
            "ARCH".to_string() => config.arch().to_string(),
            "DISTRO".to_string() => config.bitbake().distro().to_string(),
            "VARIATN".to_string() => "".to_string(),
            "PRODUCT_NAME".to_string() => config.name().to_string(),
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
        ctx.update(config.context());

        WsConfigHandler {
            config,
            ctx,
            work_dir: settings.work_dir().clone(),
        }
    }    
}