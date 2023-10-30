use indexmap::{indexmap, IndexMap};
use serde_json::Value;
use std::path::PathBuf;

use crate::configs::Context;
use crate::error::BError;
use crate::configs::Config;

pub struct WsContextData {
    context: Context,
}

// Built in context variables
pub const CTX_KEY_MACHINE: &str = "MACHINE";
pub const CTX_KEY_ARCH: &str = "ARCH";
pub const CTX_KEY_DISTRO: &str = "DISTRO";
pub const CTX_KEY_BB_BUILD_DIR: &str = "BB_BUILD_DIR";
pub const CTX_KEY_BB_DEPLOY_DIR: &str = "BB_DEPLOY_DIR";
pub const CTX_KEY_PRODUCT_NAME: &str = "PRODUCT_NAME";
pub const CTX_KEY_ARTIFACTS_DIR: &str = "ARTIFACTS_DIR";
pub const CTX_KEY_BUILDS_DIR: &str = "BUILDS_DIR";
pub const CTX_KEY_WORK_DIR: &str = "WORK_DIR";
pub const CTX_KEY_PLATFORM_VERSION: &str = "PLATFORM_VERSION";
pub const CTX_KEY_BUILD_ID: &str = "BUILD_ID";
pub const CTX_KEY_PLATFORM_RELEASE: &str = "PLATFORM_RELEASE";
pub const CTX_KEY_BUILD_SHA: &str = "BUILD_SHA";
pub const CTX_KEY_VARIANT: &str = "VARIANT";
pub const CTX_KEY_RELEASE_BUILD: &str = "RELEASE_BUILD";
pub const CTX_KEY_ARCHIVER: &str = "ARCHIVER";
pub const CTX_KEY_DEBUG_SYMBOLS: &str = "DEBUG_SYMBOLS";

impl Config for WsContextData {}

impl WsContextData {
    pub fn mutable_key(key: &str) -> bool {
        match key {
            CTX_KEY_ARTIFACTS_DIR |
            CTX_KEY_PLATFORM_VERSION |
            CTX_KEY_BUILD_ID |
            CTX_KEY_PLATFORM_RELEASE |
            CTX_KEY_BUILD_SHA |
            CTX_KEY_VARIANT |
            CTX_KEY_RELEASE_BUILD |
            CTX_KEY_ARCHIVER |
            CTX_KEY_DEBUG_SYMBOLS => true,
            CTX_KEY_MACHINE |
            CTX_KEY_ARCH |
            CTX_KEY_DISTRO |
            CTX_KEY_BB_BUILD_DIR |
            CTX_KEY_BB_DEPLOY_DIR |
            CTX_KEY_PRODUCT_NAME |
            CTX_KEY_WORK_DIR |
            CTX_KEY_BUILDS_DIR => false,
            _ => false,
        }
    }

    pub fn from_str(json_string: &str) -> Result<Self, BError> {
        let data: Value = Self::parse(json_string)?;
        Self::from_value(&data)
    }

    pub fn from_value(data: &Value) -> Result<Self, BError> {
        let variables: IndexMap<String, String> = Self::get_hashmap_value("context", &data)?;
        Self::new(&variables)
    }

    pub fn new(variables: &IndexMap<String, String>) -> Result<Self, BError> {
        let ctx_default_variables: IndexMap<String, String> = indexmap! {
            CTX_KEY_MACHINE.to_string() => "NA".to_string(),
            CTX_KEY_ARCH.to_string() => "NA".to_string(),
            CTX_KEY_DISTRO.to_string() => "NA".to_string(),
            CTX_KEY_PRODUCT_NAME.to_string() => "NA".to_string(),
            CTX_KEY_BB_BUILD_DIR.to_string() => "".to_string(),
            CTX_KEY_BB_DEPLOY_DIR.to_string() => "".to_string(),
            CTX_KEY_ARTIFACTS_DIR.to_string() => "".to_string(),
            CTX_KEY_BUILDS_DIR.to_string() => "".to_string(),
            CTX_KEY_WORK_DIR.to_string() => "".to_string(),
            CTX_KEY_PLATFORM_VERSION.to_string() => "0.0.0".to_string(),
            CTX_KEY_BUILD_ID.to_string() => "0".to_string(),
            CTX_KEY_PLATFORM_RELEASE.to_string() => "0.0.0-0".to_string(), // We should combine the PLATFORM_VERSION with the BUILD_NUMBER
            CTX_KEY_BUILD_SHA.to_string() => "dev".to_string(), // If no git sha is specified and it is built locally then this is the default
            CTX_KEY_RELEASE_BUILD.to_string() => "0".to_string(),
            CTX_KEY_VARIANT.to_string() => "dev".to_string(), // The variant can be dev, release and manufacturing
            CTX_KEY_ARCHIVER.to_string() => "0".to_string(),
            CTX_KEY_DEBUG_SYMBOLS.to_string() => "0".to_string(), // This can be used if you need to collect debug symbols from a build and have specific task defined for it
        };
        let mut ctx: Context = Context::new(&ctx_default_variables);
        ctx.update(&variables);
        Ok(WsContextData {
            context: ctx,
        })
    }

    pub fn is_mutable(&self, key: &str) -> bool {
        Self::mutable_key(key)
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
        /*for (key, value) in variables {
            if !self.is_mutable(key) {
                return Err(BError::CtxKeyError(format!("Context value {} cannot not be changed!", key))));
            }
        }*/
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

    use crate::data::context::{
        CTX_KEY_MACHINE,
        CTX_KEY_ARCH,
        CTX_KEY_DISTRO,
        CTX_KEY_VARIANT,
        CTX_KEY_PRODUCT_NAME,
        CTX_KEY_BB_BUILD_DIR,
        CTX_KEY_BB_DEPLOY_DIR,
        CTX_KEY_ARTIFACTS_DIR,
        CTX_KEY_BUILDS_DIR,
        CTX_KEY_WORK_DIR,
        CTX_KEY_PLATFORM_VERSION,
        CTX_KEY_BUILD_ID,
        CTX_KEY_BUILD_SHA,
        CTX_KEY_RELEASE_BUILD,
        CTX_KEY_ARCHIVER,
        CTX_KEY_DEBUG_SYMBOLS, CTX_KEY_PLATFORM_RELEASE,
    };
    use crate::workspace::WsSettingsHandler;
    use crate::data::WsContextData;

    #[test]
    fn test_ws_context_data_default() {
        let json_default_build_config = r#"
        {                                                                                                                   
            "version": "4"
        }"#;
        let data: WsContextData = WsContextData::from_str(json_default_build_config).expect("Failed to parse context data");
        assert_eq!(data.get_ctx_value(CTX_KEY_MACHINE), "NA");
        assert_eq!(data.get_ctx_value(CTX_KEY_ARCH), "NA");
        assert_eq!(data.get_ctx_value(CTX_KEY_DISTRO), "NA");
        assert_eq!(data.get_ctx_value(CTX_KEY_VARIANT), "dev");
        assert_eq!(data.get_ctx_value(CTX_KEY_PRODUCT_NAME), "NA");
        assert_eq!(
            data.get_ctx_path(CTX_KEY_BB_BUILD_DIR),
            PathBuf::from("")
        );
        assert_eq!(
            data.get_ctx_path(CTX_KEY_BB_DEPLOY_DIR),
            PathBuf::from("")
        );
        assert_eq!(
            data.get_ctx_path(CTX_KEY_ARTIFACTS_DIR),
            PathBuf::from("")
        );
        assert_eq!(
            data.get_ctx_path(CTX_KEY_BUILDS_DIR),
            PathBuf::from("")
        );
        assert_eq!(data.get_ctx_path(CTX_KEY_WORK_DIR), PathBuf::from(""));
        assert_eq!(data.get_ctx_value(CTX_KEY_PLATFORM_VERSION), "0.0.0");
        assert_eq!(data.get_ctx_value(CTX_KEY_BUILD_ID), "0");
        assert_eq!(data.get_ctx_value(CTX_KEY_PLATFORM_RELEASE), "0.0.0-0");
        assert_eq!(data.get_ctx_value(CTX_KEY_BUILD_SHA), "dev");
        assert_eq!(data.get_ctx_value(CTX_KEY_RELEASE_BUILD), "0");
        assert_eq!(data.get_ctx_value(CTX_KEY_ARCHIVER), "0");
        assert_eq!(data.get_ctx_value(CTX_KEY_DEBUG_SYMBOLS), "0");
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
            CTX_KEY_MACHINE.to_string() => "test-machine".to_string(),
            CTX_KEY_ARCH.to_string() => "test-arch".to_string(),
            CTX_KEY_DISTRO.to_string() => "test-distro".to_string(),
            CTX_KEY_VARIANT.to_string() => "test-variant".to_string(),
            CTX_KEY_PRODUCT_NAME.to_string() => "test".to_string(),
            CTX_KEY_WORK_DIR.to_string() => settings.work_dir().to_string_lossy().to_string(),
        };
        data.update(&ctx_built_in_variables);
        assert_eq!(data.get_ctx_value(CTX_KEY_MACHINE), "test-machine");
        assert_eq!(data.get_ctx_value(CTX_KEY_ARCH), "test-arch");
        assert_eq!(data.get_ctx_value(CTX_KEY_DISTRO), "test-distro");
        assert_eq!(data.get_ctx_value(CTX_KEY_VARIANT), "test-variant");
        assert_eq!(data.get_ctx_value(CTX_KEY_PRODUCT_NAME), "test");
        assert_eq!(data.get_ctx_value("KEY1"), "value1");
        assert_eq!(data.get_ctx_value("KEY2"), "value2");
        assert_eq!(data.get_ctx_value("KEY3"), "value3");
        assert_eq!(data.get_ctx_path(CTX_KEY_WORK_DIR), settings.work_dir());

    }
}
