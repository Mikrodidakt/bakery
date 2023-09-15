use indexmap::{IndexMap, indexmap};
use std::path::{PathBuf, Path};


use crate::configs::{Context, BuildConfig};
use crate::workspace::WsSettingsHandler;

pub struct WsConfigHandler {
    ctx: Context,
    config: BuildConfig,
    work_dir: PathBuf,
    build_dir: PathBuf,
    cache_dir: PathBuf,
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
            build_dir: settings.builds_dir().clone(),
            cache_dir: settings.cache_dir().clone(),
        }
    }

    /*
    pub fn build_dir(&self, build: &str) -> PathBuf {}

    pub fn extend_ctx(&self, ctx: &Context) {}

    pub recipes(&self, build: &str) -> Vec<String> {}

    // Returns true if the condition is 1, true, True, TRUE, yes, YES, Yes, Y
    // the condition is expanded using the context so a context variable could be used
    // as a condition.
    pub fn build_condition(&self, build: &str) -> bool {}

    pub fn build_enabled(&self, build: &str) -> bool {}

    // Returns the command defined for a specific build
    // it can either be a clean or build command
    pub fn task_cmd(&self, build: &str, task: &str) -> &str {}

    pub fn build_name(&self, build: &str) -> &str {}

    pub fn build_type(self, build: &str) -> &str {}

    pub fn artifacts(&self, build: &str) -> IndexMap<String, String> {}

    pub fn docker_image(&self, build: &str) -> DockerImage {}
    */

    pub fn product_name(&self) -> &str {
        // Currently the product name is the
        // same as config name but this might not
        // be the case in the future so therefore
        // I have added a specific getter
        self.config_name()
    }

    pub fn config_name(&self) -> &str {
        &self.config.name()
    }

    //pub fn config_enabled(&self) -> bool {
    //    self.config.enabled()
    //}

    pub fn version(&self) -> &str {
        &self.config.version()
    }

    pub fn arch(&self) -> &str {
        &self.config.arch()
    }

    pub fn bb_layers_conf(&self) -> &Vec<String> {
        &self.config.bitbake().bblayers_conf()
    }

    pub fn bb_local_conf(&self) -> Vec<String> {
        let mut local_conf: Vec<String> = self.config.bitbake().local_conf().clone();
        local_conf.push(format!("MACHINE ?= {}", self.bb_machine()));
        // TODO: we need to handle VARIANT correctly but this is good enough for now
        local_conf.push(format!("VARIANT ?= {}", "dev".to_string()));
        // TODO: we should define a method product_name() call that instead
        local_conf.push(format!("PRODUCT_NAME ?= {}", self.config.name()));
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
        &self.config.bitbake().machine()
    }

    //pub fn variant(&self) -> &str {
    //    self.config.variant()
    //}

    pub fn bb_distro(&self) -> &str {
        &self.config.bitbake().distro()
    }

    pub fn bb_build_dir(&self) -> PathBuf {
        let mut path_buf = self.build_dir.clone();
        path_buf.join(self.config.name())
    }

    pub fn bb_docker_image(&self) -> String {
        let docker: String = self.config.bitbake().docker().clone();
        if !docker.is_empty() {
            return self.ctx.expand_str(docker.as_str())
        }
        docker
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
        self.bb_build_dir().join(self.config.bitbake().deploy_dir())
    }    

    pub fn bb_sstate_dir(&self) -> PathBuf {
        let mut path_buf = self.cache_dir.clone();
        path_buf.join(self.config.arch()).join("sstate-cache".to_string())
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
        self.work_dir.join("layers".to_string()).join("poky".to_string())
    }

    pub fn oe_init_file(&self) -> PathBuf {
        // TODO: we should probably setup an option to configure what OE init script
        // to source to setup the env.
        self.poky_dir().join("oe-init-build-env")
    }   
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::workspace::{WsSettingsHandler, WsConfigHandler};
    use crate::helper::Helper;

    #[test]
    fn test_settings_default_ws_dirs() {
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let settings: WsSettingsHandler = WsSettingsHandler::new(
            work_dir,
            Helper::setup_ws_settings(json_test_str),
        );
        let json_test_str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {}
        }"#;
        let config = Helper::setup_build_config(json_test_str);
        let ws_config: WsConfigHandler = WsConfigHandler::new(&settings, config);
    }
}