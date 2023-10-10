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
use crate::configs::{Config, Context};
use serde_json::Value;
use crate::error::BError;

use super::ArtifactConfig;

#[derive(Clone, PartialEq, Debug)]
pub enum TType {
    Bitbake,
    NonBitbake,
}

pub struct TaskConfig {
    pub index: String,
    pub name: String,
    pub ttype: TType, // Optional if not set for the task the default type 'bitbake' is used
    pub disabled: String, // Optional if not set for the task the default value 'false' is used
    pub builddir: String,
    pub build: String,
    pub docker: String,
    pub condition: String,
    pub clean: String,
    pub recipes: Vec<String>, // The list of recipes will be empty if the type for the task is 'non-bitbake'
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
                    return Err(BError::ParseArtifactsError(format!("Invalid 'artifacts' node in build config")));
                } else {
                    return Err(BError::ParseArtifactsError(format!("Node is not an array")));
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
        let docker: String = Self::get_str_value("docker", data, Some(String::from("")))?;
        let condition: String = Self::get_str_value("condition", data, Some(String::from("true")))?;
        let build: String = Self::get_str_value("build", &data, Some(String::from("")))?;
        let clean: String = Self::get_str_value("clean", &data, Some(String::from("")))?;
        let recipes: Vec<String> = Self::get_array_value("recipes", &data, Some(vec![]))?;

        let enum_ttype: TType;
        match ttype.as_str() {
            "bitbake" => {
                enum_ttype = TType::Bitbake;
            },
            "non-bitbake" => {
                enum_ttype = TType::NonBitbake;
            },
            _ => {
                return Err(BError::ParseTasksError(format!("Invalid type '{}'", ttype)));
            },
        }

        // if the task type is bitbake then at least one recipe is required
        if recipes.is_empty() && ttype == "bitbake" {
            return Err(BError::ParseTasksError(format!("The 'bitbake' type requires at least one entry in 'recipes'")));
        }

        Ok(TaskConfig {
            index,
            name,
            ttype: enum_ttype,
            disabled,
            docker,
            condition,
            builddir,
            build,
            clean,
            recipes,
        })
    }
    
    pub fn expand_ctx(&mut self, ctx: &Context) {
        self.builddir = ctx.expand_str(&self.builddir);
        self.build = ctx.expand_str(&self.build);
        self.clean = ctx.expand_str(&self.clean);
        self.condition = ctx.expand_str(&self.condition);
        self.recipes.iter_mut().for_each(|r: &mut String| *r = ctx.expand_str(r));
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::{TaskConfig, TType, Context};
    use crate::error::BError;
    use crate::helper::Helper;

    use indexmap::{IndexMap, indexmap};

    #[test]
    fn test_task_config_none_bb() {
        let json_test_str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "disabled": "false",
            "type": "non-bitbake",
            "builddir": "test/builddir",
            "docker": "test-registry/test-image:0.1",
            "build": "build-cmd",
            "clean": "clean-cmd"
        }
        "#;
        let config = Helper::setup_task_config(json_test_str);
        assert_eq!(config.index, "0");
        assert_eq!(config.name, "task1-name");
        assert_eq!(config.disabled, "false");
        assert_eq!(config.ttype, TType::NonBitbake);
        assert_eq!(config.builddir, "test/builddir");
        assert_eq!(config.build, "build-cmd");
        assert_eq!(config.clean, "clean-cmd");
        assert_eq!(config.docker, "test-registry/test-image:0.1");
        assert_eq!(config.condition, "true");
    }

    #[test]
    fn test_task_config_bb() {
        let json_test_str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "disabled": "false",
            "condition": "false",
            "type": "bitbake",
            "recipes": [
                "test-image",
                "test-image:do_populate_sdk"
            ]
        }
        "#;
        let config = Helper::setup_task_config(json_test_str);
        assert_eq!(config.index, "0");
        assert_eq!(config.name, "task1-name");
        assert_eq!(config.disabled, "false");
        assert_eq!(config.condition, "false");
        assert_eq!(config.ttype, TType::Bitbake);
        assert_eq!(&config.recipes, &vec![String::from("test-image"), String::from("test-image:do_populate_sdk")]);
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
        let config = Helper::setup_task_config(json_test_str);
        assert_eq!(config.index, "0");
        assert_eq!(config.name, "task1-name");
        assert_eq!(config.disabled, "false");
        assert_eq!(config.ttype, TType::Bitbake);
        assert_eq!(&config.recipes, &vec![String::from("test-image")]);
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
        let config: TaskConfig = Helper::setup_task_config(json_test_str);
        assert_eq!(config.index, "0");
        assert_eq!(config.name, "task1-name");
        assert_eq!(config.disabled, "false");
        assert_eq!(config.ttype, TType::Bitbake);
        assert_eq!(&config.recipes, &vec![String::from("test-image")]);
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
                assert_eq!(e.to_string(), String::from("Invalid 'task' node in build config. The 'bitbake' type requires at least one entry in 'recipes'"));
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
                assert_eq!(e.to_string(), String::from("Invalid 'task' node in build config. Invalid type 'invalid'"));
            }
        }
    }

    #[test]
    fn test_task_config_expand_context() {
        let variables: IndexMap<String, String> = indexmap! {
            "TEST_RECIPE".to_string() => "test-image".to_string(),
            "TEST_ARCHIVE".to_string() => "test.zip".to_string(),
            "TEST_DIR".to_string() => "dir-name".to_string(),
            "TEST_FILE".to_string() => "file1.txt".to_string()
        };
        let ctx: Context = Context::new(&variables);
        let json_test_str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "recipes": [
                "${TEST_RECIPE}"
            ],
            "artifacts": [
                {
                    "type": "archive",
                    "name": "${TEST_ARCHIVE}",
                    "artifacts": [
                        {
                            "type": "directory",
                            "name": "${TEST_DIR}",
                            "artifacts": [
                                {
                                    "source": "${TEST_FILE}"
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
        let mut config = Helper::setup_task_config(json_test_str);
        config.expand_ctx(&ctx);
        assert_eq!(config.index, "0");
        assert_eq!(config.name, "task1-name");
        assert_eq!(config.disabled, "false");
        assert_eq!(config.ttype, TType::Bitbake);
        assert_eq!(&config.recipes, &vec![String::from("test-image")]);
    }
}