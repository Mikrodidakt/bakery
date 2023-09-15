use indexmap::{IndexMap, indexmap};
use std::path::{PathBuf, Path};


use crate::configs::{Context, BuildConfig, TaskConfig};
use crate::workspace::WsSettingsHandler;
use crate::error::BError;

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
    pub fn task_type(self, task: &str) -> &str {
        return self.__config.builds[build].get("type", "bitbake")
    }

    pub fn build_dir(&self, task: &str) -> Result<PathBuf, BError> {
        match self.config.tasks().get(task) {
            Some(task) => {
                if task.ttype()
            },
            None => {

            }
        }
    }
 
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

    pub fn artifacts(&self, build: &str) -> IndexMap<String, String> {}

    pub fn docker_image(&self, build: &str) -> DockerImage {}
    */

    pub fn description(&self) -> &str {
        &self.config.description()
    }

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
        let mut path_buf = self.work_dir.clone();
        path_buf.join("layers".to_string()).join("poky".to_string())
    }

    pub fn oe_init_file(&self) -> PathBuf {
        // TODO: we should probably setup an option to configure what OE init script
        // to source to setup the env.
        self.poky_dir().join("oe-init-build-env")
    }   
}

#[cfg(test)]
mod tests {
    use std::path::{PathBuf, Path};

    use crate::workspace::{WsSettingsHandler, WsConfigHandler};
    use crate::helper::Helper;

    #[test]
    fn test_ws_config_default() {
        let json_settings = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {}
        }"#;
        let ws_config: WsConfigHandler = Helper::setup_ws_config_handler("/workspace", json_settings, json_build_config);
        assert_eq!(ws_config.version(), "4".to_string());
        assert_eq!(ws_config.arch(), "test-arch".to_string());
        assert_eq!(ws_config.description(), "Test Description".to_string());
        assert_eq!(ws_config.config_name(), "test-name".to_string());
        assert_eq!(ws_config.product_name(), "test-name".to_string());
        assert_eq!(ws_config.bb_distro(), "".to_string());
        assert_eq!(ws_config.bb_machine(), "".to_string());
        assert_eq!(ws_config.bb_build_dir(), PathBuf::from("/workspace/builds/test-name"));
        assert_eq!(ws_config.bb_build_config_dir(), PathBuf::from("/workspace/builds/test-name/conf"));
        assert_eq!(ws_config.bb_deploy_dir(), PathBuf::from("/workspace/builds/test-name/tmp/deploy/images"));
        assert_eq!(ws_config.bb_dl_dir(), PathBuf::from("/workspace/.cache/download"));
        assert_eq!(ws_config.bb_sstate_dir(), PathBuf::from("/workspace/.cache/test-arch/sstate-cache"));
        assert_eq!(ws_config.bb_layers_config(), PathBuf::from("/workspace/builds/test-name/conf/bblayers.conf"));
        assert!(ws_config.bb_layers_conf().is_empty());
        assert_eq!(ws_config.bb_local_config(), PathBuf::from("/workspace/builds/test-name/conf/local.conf"));
        assert!(!ws_config.bb_local_conf().is_empty());
        let local_conf: Vec<String> = vec![
            format!("MACHINE ?= {}", ws_config.bb_machine()),
            "VARIANT ?= dev".to_string(),
            format!("PRODUCT_NAME ?= {}", ws_config.product_name()),
            format!("DISTRO ?= {}", ws_config.bb_distro()),
            format!("SSTATE_DIR ?= {}", ws_config.bb_sstate_dir().to_str().unwrap()),
            format!("DL_DIR ?= {}", ws_config.bb_dl_dir().to_str().unwrap()),
            //format!("PLATFORM_VERSION ?= {}", ws_config.platform_version()),
            //format!("BUILD_NUMBER ?= {}", ws_config.build_number()),
            //format!("BUILD_SHA ?= {}", ws_config.build_sha()),
            //format!("RELASE_BUILD ?= {}", ws_config.release_build()),
            //format!("BUILD_VARIANT ?= {}", ws_config.build_variant()),
        ];
        assert_eq!(ws_config.bb_local_conf(), local_conf);
        assert_eq!(ws_config.bb_docker_image(), "".to_string());

    }

    #[test]
    fn test_ws_config_context_docker() {
        let json_settings = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "context": [
                "DOCKER_REGISTRY=test-registry",
                "DOCKER_TAG=0.1",
                "DOCKER_IMAGE=test-image"
            ],
            "bb": {
                "docker": "${DOCKER_REGISTRY}/${DOCKER_IMAGE}:${DOCKER_TAG}"
            }
        }"#;
        let ws_config: WsConfigHandler = Helper::setup_ws_config_handler("/workspace", json_settings, json_build_config);
        assert_eq!(ws_config.bb_docker_image(), "test-registry/test-image:0.1");
    }
}