use serde_json::Value;

use crate::configs::Config;
use crate::error::BError;

pub struct WsConfigData {
    version: String,
    name: String,
}

impl Config for WsConfigData {}

impl WsConfigData {
    pub fn from_str(json_string: &str) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data)
    }

    pub fn from_value(data: &Value) -> Result<Self, BError> {
        let version: String = Self::get_str_value("version", &data, None)?;
        // Duplication from WsProductData which is also keeping track of the name
        // for now leave it but should potentially move it
        let name: String = Self::get_str_value("name", &data, Some(String::from("NA")))?;

        Ok(WsConfigData { version, name })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

#[cfg(test)]
mod tests {
    use crate::data::WsConfigData;

    #[test]
    fn test_ws_config_data_default() {
        let json_build_config = r#"
        {
            "version": "5"
        }"#;
        let data: WsConfigData =
            WsConfigData::from_str(json_build_config).expect("Failed to parse config data");
        assert_eq!(data.version(), "5");
        assert_eq!(data.name(), "NA");
    }

    #[test]
    fn test_ws_config_data() {
        let json_build_config = r#"
        {
            "version": "5",
            "name": "test-name"
        }"#;
        let data: WsConfigData =
            WsConfigData::from_str(json_build_config).expect("Failed to parse config data");
        assert_eq!(data.version(), "5");
        assert_eq!(data.name(), "test-name");
    }
}
