use serde_json::Value;
use crate::error::BError;
use crate::configs::Config;

pub struct WsProductData {
    name: String,
    arch: String,
    description: String,
}

impl Config for WsProductData {}

impl WsProductData {
    pub fn from_str(json_string: &str) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data)
    }

    pub fn from_value(data: &Value) -> Result<Self, BError> {
        let name: String = Self::get_str_value("name", &data, Some(String::from("NA")))?;
        let description: String = Self::get_str_value("description", &data, Some(String::from("NA")))?;
        let arch: String = Self::get_str_value("arch", &data, Some(String::from("NA")))?;

        Ok(WsProductData {
            name,
            arch,
            description,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arch(&self) -> &str {
        &self.arch
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}


#[cfg(test)]
mod tests {
    use crate::data::WsProductData;

    #[test]
    fn test_ws_product_data_default() {
        let json_default_build_config = r#"
        {
            "version": "4"
        }"#;
        let data: WsProductData = WsProductData::from_str(json_default_build_config).expect("Failed to parse product data");
        assert_eq!(data.name(), "NA");
        assert_eq!(data.description(), "NA");
        assert_eq!(data.arch(), "NA");
    }

    #[test]
    fn test_ws_product_data() {
        let json_default_build_config = r#"
        {
            "version": "4",
            "name": "test-name",
            "description": "test description",
            "arch": "test-arch"
        }"#;
        let data: WsProductData = WsProductData::from_str(json_default_build_config).expect("Failed to parse product data");
        assert_eq!(data.name(), "test-name");
        assert_eq!(data.description(), "test description");
        assert_eq!(data.arch(), "test-arch");
    }
}
