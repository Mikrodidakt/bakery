use serde_json::Value;

use crate::error::BError;
use crate::configs::Config;
use crate::workspace::WsSettingsHandler;

pub struct WsIncludeData {
    includes: Vec<String>,
}

impl Config for WsIncludeData {}

impl WsIncludeData {
    pub fn from_str(json_string: &str, settings: &WsSettingsHandler) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data, settings)
    }

    pub fn from_value(data: &Value, settings: &WsSettingsHandler) -> Result<Self, BError> {
        let includes: Vec<String> = Self::get_array_value("includes", data, Some(vec![]))?;

        Ok(WsIncludeData {
            includes,
        })
    }
}
