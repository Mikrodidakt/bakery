use serde_json::Value;
use std::path::PathBuf;

use crate::configs::Config;
use crate::error::BError;
use crate::workspace::WsSettingsHandler;

pub struct WsIncludeData {
    configs: Vec<PathBuf>,
}

impl Config for WsIncludeData {}

impl WsIncludeData {
    pub fn from_str(json_string: &str, settings: &WsSettingsHandler) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data, settings)
    }

    pub fn from_value(data: &Value, settings: &WsSettingsHandler) -> Result<Self, BError> {
        let configs: Vec<String> = Self::get_array_value("include", data, Some(vec![]))?;
        let paths: Vec<PathBuf> = configs
            .iter()
            .map(|config| {
                let path: PathBuf = settings
                    .include_dir()
                    .join(PathBuf::from(format!("{}.json", config)));
                Ok(path)
            })
            .collect::<Result<Vec<_>, BError>>()?;

        Ok(WsIncludeData { configs: paths })
    }

    pub fn configs(&self) -> &Vec<PathBuf> {
        &self.configs
    }
}
