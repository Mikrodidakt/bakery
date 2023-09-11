use crate::configs::Config;
use serde_json::Value;
use crate::error::BError;

pub struct TaskConfig {
    index: String,
    name: String,
    ttype: String, // Optional if not set for the task the default type 'bitbake' is used
    disabled: String, // Optional if not set for the task the default value 'false' is used
    builddir: String,
    build: String,
    clean: String,
    recipes: Vec<String>, // The list of recipes will be empty if the type for the task is 'non-bitbake'
    artifacts: Value, // For some tasks there might not be any artifacts to collect then this will be empty
}

impl Config for TaskConfig {
}

impl TaskConfig {
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
        let artifacts: &Value = Self::get_value("artifacts", &data)?;
        Ok(TaskConfig {
            index,
            name,
            ttype,
            disabled,
            builddir,
            build,
            clean,
            recipes,
            artifacts: artifacts.clone(),
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

    pub fn artifacts(&self) -> &Value {
        // TODO: we should most likely change this so that artifacts is a struct just like
        // we have done with the TaskConfig struct we should setup a ArtifactsConfig and
        // have this method return a &HashMap<String, ArtifactsConfig>
        &self.artifacts
    }
}