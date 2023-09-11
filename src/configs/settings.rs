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

use serde_json::Value;

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
        // TODO we should define how to handle the version of the file
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

#[cfg(test)]
mod tests {
    use crate::configs::Settings;

    fn helper_settings_from_str(json_test_str: &str) -> Settings {
        let result: Result<Settings, serde_json::Error> = Settings::from_str(json_test_str);
        let settings: Settings;
        match result {
            Ok(rsettings) => {
                settings = rsettings;
            }
            Err(e) => {
                eprintln!("Error parsing JSON: {}", e);
                panic!();
            } 
        }
        settings
    }

    #[test]
    fn test_settings_configs_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "configsdir": "configs_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_configs_dir(),  "configs_test");
    }

    #[test]
    fn test_settings_no_configs_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "artifactsdir": "artifacts_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_configs_dir(),  "configs");
    }

    #[test]
    fn test_settings_no_configs_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_configs_dir(),  "configs");
    }

    #[test]
    fn test_settings_builds_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "buildsdir": "builds_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_builds_dir(),  "builds_test");
    }

    #[test]
    fn test_settings_no_builds_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "artifactsdir": "artifacts_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_builds_dir(),  "builds");
    }

    #[test]
    fn test_settings_no_builds_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_builds_dir(),  "builds");
    }

    #[test]
    fn test_settings_artifacts_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "artifactsdir": "artifacts_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_artifacts_dir(), "artifacts_test");
    }

    #[test]
    fn test_settings_no_artifacts_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "buildsdir": "builds_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_artifacts_dir(), "artifacts");
    }

    #[test]
    fn test_settings_no_artifacts_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_artifacts_dir(), "artifacts");
    }

    #[test]
    fn test_settings_scripts_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "scriptsdir": "scripts_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_scripts_dir(), "scripts_test");
    }

    #[test]
    fn test_settings_no_scripts_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "buildsdir": "builds_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_scripts_dir(), "scripts");
    }

    #[test]
    fn test_settings_no_scripts_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_scripts_dir(), "scripts");
    }

    #[test]
    fn test_settings_docker_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "dockerdir": "docker_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_docker_dir(), "docker_test");
    }

    #[test]
    fn test_settings_no_docker_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "buildsdir": "builds_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_docker_dir(), "docker");
    }

    #[test]
    fn test_settings_no_docker_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_docker_dir(), "docker");
    }

    #[test]
    fn test_settings_cache_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "cachedir": "cache_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_cache_dir(), "cache_test");
    }

    #[test]
    fn test_settings_no_cache_dir() {
        let json_test_str = r#"
        {
            "version": 4,
            "workspace": {
              "buildsdir": "builds_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_cache_dir(), "cache");
    }

    #[test]
    fn test_settings_no_cache_workspace_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_cache_dir(), "cache");
    }

    #[test]
    fn test_settings_docker_image() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "image": "test-workspace"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_image(), "test-workspace");
    }

    #[test]
    fn test_settings_no_docker_image() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "tag": "0.1"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_image(), "bakery-workspace");
    }

    #[test]
    fn test_settings_no_docker_image_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_image(), "bakery-workspace");
    }

    #[test]
    fn test_settings_docker_tag() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "tag": "0.1"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_tag(), "0.1");
    }

    #[test]
    fn test_settings_no_docker_tag() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "image": "test-workspace"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_tag(),  "0.68");
    }

    #[test]
    fn test_settings_no_docker_tag_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_tag(), "0.68");
    }

    #[test]
    fn test_settings_docker_registry() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "registry": "test-registry"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_registry(), "test-registry");
    }

    #[test]
    fn test_settings_no_docker_registry() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "image": "test-workspace"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_registry(), "strixos");
    }

    #[test]
    fn test_settings_no_docker_registry_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_registry(), "strixos");
    }

    #[test]
    fn test_settings_docker_args() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "args": [
                  "--rm=true",
                  "-t",
                  "--dns=8.8.8.8"
                ]
              }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_args(), [String::from("--rm=true"), String::from("-t"), String::from("--dns=8.8.8.8")]);
    }

    #[test]
    fn test_settings_no_docker_args() {
        let json_test_str = r#"
        {
            "version": 4,
            "docker": {
                "image": "test-workspace"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_args(), [String::from("--rm=true"), String::from("-t")]);
    }

    #[test]
    fn test_settings_no_docker_args_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);  
        assert_eq!(settings.docker_args(), [String::from("--rm=true"), String::from("-t")]);
    }

    #[test]
    fn test_settings_build_configs() {
        let json_test_str = r#"
        {
            "version": 4,
            "build": {
                "supported": [
                  "machine1-test",
                  "machine2-test"
                ]
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.supported_build_configs(),  vec![String::from("machine1-test"), String::from("machine2-test")]);
        let mut i: i32 = 1;
        for supported in settings.supported_build_configs() {
            assert_eq!(supported, format!("machine{}-test", i));
            i += 1;
        }
    }

    #[test]
    fn test_settings_no_supported_build_configs() {
        let json_test_str = r#"
        {
            "version": 4,
            "build": {
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.supported_build_configs().is_empty(), true);
    }

    #[test]
    fn test_settings_no_build_node() {
        let json_test_str = r#"
        {
            "version": 4
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.supported_build_configs().is_empty(), true);
    }
}

