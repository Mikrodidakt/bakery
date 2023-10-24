use indexmap::{indexmap, IndexMap};
use serde_json::Value;
use std::path::PathBuf;

use crate::configs::Context;
use crate::error::BError;
use crate::configs::Config;

pub struct WsContextData {
    context: Context,
}

impl Config for WsContextData {}

impl WsContextData {
    pub fn from_str(json_string: &str) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data)
    }

    pub fn from_value(data: &Value) -> Result<Self, BError> {
        let variables: IndexMap<String, String> = Self::get_hashmap_value("context", &data)?;
        let ctx_default_variables: IndexMap<String, String> = indexmap! {
            "MACHINE".to_string() => "NA".to_string(),
            "ARCH".to_string() => "NA".to_string(),
            "DISTRO".to_string() => "NA".to_string(),
            "VARIANT".to_string() => "NA".to_string(),
            "PRODUCT_NAME".to_string() => "NA".to_string(),
            "BB_BUILD_DIR".to_string() => "".to_string(),
            "BB_DEPLOY_DIR".to_string() => "".to_string(),
            "ARTIFACTS_DIR".to_string() => "".to_string(),
            "BUILDS_DIR".to_string() => "".to_string(),
            "WORK_DIR".to_string() => "".to_string(),
            "PLATFORM_VERSION".to_string() => "0.0.0".to_string(),
            "BUILD_NUMBER".to_string() => "0".to_string(),
            "PLATFORM_RELEASE".to_string() => "0.0.0-0".to_string(), // We should combine the PLATFORM_VERSION with the BUILD_NUMBER
            "BUILD_SHA".to_string() => "dev".to_string(), // If no git sha is specified and it is built locally then this is the default
            "RELEASE_BUILD".to_string() => "0".to_string(),
            "BUILD_VARIANT".to_string() => "dev".to_string(), // The variant can be dev, release and manufacturing
            "ARCHIVER".to_string() => "0".to_string(),
            "DEBUG_SYMBOLS".to_string() => "0".to_string(), // This can be used if you need to collect debug symbols from a build and have specific task defined for it
        };
        let mut ctx: Context = Context::new(&ctx_default_variables);
        ctx.update(&variables);
        Ok(WsContextData {
            context: ctx,
        })
    }

    /*
    pub fn variables(&self) -> &IndexMap<String, String> {
        &self.variables
    }
    */
    pub fn ctx(&self) -> &Context {
        &self.context
    }

    pub fn update(&mut self, variables: &IndexMap<String, String>) {
        self.context.update(variables);
    }

    pub fn update_ctx(&mut self, context: &Context) {
        self.update(context.variables());
    }

    pub fn get_ctx_path(&self, key: &str) -> PathBuf {
        PathBuf::from(self.get_ctx_value(key))
    }

    pub fn get_ctx_value(&self, key: &str) -> String {
        self.context.value(key)
    }
}

#[cfg(test)]
mod tests {
    use indexmap::{indexmap, IndexMap};
    use std::path::PathBuf;

    use crate::workspace::WsSettingsHandler;
    use crate::data::WsContextData;

    #[test]
    fn test_ws_context_data_default() {
        let json_default_build_config = r#"
        {                                                                                                                   
            "version": "4"
        }"#;
        let data: WsContextData = WsContextData::from_str(json_default_build_config).expect("Failed to parse context data");
        assert_eq!(data.get_ctx_value("MACHINE"), "NA");
        assert_eq!(data.get_ctx_value("ARCH"), "NA");
        assert_eq!(data.get_ctx_value("DISTRO"), "NA");
        assert_eq!(data.get_ctx_value("VARIANT"), "NA");
        assert_eq!(data.get_ctx_value("PRODUCT_NAME"), "NA");
        assert_eq!(
            data.get_ctx_path("BB_BUILD_DIR"),
            PathBuf::from("")
        );
        assert_eq!(
            data.get_ctx_path("BB_DEPLOY_DIR"),
            PathBuf::from("")
        );
        assert_eq!(
            data.get_ctx_path("ARTIFACTS_DIR"),
            PathBuf::from("")
        );
        assert_eq!(
            data.get_ctx_path("BUILDS_DIR"),
            PathBuf::from("")
        );
        assert_eq!(data.get_ctx_path("WORK_DIR"), PathBuf::from(""));
        assert_eq!(data.get_ctx_value("PLATFORM_VERSION"), "0.0.0");
        assert_eq!(data.get_ctx_value("BUILD_NUMBER"), "0");
        assert_eq!(data.get_ctx_value("PLATFORM_RELEASE"), "0.0.0-0");
        assert_eq!(data.get_ctx_value("BUILD_SHA"), "dev");
        assert_eq!(data.get_ctx_value("RELEASE_BUILD"), "0");
        assert_eq!(data.get_ctx_value("BUILD_VARIANT"), "dev");
        assert_eq!(data.get_ctx_value("ARCHIVER"), "0");
        assert_eq!(data.get_ctx_value("DEBUG_SYMBOLS"), "0");
    }

    #[test]
    fn test_ws_context_data_overwrite() {
        let json_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {                                                                                                                   
            "version": "4",
            "context": [
                "KEY1=value1",
                "KEY2=value2",
                "KEY3=value3" 
            ]
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings).expect("Failed to parse settings");
        let mut data: WsContextData = WsContextData::from_str(json_build_config).expect("Failed to parse context data");
        let ctx_built_in_variables: IndexMap<String, String> = indexmap! {
            "MACHINE".to_string() => "test-machine".to_string(),
            "ARCH".to_string() => "test-arch".to_string(),
            "DISTRO".to_string() => "test-distro".to_string(),
            "VARIANT".to_string() => "test-variant".to_string(),
            "PRODUCT_NAME".to_string() => "test".to_string(),
            "WORK_DIR".to_string() => settings.work_dir().to_string_lossy().to_string(),
        };
        data.update(&ctx_built_in_variables);
        assert_eq!(data.get_ctx_value("MACHINE"), "test-machine");
        assert_eq!(data.get_ctx_value("ARCH"), "test-arch");
        assert_eq!(data.get_ctx_value("DISTRO"), "test-distro");
        assert_eq!(data.get_ctx_value("VARIANT"), "test-variant");
        assert_eq!(data.get_ctx_value("PRODUCT_NAME"), "test");
        assert_eq!(data.get_ctx_value("KEY1"), "value1");
        assert_eq!(data.get_ctx_value("KEY2"), "value2");
        assert_eq!(data.get_ctx_value("KEY3"), "value3");
        assert_eq!(data.get_ctx_path("WORK_DIR"), settings.work_dir());

    }
}
