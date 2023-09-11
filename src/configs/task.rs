/*
The Task config is a subset of the build config. For a full view of the build config please refere to build.rs in configs submodule.
The task config has the following format in the build config

{ 
    "image": {
        "index": "0",
        "name": "image",
        "recipes": [
            "rpi-image" 
        ],
        "artifacts": [   
            {
                "source": "${POKY_DEPLOY_DIR}/${MACHINE}/strix-image-${MACHINE}.rpi-sdimg"
            }
        ]
    },
    "sdk": {
        "index": "1",
        "name": "sdk",
        "disabled": "true",
        "recipes": [
            "rpi-image:do_populate_sdk"
        ],
        "artifacts": [
            {
                "source": "${POKY_DEPLOY_DIR}/${MACHINE}/strix-sdk-${MACHINE}.sh"
            }
        ]
    }
}
*/
use crate::configs::Config;
use serde_json::Value;
use crate::error::BError;

use super::ArtifactConfig;

pub struct TaskConfig {
    index: String,
    name: String,
    ttype: String, // Optional if not set for the task the default type 'bitbake' is used
    disabled: String, // Optional if not set for the task the default value 'false' is used
    builddir: String,
    build: String,
    clean: String,
    recipes: Vec<String>, // The list of recipes will be empty if the type for the task is 'non-bitbake'
    artifacts: Vec<ArtifactConfig>, // For some tasks there might not be any artifacts to collect then this will be empty
}

impl Config for TaskConfig {
}

impl TaskConfig {
    fn get_artifacts(data: &Value) -> Result<Vec<ArtifactConfig>, BError> {
        match data.get("artifacts") {
            Some(value) => {
                if value.is_array() {
                    if let Some(artifact_vec) = value.as_array() {
                        let mut artifacts: Vec<ArtifactConfig> = Vec::new();
                        for artifact_data in artifact_vec.iter() {
                            let artifact: ArtifactConfig = ArtifactConfig::from_value(&artifact_data)?;
                            artifacts.push(artifact);
                        }
                        return Ok(artifacts);
                    }
                    return Err(BError{ code: 0, message: format!("Invalid 'artifacts' format in build config")});
                } else {
                    return Err(BError{ code: 0, message: format!("Invalid 'artifacts' format in build config")});
                }
            }
            None => {
                return Ok(Vec::new());
            }
        }
    }

    pub fn from_str(json_string: &str) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data)
    }

    pub fn from_value(data: &Value) -> Result<Self, BError> {
        let index: String = Self::get_str_value("index", &data, None)?;
        let name: String = Self::get_str_value("name", &data, None)?;
        let ttype: String = Self::get_str_value("type", &data, Some(String::from("bitbake")))?;
        let disabled: String = Self::get_str_value("disabled", &data, Some(String::from("false")))?;
        let builddir: String = Self::get_str_value("builddir", &data, Some(String::from("")))?;
        let build: String = Self::get_str_value("build", &data, Some(String::from("")))?;
        let clean: String = Self::get_str_value("clean", &data, Some(String::from("")))?;
        let recipes: Vec<String> = Self::get_array_value("recipes", &data, Some(vec![]))?;
        let artifacts: Vec<ArtifactConfig> = Self::get_artifacts(&data)?;
        if ttype != "bitbake" && ttype != "non-bitbake" {
            return Err(BError{ code: 0, message: format!("Invalid 'artifact' format in build config. Invalid type '{}'", ttype)}); 
        }
        // if the task type is bitbake then at least one recipe is required
        if recipes.is_empty() && ttype == "bitbake" {
            return Err(BError{ code: 0, message: format!("Invalid 'task' format in build config. The 'bitbake' type requires at least one entry in 'recipes'")});
        }
        Ok(TaskConfig {
            index,
            name,
            ttype,
            disabled,
            builddir,
            build,
            clean,
            recipes,
            artifacts,
        })
    }
    
    pub fn index(&self) -> &String {
        &self.index
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn ttype(&self) -> &String {
        &self.ttype
    }

    pub fn disabled(&self) -> &String {
        &self.disabled
    }

    pub fn builddir(&self) -> &String {
        &self.builddir
    }

    pub fn build(&self) -> &String {
        &self.build
    }

    pub fn clean(&self) -> &String {
        &self.clean
    }

    pub fn recipes(&self) -> &Vec<String> {
        &self.recipes
    }

    pub fn artifacts(&self) -> &Vec<ArtifactConfig> {
        // TODO: we should most likely change this so that artifacts is a struct just like
        // we have done with the TaskConfig struct we should setup a ArtifactsConfig and
        // have this method return a &HashMap<String, ArtifactsConfig>
        &self.artifacts
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::TaskConfig;
    use crate::error::BError;

    fn helper_task_config_from_str(json_test_str: &str) -> TaskConfig {
        let result: Result<TaskConfig, BError> = TaskConfig::from_str(json_test_str);
        match result {
            Ok(rconfig) => {
                rconfig
            }
            Err(e) => {
                eprintln!("Error parsing tasks from build config: {}", e);
                panic!();
            } 
        }
    }

    #[test]
    fn test_task_config_none_bb() {
        let json_test_str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "disabled": "false",
            "type": "non-bitbake",
            "builddir": "test/builddir",
            "build": "build-cmd",
            "clean": "clean-cmd"
        }
        "#;
        let config = helper_task_config_from_str(json_test_str);
        assert_eq!(config.index(), "0");
        assert_eq!(config.name(), "task1-name");
        assert_eq!(config.disabled(), "false");
        assert_eq!(config.ttype(), "non-bitbake");
        assert_eq!(config.builddir(), "test/builddir");
        assert_eq!(config.build(), "build-cmd");
        assert_eq!(config.clean(), "clean-cmd");
    }

    #[test]
    fn test_task_config_bb() {
        let json_test_str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "disabled": "false",
            "type": "bitbake",
            "recipes": [
                "test-image",
                "test-image:do_populate_sdk"
            ]
        }
        "#;
        let config = helper_task_config_from_str(json_test_str);
        assert_eq!(config.index(), "0");
        assert_eq!(config.name(), "task1-name");
        assert_eq!(config.disabled(), "false");
        assert_eq!(config.ttype(), "bitbake");
        assert_eq!(config.recipes(), &vec![String::from("test-image"), String::from("test-image:do_populate_sdk")]);
    }

    #[test]
    fn test_task_config_optional() {
        let json_test_str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "recipes": [
                "test-image"
            ]
        }
        "#;
        let config = helper_task_config_from_str(json_test_str);
        assert_eq!(config.index(), "0");
        assert_eq!(config.name(), "task1-name");
        assert_eq!(config.disabled(), "false");
        assert_eq!(config.ttype(), "bitbake");
        assert_eq!(config.recipes(), &vec![String::from("test-image")]);
    }

    #[test]
    fn test_task_config_artifacts() {
        let json_test_str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "recipes": [
                "test-image"
            ],
            "artifacts": [
                {
                    "type": "archive",
                    "name": "test.zip",
                    "artifacts": [
                        {
                            "type": "directory",
                            "name": "dir-name",
                            "artifacts": [
                                {
                                    "source": "file1.txt"
                                },
                                {
                                    "source": "file2.txt"
                                }
                            ]
                        }
                    ]
                }
            ]
        }
        "#;
        let config = helper_task_config_from_str(json_test_str);
        assert_eq!(config.index(), "0");
        assert_eq!(config.name(), "task1-name");
        assert_eq!(config.disabled(), "false");
        assert_eq!(config.ttype(), "bitbake");
        assert_eq!(config.recipes(), &vec![String::from("test-image")]);
        assert!(!config.artifacts().is_empty());
        let artifacts = config.artifacts();
        assert_eq!(artifacts[0].ttype(), "archive");
        assert_eq!(artifacts[0].name(), "test.zip");
        let dir_artifacts = artifacts[0].artifacts();
        assert_eq!(dir_artifacts[0].ttype(), "directory");
        assert_eq!(dir_artifacts[0].name(), "dir-name");
        assert!(!dir_artifacts[0].artifacts().is_empty());
        let files = dir_artifacts[0].artifacts();
        let mut i = 1;
        for f in files.iter() {
            assert_eq!(f.ttype(), "file");
            assert!(f.name().is_empty());
            assert_eq!(f.source(), &format!("file{}.txt", i));
            assert!(f.dest().is_empty());
            i += 1;
        }

    }

    #[test]
    fn test_task_config_error_no_recipes() {
        let json_test_str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "type": "bitbake"
        }
        "#;
        let result: Result<TaskConfig, BError> = TaskConfig::from_str(json_test_str);
        match result {
            Ok(_rconfig) => {
                panic!("We should have recived an error because we have no recipes defined!");
            }
            Err(e) => {
                assert_eq!(e.message, String::from("Invalid 'task' format in build config. The 'bitbake' type requires at least one entry in 'recipes'"));
            } 
        }
    }

    #[test]
    fn test_task_config_error_invalid_type() {
        let json_test_str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "type": "invalid"
        }
        "#;
        let result: Result<TaskConfig, BError> = TaskConfig::from_str(json_test_str);
        match result {
            Ok(_rconfig) => {
                panic!("We should have recived an error because we have no recipes defined!");
            }
            Err(e) => {
                assert_eq!(e.message, String::from("Invalid 'artifact' format in build config. Invalid type 'invalid'"));
            } 
        }
    }
}