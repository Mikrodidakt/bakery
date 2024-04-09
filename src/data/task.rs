use indexmap::IndexMap;
use serde_json::Value;
use std::path::PathBuf;

use crate::configs::Context;
use crate::error::BError;
use crate::data::WsBuildData;
use crate::configs::Config;

#[derive(Clone, PartialEq, Debug)]
pub enum TType {
    Bitbake,
    NonBitbake,
}

pub struct WsTaskData {
    index: u32,
    name: String,
    ttype: TType, // Optional if not set for the task the default type 'bitbake' is used
    disabled: String, // Optional if not set for the task the default value 'false' is used
    build_dir: PathBuf,
    build: String,
    docker: String,
    condition: String,
    clean: String,
    recipes: Vec<String>, // The list of recipes will be empty if the type for the task is 'non-bitbake'
    description: String,
    env: IndexMap<String, String>,
}

impl Config for WsTaskData {
}

impl WsTaskData {
    fn determine_build_dir(ttype: TType, task_build_dir: &str, bb_build_dir: &PathBuf, work_dir: &PathBuf) -> PathBuf {
        if ttype == TType::Bitbake {
            if task_build_dir.is_empty() {
                return bb_build_dir.clone();
            }
        }

        work_dir.join(PathBuf::from(task_build_dir))
    }

    pub fn from_str(json_string: &str, build_data: &WsBuildData) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data, build_data)
    }

    pub fn from_value(data: &Value, build_data: &WsBuildData) -> Result<Self, BError> {
        Self::new(data,
            &build_data.settings().work_dir(),
            &build_data.bitbake().build_dir()
        )
    }

    pub fn new(data: &Value, work_dir: &PathBuf, bb_build_dir: &PathBuf) -> Result<Self, BError> {
        let index: u32 = Self::get_u32_value("index", &data, None)?;
        let name: String = Self::get_str_value("name", &data, None)?;
        let ttype: String = Self::get_str_value("type", &data, Some(String::from("bitbake")))?;
        let disabled: String = Self::get_str_value("disabled", &data, Some(String::from("false")))?;
        let build_dir: String = Self::get_str_value("builddir", &data, Some(String::from("")))?;
        let docker: String = Self::get_str_value("docker", data, Some(String::from("")))?;
        let condition: String = Self::get_str_value("condition", data, Some(String::from("true")))?;
        let build: String = Self::get_str_value("build", &data, Some(String::from("")))?;
        let clean: String = Self::get_str_value("clean", &data, Some(String::from("")))?;
        let description: String = Self::get_str_value("description", &data, Some(String::from("NA")))?;
        let env: IndexMap<String, String> = Self::get_hashmap_value("env", &data)?;
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

        let task_build_dir: PathBuf = Self::determine_build_dir(enum_ttype.clone(), &build_dir, bb_build_dir, work_dir);

        // if the task type is bitbake then at least one recipe is required
        if recipes.is_empty() && ttype == "bitbake" {
            return Err(BError::ParseTasksError(format!("The 'bitbake' type requires at least one entry in 'recipes'")));
        }

        Ok(WsTaskData {
            index,
            name,
            ttype: enum_ttype,
            disabled,
            docker,
            condition,
            build_dir: task_build_dir,
            build,
            clean,
            recipes,
            description,
            env,
        })
    }

    pub fn expand_ctx(&mut self, ctx: &Context) -> Result<(), BError>{
        self.name = ctx.expand_str(&self.name)?;
        self.build_dir = ctx.expand_path(&self.build_dir)?;
        self.build = ctx.expand_str(&self.build)?;
        self.clean = ctx.expand_str(&self.clean)?;
        self.condition = ctx.expand_str(&self.condition)?;
        for r in self.recipes.iter_mut() {
            *r = ctx.expand_str(r)?;
        }
        for (_key, value) in self.env.iter_mut() {
            *value = ctx.expand_str(value)?;
        }
        Ok(())
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ttype(&self) -> &TType {
        &self.ttype
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn disabled(&self) -> bool {
        if self.disabled == "true" {
            return true;
        }
        return false;
    }

    pub fn docker_image(&self) -> &str {
        &self.docker
    }

    pub fn condition(&self) -> bool {
        let condition: &str = &self.condition;

        if condition.is_empty() {
            return true;
        }

        match condition {
            "1" | "yes" | "y" | "Y" | "true" | "YES" | "TRUE" | "True" | "Yes" => return true,
            _ => return false,
        }
    }

    pub fn build_dir(&self) -> &PathBuf {
        &self.build_dir
    }

    pub fn build_cmd(&self) -> &str {
        &self.build
    }

    pub fn clean_cmd(&self) -> &str {
        &self.clean
    }

    pub fn recipes(&self) -> &Vec<String> {
        &self.recipes
    }

    pub fn env(&self) -> &IndexMap<String, String> {
        &self.env
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use serde_json::Value;
    use indexmap::{IndexMap, indexmap};

    use crate::error::BError;
    use crate::helper::Helper;
    use crate::data::{
        WsTaskData,
        TType,
    };
    use crate::configs::Context;

    #[test]
    fn test_ws_task_data_nonbitbake() {
        let json_task_config: &str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "description": "test",
            "disabled": "false",
            "type": "non-bitbake",
            "builddir": "test/builddir",
            "docker": "test-registry/test-image:0.1",
            "build": "build-cmd",
            "clean": "clean-cmd"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let bb_build_dir: PathBuf = work_dir.clone().join(String::from("test/builddir"));
        let data: Value = Helper::parse(json_task_config).expect("Failed to parse task config");
        let task: WsTaskData = WsTaskData::new(&data, &work_dir, &bb_build_dir).expect("Failed parsing task data");
        assert_eq!(task.index(), 0);
        assert_eq!(task.name(), "task1-name");
        assert_eq!(task.disabled(), false);
        assert_eq!(task.condition(), true);
        assert_eq!(task.description(), "test");
        assert_eq!(task.ttype(), &TType::NonBitbake);
        assert_eq!(task.build_dir(), &PathBuf::from("/workspace/test/builddir"));
        assert_eq!(task.build_cmd(), "build-cmd");
        assert_eq!(task.clean_cmd(), "clean-cmd");
        assert_eq!(task.docker_image(), "test-registry/test-image:0.1");
    }

    #[test]
    fn test_ws_task_data_bitbake() {
        let json_task_config: &str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "recipes": [
                "test-image",
                "test-image:sdk"
            ]
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let bb_build_dir: PathBuf = work_dir.clone().join(String::from("builds/test-name"));
        let data: Value = Helper::parse(json_task_config).expect("Failed to parse task config");
        let task: WsTaskData = WsTaskData::new(&data, &work_dir, &bb_build_dir).expect("Failed parsing task data");
        assert_eq!(task.index(), 0);
        assert_eq!(task.name(), "task1-name");
        assert_eq!(task.disabled(), false);
        assert_eq!(task.condition(), true);
        assert_eq!(task.ttype(), &TType::Bitbake);
        assert_eq!(task.build_dir(), &PathBuf::from("/workspace/builds/test-name"));
        assert_eq!(task.build_cmd(), "");
        assert_eq!(task.clean_cmd(), "");
        assert_eq!(task.docker_image(), "");
        assert_eq!(task.description(), "NA");
        assert_eq!(task.recipes(), &vec![String::from("test-image"), String::from("test-image:sdk")]);
    }

    #[test]
    fn test_ws_task_data_context() {
        let json_task_config: &str = r#"
        {
            "index": "2",
            "name": "$#[TASK_NAME]",
            "recipes": [
                "$#[IMAGE_RECIPE]",
                "$#[IMAGE_RECIPE_SDK]"
            ]
        }"#;
        let ctx_variables: IndexMap<String, String> = indexmap! {
            "TASK_NAME".to_string() => "task1-name".to_string(),
            "IMAGE_RECIPE".to_string() => "test-image".to_string(),
            "IMAGE_RECIPE_SDK".to_string() => "test-image:sdk".to_string(),
        };
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let bb_build_dir: PathBuf = work_dir.clone().join(String::from("builds/test-name"));
        let context: Context = Context::new(&ctx_variables);
        let data: Value = Helper::parse(json_task_config).expect("Failed to parse task config");
        let mut task: WsTaskData = WsTaskData::new(&data, &work_dir, &bb_build_dir).expect("Failed parsing task data");
        task.expand_ctx(&context).unwrap();
        assert_eq!(task.index(), 2);
        assert_eq!(task.name(), "task1-name");
        assert_eq!(task.disabled(), false);
        assert_eq!(task.condition(), true);
        assert_eq!(task.ttype(), &TType::Bitbake);
        assert_eq!(task.build_dir(), &PathBuf::from("/workspace/builds/test-name"));
        assert_eq!(task.build_cmd(), "");
        assert_eq!(task.clean_cmd(), "");
        assert_eq!(task.docker_image(), "");
        assert_eq!(task.recipes(), &vec![String::from("test-image"), String::from("test-image:sdk")]);
    }

    #[test]
    fn test_ws_task_data_error_no_recipes() {
        let json_task_config: &str = r#"
        {
            "index": "0",
            "name": "task1-name"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let bb_build_dir: PathBuf = work_dir.clone().join(String::from("test/builddir"));
        let data: Value = Helper::parse(json_task_config).expect("Failed to parse task config");
        let result: Result<WsTaskData, BError> = WsTaskData::new(&data, &work_dir, &bb_build_dir);
        match result {
            Ok(_data) => {
                panic!("We should have recived an error because we have no recipes defined!");
            }
            Err(e) => {
                assert_eq!(e.to_string(), String::from("Invalid 'task' node in build config. The 'bitbake' type requires at least one entry in 'recipes'"));
            }
        }
    }

    #[test]
    fn test_ws_task_data_error_invalid_type() {
        let json_task_config: &str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "type": "invalid"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let bb_build_dir: PathBuf = work_dir.clone().join(String::from("test/builddir"));
        let data: Value = Helper::parse(json_task_config).expect("Failed to parse task config");
        let result: Result<WsTaskData, BError> = WsTaskData::new(&data, &work_dir, &bb_build_dir);
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
    fn test_ws_task_env() {
        let json_task_config: &str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "type": "non-bitbake",
            "builddir": "test/builddir",
            "env": [
                "KEY1=VALUE1",
                "KEY2=VALUE2",
                "KEY3=VALUE3"
            ],
            "build": "build-cmd",
            "clean": "clean-cmd"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let bb_build_dir: PathBuf = work_dir.clone().join(String::from("test/builddir"));
        let data: Value = Helper::parse(json_task_config).expect("Failed to parse task config");
        let task: WsTaskData = WsTaskData::new(&data, &work_dir, &bb_build_dir).expect("Failed parsing task data");
        let task_env: &IndexMap<String, String> = task.env();
        let mut i: usize = 1;
        task_env.iter().for_each(|(key, value)| {
            assert_eq!(key, &format!("KEY{}", i));
            assert_eq!(value, &format!("VALUE{}", i));
            i += 1;
        });
    }

    #[test]
    fn test_ws_task_env_context() {
        let json_task_config: &str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "type": "non-bitbake",
            "builddir": "test/builddir",
            "env": [
                "KEY1=$#[CTX_VALUE1]",
                "KEY2=$#[CTX_VALUE2]",
                "KEY3=VALUE3"
            ],
            "build": "build-cmd",
            "clean": "clean-cmd"
        }"#;
        let ctx_variables: IndexMap<String, String> = indexmap! {
            "CTX_VALUE1".to_string() => "ctx-value1".to_string(),
            "CTX_VALUE2".to_string() => "$#[CTX_VALUE3]-value2".to_string(),
            "CTX_VALUE3".to_string() => "ctx".to_string(),
        };
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let bb_build_dir: PathBuf = work_dir.clone().join(String::from("builds/test-name"));
        let context: Context = Context::new(&ctx_variables);
        let data: Value = Helper::parse(json_task_config).expect("Failed to parse task config");
        let mut task: WsTaskData = WsTaskData::new(&data, &work_dir, &bb_build_dir).expect("Failed parsing task data");
        task.expand_ctx(&context).unwrap();
        assert_eq!(task.env(), &indexmap! {
            "KEY1".to_string() => "ctx-value1".to_string(),
            "KEY2".to_string() => "ctx-value2".to_string(),
            "KEY3".to_string() => "VALUE3".to_string(),
        });
    }
}
