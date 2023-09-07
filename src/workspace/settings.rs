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

    pub fn workspace_dir(&self) -> String {
        return String::from("work_dir");
    }

    fn get_ws_dir(&self, dir: &str) -> String {
        let work_dir = self.workspace_dir();
        // TODO: we should consider to maybe have the default cache dir as .cache
        let key = format!("{}dir", dir);
        let default_value = format!("{}/{}", work_dir, dir);
        if !self.workspace.is_null() {
            if self.workspace[key.clone()].is_null() {
                return default_value;
            }
            return format!("{}/{}", work_dir, self.workspace[key.clone()].as_str().unwrap());
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
}

