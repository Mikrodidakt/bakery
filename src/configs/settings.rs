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
            "builds": {
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
use crate::configs::Config;
use crate::error::BError;

// Not the ideal solution we should see if it is possible to
// read them from the Cargo.toml and then incorporate them
// into the binary and read them out in runtime.
pub const BAKERY_DOCKER_ARGS: [&str; 2] = ["--rm=true", "-t"];                                                                          
pub const BAKERY_DOCKER_IMAGE: &str = "bakery-workspace";                                                                           
pub const BAKERY_DOCKER_TAG: &str = "0.68";                                                                                      
pub const BAKERY_DOCKER_REGISTRY: &str = "strixos";

pub struct Settings {
    version: String,
    configsdir: String,
    buildsdir: String,
    artifactsdir: String,
    scriptsdir: String,
    dockerdir: String,
    cachedir: String,
    supported: Vec<String>,
    docker_tag: String,
    docker_image: String,
    docker_registry: String,
    docker_args: Vec<String>,
}

impl Config for Settings {
}

impl Settings {
    pub fn from_str(json_string: &str) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        let version: String = Self::get_str_value("version", &data, None)?;
        let mut configsdir: String = String::from("configs");
        let mut buildsdir: String = String::from("builds");
        let mut artifactsdir: String = String::from("artifacts");
        let mut scriptsdir: String = String::from("scripts");
        let mut dockerdir: String = String::from("docker");
        let mut cachedir: String = String::from(".cache");
        match Self::get_value("workspace", &data) {
            Ok(ws_data) => { 
                configsdir = Self::get_str_value("configsdir", ws_data, Some(String::from("configs")))?;
                buildsdir = Self::get_str_value("buildsdir", ws_data, Some(String::from("builds")))?;
                artifactsdir = Self::get_str_value("artifactsdir", ws_data, Some(String::from("artifacts")))?;
             
                scriptsdir = Self::get_str_value("scriptsdir", ws_data, Some(String::from("scripts")))?;
                dockerdir = Self::get_str_value("dockerdir", ws_data, Some(String::from("docker")))?;
                cachedir = Self::get_str_value("cachedir", ws_data, Some(String::from(".cache")))?;
            },
            Err(_err) => {}
        }
        let supported: Vec<String>;
        match Self::get_value("builds", &data) {
            Ok(build_data) => {
                supported = Self::get_array_value("supported", build_data, Some(vec![]))?;
            },
            Err(_err) => {
                supported = vec![];
            }
        }
        let mut docker_image: String = String::from(BAKERY_DOCKER_IMAGE);
        let mut docker_tag: String = String::from(BAKERY_DOCKER_TAG);
        let mut docker_registry: String = String::from(BAKERY_DOCKER_REGISTRY);
        let mut docker_args: Vec<String> = vec![String::from("--rm=true"), String::from("-t")];
        match Self::get_value("docker", &data) {
            Ok(docker_data) => {
                docker_image = Self::get_str_value("image", docker_data, Some(String::from(BAKERY_DOCKER_IMAGE)))?;
                docker_tag = Self::get_str_value("tag", docker_data, Some(String::from(BAKERY_DOCKER_TAG)))?;
                docker_registry = Self::get_str_value("registry", docker_data, Some(String::from(BAKERY_DOCKER_REGISTRY)))?;
                docker_args = Self::get_array_value("args", docker_data, Some(vec![String::from("--rm=true"), String::from("-t")]))?;
            },
            Err(_err) => {}
        }
        Ok(Settings {
            version,
            configsdir,
            buildsdir,
            artifactsdir,
            scriptsdir,
            dockerdir,
            cachedir,
            supported,
            docker_tag,
            docker_image,
            docker_registry,
            docker_args,
        })
    }

    pub fn workspace_configs_dir(&self) -> &String {
        &self.configsdir
    }

    pub fn workspace_builds_dir(&self) -> &String {
        &self.buildsdir
    }

    pub fn workspace_artifacts_dir(&self) -> &String {
        &self.artifactsdir
    }

    pub fn workspace_scripts_dir(&self) -> &String {
        &self.scriptsdir
    }

    pub fn workspace_docker_dir(&self) -> &String {
        &self.dockerdir
    }

    pub fn workspace_cache_dir(&self) -> &String {
        &self.cachedir
    }

    pub fn docker_image(&self) -> &String {
        &self.docker_image
    }

    pub fn docker_tag(&self) -> &String {
        &self.docker_tag
    }

    pub fn docker_registry(&self) -> &String {
        &self.docker_registry
    }

    pub fn docker_args(&self) -> &Vec<String> {
        &self.docker_args
    }

    pub fn supported_build_configs(&self) -> &Vec<String> {
        &self.supported
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::Settings;
    use crate::error::BError;

    fn helper_settings_from_str(json_test_str: &str) -> Settings {
        let result: Result<Settings, BError> = Settings::from_str(json_test_str);
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
    fn test_settings_workspace_dirs() {
        let json_test_str = r#"
        {
            "version": "4",
            "workspace": {
              "configsdir": "configs_test",
              "artifactsdir": "artifacts_test",
              "buildsdir": "builds_test",
              "artifactsdir": "artifacts_test",
              "scriptsdir": "scripts_test",
              "dockerdir": "docker_test",
              "cachedir": "cache_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_configs_dir(),  "configs_test");
        assert_eq!(settings.workspace_artifacts_dir(),  "artifacts_test");
    }

    #[test]
    fn test_settings_no_configs_workspace_node() {
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_configs_dir(),  "configs");
    }

    #[test]
    fn test_settings_no_builds_dir() {
        let json_test_str = r#"
        {
            "version": "4",
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
            "version": "4"
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_builds_dir(),  "builds");
    }

    #[test]
    fn test_settings_no_artifacts_dir() {
        let json_test_str = r#"
        {
            "version": "4",
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
            "version": "4"
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_artifacts_dir(), "artifacts");
    }

    #[test]
    fn test_settings_no_scripts_dir() {
        let json_test_str = r#"
        {
            "version": "4",
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
            "version": "4"
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_scripts_dir(), "scripts");
    }

    #[test]
    fn test_settings_no_docker_dir() {
        let json_test_str = r#"
        {
            "version": "4",
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
            "version": "4"
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_docker_dir(), "docker");
    }

    #[test]
    fn test_settings_no_cache_dir() {
        let json_test_str = r#"
        {
            "version": "4",
            "workspace": {
              "buildsdir": "builds_test"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_cache_dir(), ".cache");
    }

    #[test]
    fn test_settings_no_cache_workspace_node() {
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.workspace_cache_dir(), ".cache");
    }

    #[test]
    fn test_settings_docker_image() {
        let json_test_str = r#"
        {
            "version": "4",
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
            "version": "4",
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
            "version": "4"
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_image(), "bakery-workspace");
    }

    #[test]
    fn test_settings_docker_tag() {
        let json_test_str = r#"
        {
            "version": "4",
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
            "version": "4",
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
            "version": "4"
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_tag(), "0.68");
    }

    #[test]
    fn test_settings_docker_registry() {
        let json_test_str = r#"
        {
            "version": "4",
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
            "version": "4",
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
            "version": "4"
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_registry(), "strixos");
    }

    #[test]
    fn test_settings_docker_args() {
        let json_test_str = r#"
        {
            "version": "4",
            "docker": {
                "args": [
                  "--rm=true",
                  "-t",
                  "--dns=8.8.8.8"
                ]
              }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_args(), &vec![String::from("--rm=true"), String::from("-t"), String::from("--dns=8.8.8.8")]);
    }

    #[test]
    fn test_settings_no_docker_args() {
        let json_test_str = r#"
        {
            "version": "4",
            "docker": {
                "image": "test-workspace"
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.docker_args(), &vec![String::from("--rm=true"), String::from("-t")]);
    }

    #[test]
    fn test_settings_no_docker_args_node() {
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let settings = helper_settings_from_str(json_test_str);  
        assert_eq!(settings.docker_args(), &vec![String::from("--rm=true"), String::from("-t")]);
    }

    #[test]
    fn test_settings_build_configs() {
        let json_test_str = r#"
        {
            "version": "4",
            "builds": {
                "supported": [
                  "machine1-test",
                  "machine2-test"
                ]
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.supported_build_configs(), &vec![String::from("machine1-test"), String::from("machine2-test")]);
        let mut i: i32 = 1;
        for supported in settings.supported_build_configs() {
            assert_eq!(supported, &format!("machine{}-test", i));
            i += 1;
        }
    }

    #[test]
    fn test_settings_no_supported_build_configs() {
        let json_test_str = r#"
        {
            "version": "4",
            "builds": {
            }
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.supported_build_configs().is_empty(), true);
    }

    #[test]
    fn test_settings_no_build_node() {
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let settings = helper_settings_from_str(json_test_str);
        assert_eq!(settings.supported_build_configs().is_empty(), true);
    }
}

