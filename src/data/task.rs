use indexmap::IndexMap;
use serde_json::Value;
use std::path::PathBuf;

use crate::configs::Config;
use crate::configs::Context;
use crate::data::WsBuildData;
use crate::error::BError;

#[derive(Clone, PartialEq, Debug)]
pub enum TType {
    NonHLOS,
    QSSI,
    VENDOR,
    KERNEL,
}

pub struct WsTaskData {
    index: u32,
    name: String,
    ttype: TType, // Optional if not set for the task the default type 'hlos' is used
    disabled: String, // Optional if not set for the task the default value 'false' is used
    build_dir: PathBuf,
    init_env: PathBuf,
    build: String,
    docker: String,
    condition: String,
    clean: String,
    description: String,
    env: IndexMap<String, String>,
}

impl Config for WsTaskData {}

impl WsTaskData {
    pub fn from_str(json_string: &str, build_data: &WsBuildData) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data, build_data)
    }

    pub fn from_value(data: &Value, build_data: &WsBuildData) -> Result<Self, BError> {
        Self::new(
            data,
            &build_data.settings().work_dir(),
        )
    }

    pub fn new(data: &Value, work_dir: &PathBuf) -> Result<Self, BError> {
        let index: u32 = Self::get_u32_value("index", &data, None)?;
        let name: String = Self::get_str_value("name", &data, None)?;
        let ttype: String = Self::get_str_value("type", &data, Some(String::from("vendor")))?;
        let disabled: String = Self::get_str_value("disabled", &data, Some(String::from("false")))?;
        let build_dir: String = Self::get_str_value("builddir", &data, Some(String::from("")))?;
        let docker: String = Self::get_str_value("docker", data, Some(String::from("")))?;
        let condition: String = Self::get_str_value("condition", data, Some(String::from("true")))?;
        let build: String = Self::get_str_value("build", &data, Some(String::from("")))?;
        let clean: String = Self::get_str_value("clean", &data, Some(String::from("")))?;
        let init_env: String = Self::get_str_value("initenv", &data, Some(String::from("")))?;
        let description: String =
            Self::get_str_value("description", &data, Some(String::from("NA")))?;
        let env: IndexMap<String, String> = Self::get_hashmap_value("env", &data)?;

        let enum_ttype: TType;
        match ttype.as_str() {
            "non-hlos" => {
                enum_ttype = TType::NonHLOS;
            }
            "kernel" => {
                enum_ttype = TType::KERNEL;
            }
            "vendor" => {
                enum_ttype = TType::VENDOR;
            }
            "qssi" => {
                enum_ttype = TType::QSSI;
            }
            _ => {
                return Err(BError::ParseTasksError(format!("Invalid type '{}'", ttype)));
            }
        }

        let build_dir_path: PathBuf = work_dir.clone().join(PathBuf::from(build_dir));
        let init_env_path: PathBuf = build_dir_path.clone().join(PathBuf::from(init_env));

        Ok(WsTaskData {
            index,
            name,
            ttype: enum_ttype,
            disabled,
            docker,
            condition,
            build_dir: build_dir_path,
            build,
            clean,
            description,
            init_env: init_env_path,
            env,
        })
    }

    pub fn expand_ctx(&mut self, ctx: &Context) -> Result<(), BError> {
        self.name = ctx.expand_str(&self.name)?;
        self.build_dir = ctx.expand_path(&self.build_dir)?;
        self.init_env = ctx.expand_path(&self.init_env)?;
        self.build = ctx.expand_str(&self.build)?;
        self.clean = ctx.expand_str(&self.clean)?;
        self.condition = ctx.expand_str(&self.condition)?;
        self.disabled = ctx.expand_str(&self.disabled)?;
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

    pub fn init_env(&self) -> &PathBuf {
        &self.init_env
    }

    pub fn build_cmd(&self) -> &str {
        &self.build
    }

    pub fn clean_cmd(&self) -> &str {
        &self.clean
    }

    pub fn env(&self) -> &IndexMap<String, String> {
        &self.env
    }
}

/*
#[cfg(test)]
mod tests {
    use indexmap::{indexmap, IndexMap};
    use serde_json::Value;
    use std::path::PathBuf;

    use crate::configs::Context;
    use crate::data::{TType, WsTaskData};
    use crate::error::BError;
    use crate::helper::Helper;

    #[test]
    fn test_ws_task_data_nonhlos() {
        let json_task_config: &str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "description": "test",
            "disabled": "false",
            "type": "non-hlos",
            "builddir": "test/builddir",
            "docker": "test-registry/test-image:0.1",
            "build": "build-cmd",
            "clean": "clean-cmd"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let bb_build_dir: PathBuf = work_dir.clone().join(String::from("test/builddir"));
        let data: Value = Helper::parse(json_task_config).expect("Failed to parse task config");
        let task: WsTaskData =
            WsTaskData::new(&data, &work_dir).expect("Failed parsing task data");
        assert_eq!(task.index(), 0);
        assert_eq!(task.name(), "task1-name");
        assert_eq!(task.disabled(), false);
        assert_eq!(task.condition(), true);
        assert_eq!(task.description(), "test");
        assert_eq!(task.ttype(), &TType::NonHLOS);
        assert_eq!(task.build_dir(), &PathBuf::from("/workspace/test/builddir"));
        assert_eq!(task.build_cmd(), "build-cmd");
        assert_eq!(task.clean_cmd(), "clean-cmd");
        assert_eq!(task.docker_image(), "test-registry/test-image:0.1");
    }

    #[test]
    fn test_ws_task_data_context() {
        let json_task_config: &str = r#"
        {
            "index": "0",
            "name": "$#[TASK_NAME]",
            "description": "test",
            "disabled": "false",
            "type": "qssi",
            "builddir": "test/builddir",
            "docker": "test-registry/test-image:0.1",
            "build": "$#[BUILD_CMD] $#[BUILD_CMD_ARG]",
            "clean": "clean-cmd"
        }"#;
        let ctx_variables: IndexMap<String, String> = indexmap! {
            "TASK_NAME".to_string() => "task1-name".to_string(),
            "BUILD_CMD".to_string() => "test-cmd".to_string(),
            "BUILD_CMD_ARG".to_string() => "test-cmd-arg".to_string(),
        };
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let context: Context = Context::new(&ctx_variables);
        let data: Value = Helper::parse(json_task_config).expect("Failed to parse task config");
        let mut task: WsTaskData =
            WsTaskData::new(&data, &work_dir, &bb_build_dir).expect("Failed parsing task data");
        task.expand_ctx(&context).unwrap();
        assert_eq!(task.index(), 2);
        assert_eq!(task.name(), "task1-name");
        assert_eq!(task.disabled(), false);
        assert_eq!(task.condition(), true);
        assert_eq!(task.ttype(), &TType::Bitbake);
        assert_eq!(
            task.build_dir(),
            &PathBuf::from("/workspace/builds/test-name")
        );
        assert_eq!(task.build_cmd(), "");
        assert_eq!(task.clean_cmd(), "");
        assert_eq!(task.docker_image(), "");
        assert_eq!(
            task.recipes(),
            &vec![String::from("test-image"), String::from("test-image:sdk")]
        );
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
                assert_eq!(
                    e.to_string(),
                    String::from("Invalid 'task' node in build config. Invalid type 'invalid'")
                );
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
        let task: WsTaskData =
            WsTaskData::new(&data, &work_dir, &bb_build_dir).expect("Failed parsing task data");
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
        let mut task: WsTaskData =
            WsTaskData::new(&data, &work_dir, &bb_build_dir).expect("Failed parsing task data");
        task.expand_ctx(&context).unwrap();
        assert_eq!(
            task.env(),
            &indexmap! {
                "KEY1".to_string() => "ctx-value1".to_string(),
                "KEY2".to_string() => "ctx-value2".to_string(),
                "KEY3".to_string() => "VALUE3".to_string(),
            }
        );
    }
}
*/