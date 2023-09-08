use serde_json::Value;

/*
The settings file named settings.json or workspace.json should be placed
in the root of the workspace. The current format is
        {
            "version": 4,
            "workspace": {
              "configsdir": "configs_test", // Location for build configs default is ${BAKERY_WORKSPACE}/configs
              "buildsdir": "builds_test", // This is the bitbake build directory default is ${BAKERY_WORKSPACE}/builds
              "artifactsdir": "artifacts_test", // This is the location where bakery will place all artifacts defaul is ${BAKERY_WORKSPACE}/artifacts
              "scriptsdir": "scripts_test", // Each build component might expect to find scripts this default is ${BAKERY_WORKSPACE}/scripts
              "dockerdir": "scripts/docker_test", //
              "cachedir": ".cache_test" // This is the cache directory for sstate and dl dir for bitbake default is ${BAKERY_WORKSPACE}/cache
            },
            "build": {
              "supported": [
                "machine1-test",
                "machine2-test"
              ]
            },
            "docker": {
              "registry": "test",
              "image": "test-workspace",
              "tag": "0.1",
              "args": [
                "--rm=true",
                "-t",
                "--dns=8.8.8.8"
              ]
            }
          }
        }

The workspace node is for changing the default directories in the workspace. Only needs to be set
if the defaults should be changed. The build node is for defining what build configs are enabled
in the workspace if nothing is specified all build configs are available to use. The docker node
is for configuring the initial docker image that will be used to bootstrap the bakery. This is
the main docker image used so unless the build config contains specific docker images for a component
then this image will be used when building.
*/

// Not the ideal solution we should see if it is possible to
// read them from the Cargo.toml and then incorporate them
// into the binary and read them out in runtime.
pub const BAKERY_DOCKER_ARGS: [&str; 2] = ["--rm=true", "-t"];                                                                          
pub const BAKERY_DOCKER_IMAGE: &str = "bakery-workspace";                                                                           
pub const BAKERY_DOCKER_TAG: &str = "0.68";                                                                                      
pub const BAKERY_DOCKER_REGISTRY: &str = "strixos";

pub struct Settings {
    workspace: Value,
    docker: Value,
    build: Value,
}

impl Settings {
    pub fn from_str(json_string: &str) -> Result<Self, serde_json::Error> {
        let data: Value = serde_json::from_str(json_string)?;
        //let version: String = String::from(data["version"].as_str().unwrap());
        let workspace: Value = data["workspace"].clone();
        let docker: Value = data["docker"].clone();
        let build: Value = data["build"].clone();
        Ok(Settings { workspace, docker, build })
    }

    fn get_ws_dir(&self, dir: &str) -> String {
        // TODO: we should consider to maybe have the default cache dir as .cache currently it will be cache
        let key = format!("{}dir", dir);
        let default_value = String::from(dir);
        if !self.workspace.is_null() {
            if self.workspace[key.clone()].is_null() {
                return default_value;
            }
            return String::from(self.workspace[key.clone()].as_str().unwrap());
        }
        return default_value;    
    }

    pub fn workspace_configs_dir(&self) -> String {
        self.get_ws_dir("configs")
    }

    pub fn workspace_builds_dir(&self) -> String {
        self.get_ws_dir("builds")
    }

    pub fn workspace_artifacts_dir(&self) -> String {
        self.get_ws_dir("artifacts")
    }

    pub fn workspace_scripts_dir(&self) -> String {
        self.get_ws_dir("scripts")
    }

    pub fn workspace_docker_dir(&self) -> String {
        self.get_ws_dir("docker")
    }

    pub fn workspace_cache_dir(&self) -> String {
        self.get_ws_dir("cache")
    }

    pub fn docker_image(&self) -> String {
        let default_value = String::from(BAKERY_DOCKER_IMAGE);
        if !self.docker.is_null() {
            if self.docker["image"].is_null() {
                return default_value;
            }
            return String::from(self.docker["image"].as_str().unwrap());
        }
        return default_value; 
    }

    pub fn docker_tag(&self) -> String {
        let default_value = String::from(BAKERY_DOCKER_TAG);
        if !self.docker.is_null() {
            if self.docker["tag"].is_null() {
                return default_value;
            }
            return String::from(self.docker["tag"].as_str().unwrap());
        }
        return default_value; 
    }

    pub fn docker_registry(&self) -> String {
        let default_value = String::from(BAKERY_DOCKER_REGISTRY);
        if !self.docker.is_null() {
            if self.docker["registry"].is_null() {
                return default_value;
            }
            return String::from(self.docker["registry"].as_str().unwrap());
        }
        return default_value; 
    }

    pub fn docker_args(&self) -> Vec<String> {
        let default_value = BAKERY_DOCKER_ARGS.iter().map(|&s| s.to_string()).collect();
        if !self.docker.is_null() {
            if self.docker["args"].is_null() {
                return default_value;
            }

            let docker_args: Vec<String> = self.docker["args"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_owned())
                .collect();

            return docker_args;
        }
        return default_value;
    }

    pub fn supported_build_configs(&self) -> Vec<String> {
        let default_value = Vec::new();
        if !self.build.is_null() {
            if self.build["supported"].is_null() {
                return default_value;
            }

            let supported_values: Vec<String> = self.build["supported"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_owned())
                .collect();

            return supported_values;
        }
        return default_value;
    }
}

