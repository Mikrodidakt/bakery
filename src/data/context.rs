use indexmap::{indexmap, IndexMap};
use serde_json::Value;
use std::path::PathBuf;

use crate::configs::Config;
use crate::configs::Context;
use crate::error::BError;

pub struct WsContextData {
    context: Context,
}

// Built in context variables
pub const CTX_KEY_MACHINE: &str = "MACHINE";
pub const CTX_KEY_ARCH: &str = "ARCH";
pub const CTX_KEY_DISTRO: &str = "DISTRO";
pub const CTX_KEY_BB_BUILD_DIR: &str = "BB_BUILD_DIR";
pub const CTX_KEY_BB_DEPLOY_DIR: &str = "BB_DEPLOY_DIR";
pub const CTX_KEY_ARTIFACTS_DIR: &str = "ARTIFACTS_DIR";
pub const CTX_KEY_LAYERS_DIR: &str = "LAYERS_DIR";
pub const CTX_KEY_SCRIPTS_DIR: &str = "SCRIPTS_DIR";
pub const CTX_KEY_BUILDS_DIR: &str = "BUILDS_DIR";
pub const CTX_KEY_WORK_DIR: &str = "WORK_DIR";
pub const CTX_KEY_PLATFORM_VERSION: &str = "PLATFORM_VERSION";
pub const CTX_KEY_BUILD_ID: &str = "BUILD_ID";
pub const CTX_KEY_PLATFORM_RELEASE: &str = "PLATFORM_RELEASE";
pub const CTX_KEY_BUILD_SHA: &str = "BUILD_SHA";
pub const CTX_KEY_BUILD_VARIANT: &str = "BUILD_VARIANT";
pub const CTX_KEY_RELEASE_BUILD: &str = "RELEASE_BUILD";
pub const CTX_KEY_ARCHIVER: &str = "ARCHIVER";
pub const CTX_KEY_DEBUG_SYMBOLS: &str = "DEBUG_SYMBOLS";
pub const CTX_KEY_DEVICE: &str = "DEVICE";
pub const CTX_KEY_IMAGE: &str = "IMAGE";
pub const CTX_KEY_DATE: &str = "DATE";
pub const CTX_KEY_TIME: &str = "TIME";
// By default all of these are the same unless they
// are specificly defined in the build config
pub const CTX_KEY_PRODUCT_NAME: &str = "PRODUCT_NAME";
pub const CTX_KEY_PROJECT_NAME: &str = "PROJECT_NAME";
pub const CTX_KEY_NAME: &str = "NAME";

impl Config for WsContextData {}

impl WsContextData {
    fn _mutable_key(key: &str) -> bool {
        match key {
            CTX_KEY_ARTIFACTS_DIR
            | CTX_KEY_SCRIPTS_DIR
            | CTX_KEY_PLATFORM_VERSION
            | CTX_KEY_BUILD_ID
            | CTX_KEY_PLATFORM_RELEASE
            | CTX_KEY_BUILD_SHA
            | CTX_KEY_BUILD_VARIANT
            | CTX_KEY_RELEASE_BUILD
            | CTX_KEY_ARCHIVER
            | CTX_KEY_DEVICE
            | CTX_KEY_IMAGE
            | CTX_KEY_DATE
            | CTX_KEY_TIME
            | CTX_KEY_DEBUG_SYMBOLS => true,
            CTX_KEY_MACHINE
            | CTX_KEY_ARCH
            | CTX_KEY_DISTRO
            | CTX_KEY_BB_BUILD_DIR
            | CTX_KEY_BB_DEPLOY_DIR
            | CTX_KEY_PRODUCT_NAME
            | CTX_KEY_PROJECT_NAME
            | CTX_KEY_NAME
            | CTX_KEY_WORK_DIR
            | CTX_KEY_LAYERS_DIR
            | CTX_KEY_BUILDS_DIR => false,
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
        /*
         * TODO: If any of these variables are set to anything but an empty string
         * they risk over write any context variable from the build config.
         * The reason for this is because we load the context variables first
         * from the build config and then we update the context with these
         * built-in variables. The problem is that many of these built-in variables
         * are getting there values from settings while others are not. Any empty
         * string will be ignored when updating the context but if the default
         * value is set here they will overwrite any values coming from the
         * build config. We should potentially switch the order need to look into
         * this in more details
         */
        let ctx_default_variables: IndexMap<String, String> = indexmap! {
            CTX_KEY_MACHINE.to_string() => "".to_string(),
            CTX_KEY_ARCH.to_string() => "".to_string(),
            CTX_KEY_DISTRO.to_string() => "".to_string(),
            CTX_KEY_PRODUCT_NAME.to_string() => "".to_string(),
            CTX_KEY_PROJECT_NAME.to_string() => "".to_string(),
            CTX_KEY_NAME.to_string() => "".to_string(),
            CTX_KEY_BB_BUILD_DIR.to_string() => "".to_string(),
            CTX_KEY_BB_DEPLOY_DIR.to_string() => "".to_string(),
            CTX_KEY_ARTIFACTS_DIR.to_string() => "".to_string(),
            CTX_KEY_LAYERS_DIR.to_string() => "".to_string(),
            CTX_KEY_SCRIPTS_DIR.to_string() => "".to_string(),
            CTX_KEY_BUILDS_DIR.to_string() => "".to_string(),
            CTX_KEY_WORK_DIR.to_string() => "".to_string(),
            CTX_KEY_PLATFORM_VERSION.to_string() => "0.0.0".to_string(),
            CTX_KEY_BUILD_ID.to_string() => "0".to_string(),
            CTX_KEY_PLATFORM_RELEASE.to_string() => "".to_string(),
            CTX_KEY_BUILD_SHA.to_string() => "".to_string(),
            CTX_KEY_RELEASE_BUILD.to_string() => "".to_string(),
            CTX_KEY_BUILD_VARIANT.to_string() => "dev".to_string(),
            CTX_KEY_ARCHIVER.to_string() => "".to_string(),
            CTX_KEY_DEBUG_SYMBOLS.to_string() => "".to_string(),
            CTX_KEY_DEVICE.to_string() => "".to_string(),
            CTX_KEY_IMAGE.to_string() => "".to_string(),
            CTX_KEY_TIME.to_string() => "".to_string(),
            CTX_KEY_DATE.to_string() => "".to_string(),
        };
        let mut ctx: Context = Context::new(&ctx_default_variables);
        ctx.update(&variables);
        Ok(WsContextData { context: ctx })
    }

    pub fn _is_mutable(&self, key: &str) -> bool {
        Self::_mutable_key(key)
    }

    pub fn ctx(&self) -> &Context {
        &self.context
    }

    pub fn update(&mut self, variables: &IndexMap<String, String>) {
        /*
         * We need to make sure that we are not trying to update
         * any of the context variables with empty values
         */
        let mut v: IndexMap<String, String> = IndexMap::new();
        for (key, value) in variables {
            //println!("key: {}, value: {}", key, value);
            if !value.is_empty() {
                v.insert(key.to_owned(), value.to_owned());
            }
        }
        self.context.update(&v);
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
        CTX_KEY_ARCH, CTX_KEY_ARCHIVER, CTX_KEY_ARTIFACTS_DIR, CTX_KEY_BB_BUILD_DIR,
        CTX_KEY_BB_DEPLOY_DIR, CTX_KEY_BUILDS_DIR, CTX_KEY_BUILD_ID, CTX_KEY_BUILD_SHA,
        CTX_KEY_BUILD_VARIANT, CTX_KEY_DATE, CTX_KEY_DEBUG_SYMBOLS, CTX_KEY_DEVICE, CTX_KEY_DISTRO,
        CTX_KEY_IMAGE, CTX_KEY_LAYERS_DIR, CTX_KEY_MACHINE, CTX_KEY_NAME, CTX_KEY_PLATFORM_RELEASE,
        CTX_KEY_PLATFORM_VERSION, CTX_KEY_PRODUCT_NAME, CTX_KEY_PROJECT_NAME,
        CTX_KEY_RELEASE_BUILD, CTX_KEY_SCRIPTS_DIR, CTX_KEY_TIME, CTX_KEY_WORK_DIR,
    };
    use crate::data::WsContextData;
    use crate::workspace::WsSettingsHandler;

    #[test]
    fn test_ws_context_data_default() {
        let json_default_build_config = r#"
        {
            "version": "5"
        }"#;
        let data: WsContextData = WsContextData::from_str(json_default_build_config)
            .expect("Failed to parse context data");
        assert!(data.get_ctx_value(CTX_KEY_MACHINE).is_empty());
        assert!(data.get_ctx_value(CTX_KEY_ARCH).is_empty());
        assert!(data.get_ctx_value(CTX_KEY_DISTRO).is_empty());
        assert!(data.get_ctx_value(CTX_KEY_PRODUCT_NAME).is_empty());
        assert!(data.get_ctx_value(CTX_KEY_PROJECT_NAME).is_empty());
        assert!(data.get_ctx_value(CTX_KEY_NAME).is_empty());
        assert!(data.get_ctx_value(CTX_KEY_DEVICE).is_empty());
        assert!(data.get_ctx_value(CTX_KEY_IMAGE).is_empty());
        assert_eq!(data.get_ctx_path(CTX_KEY_BB_BUILD_DIR), PathBuf::from(""));
        assert_eq!(data.get_ctx_path(CTX_KEY_BB_DEPLOY_DIR), PathBuf::from(""));
        assert_eq!(data.get_ctx_path(CTX_KEY_ARTIFACTS_DIR), PathBuf::from(""));
        assert_eq!(data.get_ctx_path(CTX_KEY_LAYERS_DIR), PathBuf::from(""));
        assert_eq!(data.get_ctx_path(CTX_KEY_SCRIPTS_DIR), PathBuf::from(""));
        assert_eq!(data.get_ctx_path(CTX_KEY_BUILDS_DIR), PathBuf::from(""));
        assert_eq!(
            data.get_ctx_value(CTX_KEY_PLATFORM_VERSION),
            String::from("0.0.0")
        );
        assert_eq!(data.get_ctx_value(CTX_KEY_BUILD_ID), String::from("0"));
        assert_eq!(data.get_ctx_path(CTX_KEY_WORK_DIR), PathBuf::from(""));
        assert_eq!(
            data.get_ctx_value(CTX_KEY_BUILD_VARIANT),
            String::from("dev")
        );
        assert_eq!(data.get_ctx_value(CTX_KEY_DATE), String::from(""));
        assert_eq!(data.get_ctx_value(CTX_KEY_TIME), String::from(""));
        assert!(data.get_ctx_value(CTX_KEY_PLATFORM_RELEASE).is_empty());
        assert!(data.get_ctx_value(CTX_KEY_BUILD_SHA).is_empty());
        assert!(data.get_ctx_value(CTX_KEY_RELEASE_BUILD).is_empty());
        assert!(data.get_ctx_value(CTX_KEY_ARCHIVER).is_empty());
        assert!(data.get_ctx_value(CTX_KEY_DEBUG_SYMBOLS).is_empty());
    }

    #[test]
    fn test_ws_context_data() {
        let json_settings: &str = r#"
        {
            "version": "5"
        }"#;
        let json_build_config = r#"
        {
            "version": "5",
            "context": [
                "KEY1=value1",
                "KEY2=value2",
                "KEY3=value3"
            ]
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_settings)
            .expect("Failed to parse settings");
        let mut data: WsContextData =
            WsContextData::from_str(json_build_config).expect("Failed to parse context data");
        let ctx_built_in_variables: IndexMap<String, String> = indexmap! {
            CTX_KEY_MACHINE.to_string() => "test-machine".to_string(),
            CTX_KEY_ARCH.to_string() => "test-arch".to_string(),
            CTX_KEY_DISTRO.to_string() => "test-distro".to_string(),
            CTX_KEY_BUILD_VARIANT.to_string() => "test-variant".to_string(),
            CTX_KEY_PRODUCT_NAME.to_string() => "test".to_string(),
            CTX_KEY_WORK_DIR.to_string() => settings.work_dir().to_string_lossy().to_string(),
        };
        data.update(&ctx_built_in_variables);
        assert_eq!(data.get_ctx_value(CTX_KEY_MACHINE), "test-machine");
        assert_eq!(data.get_ctx_value(CTX_KEY_ARCH), "test-arch");
        assert_eq!(data.get_ctx_value(CTX_KEY_DISTRO), "test-distro");
        assert_eq!(data.get_ctx_value(CTX_KEY_BUILD_VARIANT), "test-variant");
        assert_eq!(data.get_ctx_value(CTX_KEY_PRODUCT_NAME), "test");
        assert_eq!(data.get_ctx_value("KEY1"), "value1");
        assert_eq!(data.get_ctx_value("KEY2"), "value2");
        assert_eq!(data.get_ctx_value("KEY3"), "value3");
        assert_eq!(data.get_ctx_path(CTX_KEY_WORK_DIR), settings.work_dir());
    }

    #[test]
    fn test_ws_builtin_context_data() {
        let json_build_config = r#"
        {
            "version": "5",
            "context": [
                "IMAGE=image",
                "DEVICE=device"
            ]
        }"#;
        let data: WsContextData =
            WsContextData::from_str(json_build_config).expect("Failed to parse context data");
        assert_eq!(data.get_ctx_value(CTX_KEY_IMAGE), "image");
        assert_eq!(data.get_ctx_value(CTX_KEY_DEVICE), "device");
    }
}
