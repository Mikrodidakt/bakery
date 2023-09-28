use indexmap::{indexmap, IndexMap};
use serde_json::Value;
use std::path::PathBuf;

use crate::configs::{bitbake, BuildConfig, Context, TaskConfig};
use crate::error::BError;
use crate::workspace::{WsArtifactsHandler, WsSettingsHandler, WsTaskHandler};

pub struct WsBuildData {
    product: String,
    ctx: Context,
    settings: WsSettingsHandler,
    bb_build_dir: PathBuf,
    bb_deploy_dir: PathBuf,
}

impl WsBuildData {
    pub fn new(
        product: &str,
        bitbake_deploy_dir: &str,
        ctx_variables: IndexMap<String, String>,
        settings: &WsSettingsHandler,
    ) -> Result<Self, BError> {
        let bb_build_dir: PathBuf = settings.builds_dir().clone().join(PathBuf::from(product));
        let bb_deploy_dir: PathBuf = bb_build_dir.clone().join(PathBuf::from(bitbake_deploy_dir));
        let ctx_default_variables: IndexMap<String, String> = indexmap! {
            "MACHINE".to_string() => "".to_string(),
            "ARCH".to_string() => "".to_string(),
            "DISTRO".to_string() => "".to_string(),
            "VARIANT".to_string() => "".to_string(),
            "PRODUCT_NAME".to_string() => product.to_string(),
            "BB_BUILD_DIR".to_string() => bb_build_dir.to_string_lossy().to_string(),
            "BB_DEPLOY_DIR".to_string() => bb_deploy_dir.to_string_lossy().to_string(),
            "ARTIFACTS_DIR".to_string() => settings.artifacts_dir().to_string_lossy().to_string(),
            "BUILDS_DIR".to_string() => settings.builds_dir().to_string_lossy().to_string(),
            "WORK_DIR".to_string() => settings.work_dir().to_string_lossy().to_string(),
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
        ctx.update(&ctx_variables);

        Ok(WsBuildData {
            bb_build_dir,
            bb_deploy_dir,
            product: product.to_string(),
            ctx,
            settings: settings.clone(),
        })
    }

    pub fn get_artifacts(
        &self,
        data: &Value,
        build_dir: &PathBuf,
    ) -> Result<Vec<WsArtifactsHandler>, BError> {
        match data.get("artifacts") {
            Some(value) => {
                if value.is_array() {
                    if let Some(artifact_vec) = value.as_array() {
                        let mut artifacts: Vec<WsArtifactsHandler> = Vec::new();
                        for artifact_data in artifact_vec.iter() {
                            let artifact: WsArtifactsHandler =
                                self.get_artifact(artifact_data, build_dir)?;
                            artifacts.push(artifact);
                        }
                        return Ok(artifacts);
                    }
                    return Err(BError {
                        code: 0,
                        message: format!("Invalid 'artifacts' format in build config"),
                    });
                } else {
                    return Err(BError {
                        code: 0,
                        message: format!("No 'artifacts' array node found in build config"),
                    });
                }
            }
            None => {
                return Ok(Vec::new());
            }
        }
    }

    pub fn get_tasks(&self, data: &Value) -> Result<IndexMap<String, WsTaskHandler>, BError> {
        match data.get("tasks") {
            Some(value) => {
                if value.is_object() {
                    if let Some(task_map) = value.as_object() {
                        let mut tasks: IndexMap<String, WsTaskHandler> = IndexMap::new();
                        for (name, data) in task_map.iter() {
                            let task: WsTaskHandler = self.get_task(data)?;
                            tasks.insert(name.clone(), task);
                        }
                        return Ok(tasks);
                    }
                    return Err(BError {
                        code: 0,
                        message: format!("Invalid 'task' format in build config"),
                    });
                } else {
                    return Err(BError {
                        code: 0,
                        message: format!("No 'tasks' node found in build config"),
                    });
                }
            }
            None => {
                return Ok(IndexMap::new());
            }
        }
    }

    pub fn get_task(&self, data: &Value) -> Result<WsTaskHandler, BError> {
        let mut task: WsTaskHandler = WsTaskHandler::new(data, &self)?;
        task.expand_ctx(self.context());
        Ok(task)
    }

    pub fn get_artifact(
        &self,
        data: &Value,
        build_dir: &PathBuf,
    ) -> Result<WsArtifactsHandler, BError> {
        let mut artifact: WsArtifactsHandler = WsArtifactsHandler::new(data, build_dir, &self)?;
        artifact.expand_ctx(self.context());
        Ok(artifact)
    }

    pub fn settings(&self) -> &WsSettingsHandler {
        &self.settings
    }

    pub fn context(&self) -> &Context {
        &self.ctx
    }

    pub fn update_context(&mut self, variables: &IndexMap<String, String>) {
        self.ctx.update(variables);
    }

    pub fn product(&self) -> &str {
        &self.product
    }

    pub fn bb_build_dir(&self) -> PathBuf {
        self.bb_build_dir.clone()
    }

    pub fn bb_deploy_dir(&self) -> PathBuf {
        self.bb_deploy_dir.clone()
    }

    pub fn get_ctx_path(&self, key: &str) -> PathBuf {
        PathBuf::from(self.get_ctx_value(key))
    }

    pub fn get_ctx_value(&self, key: &str) -> String {
        self.ctx.value(key)
    }
}

#[cfg(test)]
mod tests {
    use indexmap::{indexmap, IndexMap};
    use serde_json::Value;
    use std::path::PathBuf;

    use crate::error::BError;
    use crate::fs::JsonFileReader;
    use crate::configs::AType;
    use crate::workspace::{WsArtifactsHandler, WsBuildData, WsSettingsHandler, WsTaskHandler};

    #[test]
    fn test_ws_build_data_default() {
        let default_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let ws_settings: WsSettingsHandler =
            WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let data: WsBuildData =
            WsBuildData::new("test", "tmp/deploy/", IndexMap::new(), &ws_settings)
                .expect("Failed to setup build data");
        assert_eq!(data.product(), "test");
        assert_eq!(data.bb_build_dir(), PathBuf::from("/workspace/builds/test"));
        assert_eq!(
            data.bb_deploy_dir(),
            PathBuf::from("/workspace/builds/test/tmp/deploy/")
        );
        assert_eq!(data.get_ctx_value("MACHINE"), "");
        assert_eq!(data.get_ctx_value("ARCH"), "");
        assert_eq!(data.get_ctx_value("DISTRO"), "");
        assert_eq!(data.get_ctx_value("VARIANT"), "");
        assert_eq!(data.get_ctx_value("PRODUCT_NAME"), "test");
        assert_eq!(
            data.get_ctx_path("BB_BUILD_DIR"),
            PathBuf::from("/workspace/builds/test")
        );
        assert_eq!(
            data.get_ctx_path("BB_DEPLOY_DIR"),
            PathBuf::from("/workspace/builds/test/tmp/deploy/")
        );
        assert_eq!(
            data.get_ctx_path("ARTIFACTS_DIR"),
            PathBuf::from("/workspace/artifacts")
        );
        assert_eq!(
            data.get_ctx_path("BUILDS_DIR"),
            PathBuf::from("/workspace/builds")
        );
        assert_eq!(data.get_ctx_path("WORK_DIR"), PathBuf::from("/workspace"));
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
    fn test_ws_build_data_overwrite_ctx() {
        let ctx_variables: IndexMap<String, String> = indexmap! {
            "MACHINE".to_string() => "test-machine".to_string(),
            "ARCH".to_string() => "test-arch".to_string(),
            "DISTRO".to_string() => "test-distro".to_string(),
            "VARIANT".to_string() => "test-variant".to_string(),
            "PRODUCT_NAME".to_string() => "test".to_string(),
            "PLATFORM_VERSION".to_string() => "1.2.3".to_string(),
            "BUILD_NUMBER".to_string() => "10".to_string(),
            "PLATFORM_RELEASE".to_string() => "1.2.3-10".to_string(),
        };
        let default_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let ws_settings: WsSettingsHandler =
            WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let data: WsBuildData =
            WsBuildData::new("test", "tmp/deploy/", ctx_variables, &ws_settings)
                .expect("Failed to setup build data");
        assert_eq!(data.get_ctx_value("MACHINE"), "test-machine");
        assert_eq!(data.get_ctx_value("ARCH"), "test-arch");
        assert_eq!(data.get_ctx_value("DISTRO"), "test-distro");
        assert_eq!(data.get_ctx_value("VARIANT"), "test-variant");
        assert_eq!(data.get_ctx_value("PRODUCT_NAME"), "test");
        assert_eq!(data.get_ctx_value("PLATFORM_VERSION"), "1.2.3");
        assert_eq!(data.get_ctx_value("BUILD_NUMBER"), "10");
        assert_eq!(data.get_ctx_value("PLATFORM_RELEASE"), "1.2.3-10");
    }

    #[test]
    fn test_ws_build_data_no_tasks() {
        let default_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_build_config = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {}
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let ws_settings: WsSettingsHandler =
            WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let data: WsBuildData =
            WsBuildData::new("test", "tmp/deploy/", IndexMap::new(), &ws_settings)
                .expect("Failed to setup build data");
        let json_data: Value =
            JsonFileReader::parse(json_build_config).expect("Failed to parse json");
        let tasks: IndexMap<String, WsTaskHandler> =
            data.get_tasks(&json_data).expect("Failed to parse tasks");
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_ws_build_data_tasks_error() {
        let default_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_build_config: &str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {},
            "tasks": "error"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let ws_settings: WsSettingsHandler =
            WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let data: WsBuildData =
            WsBuildData::new("test", "tmp/deploy/", IndexMap::new(), &ws_settings)
                .expect("Failed to setup build data");
        let json_data: Value =
            JsonFileReader::parse(json_build_config).expect("Failed to parse json");
        let result: Result<IndexMap<String, WsTaskHandler>, BError> = data.get_tasks(&json_data);
        match result {
            Ok(_rconfig) => {
                panic!("We should have recived an error because we have no valid config!");
            }
            Err(e) => {
                assert_eq!(
                    e.message,
                    String::from("No 'tasks' node found in build config")
                );
            }
        }
    }

    #[test]
    fn test_ws_build_data_task() {
        let default_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "bitbake",
            "recipes": [
                "test-image"
            ]
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let ws_settings: WsSettingsHandler =
            WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let data: WsBuildData =
            WsBuildData::new("test", "tmp/deploy/", IndexMap::new(), &ws_settings)
                .expect("Failed to setup build data");
        let json_data: Value = JsonFileReader::parse(json_task_str).expect("Failed to parse json");
        let task: WsTaskHandler = data.get_task(&json_data).expect("Failed to parse task");
        assert_eq!(task.build_dir(), PathBuf::from("/workspace/builds/test"));
        assert_eq!(task.name(), "task-name");
    }

    #[test]
    fn test_ws_build_data_task_expand_ctx() {
        let ctx_variables: IndexMap<String, String> = indexmap! {
            "RECIPE_NAME".to_string() => "test-image".to_string(),
        };
        let default_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "bitbake",
            "recipes": [
                "${RECIPE_NAME}"
            ]
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let ws_settings: WsSettingsHandler =
            WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let data: WsBuildData =
            WsBuildData::new("test", "tmp/deploy/", ctx_variables, &ws_settings)
                .expect("Failed to setup build data");
        let json_data: Value = JsonFileReader::parse(json_task_str).expect("Failed to parse json");
        let task: WsTaskHandler = data.get_task(&json_data).expect("Failed to parse task");
        assert_eq!(task.recipes(), &vec!["test-image"]);
    }

    #[test]
    fn test_ws_build_tasks() {
        let default_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_build_config: &str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {},
            "tasks": { 
                "task1": {
                    "index": "1",
                    "name": "task1",
                    "type": "non-bitbake"
                },
                "task2": {
                    "index": "2",
                    "name": "task2",
                    "type": "non-bitbake"
                }
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let ws_settings: WsSettingsHandler =
            WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let data: WsBuildData =
            WsBuildData::new("test", "tmp/deploy/", IndexMap::new(), &ws_settings)
                .expect("Failed to setup build data");
        let json_data: Value =
            JsonFileReader::parse(json_build_config).expect("Failed to parse json");
        let tasks: IndexMap<String, WsTaskHandler> =
            data.get_tasks(&json_data).expect("Failed to parse tasks");
        assert!(!tasks.is_empty());
        let mut i: usize = 1;
        tasks.iter().for_each(|(name, task)| {
            assert_eq!(name, &format!("task{}", i));
            assert_eq!(task.name(), &format!("task{}", i));
            i += 1;
        });
    }

    #[test]
    fn test_ws_build_data_artifacts_error() {
        let default_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_build_config: &str = r#"
        { 
            "index": "2",
            "name": "task2",
            "type": "non-bitbake",
            "artifacts": "error"
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let ws_settings: WsSettingsHandler =
            WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let data: WsBuildData =
            WsBuildData::new("test", "tmp/deploy/", IndexMap::new(), &ws_settings)
                .expect("Failed to setup build data");
        let json_data: Value =
            JsonFileReader::parse(json_build_config).expect("Failed to parse json");
        let result: Result<Vec<WsArtifactsHandler>, BError> =
            data.get_artifacts(&json_data, &task_build_dir);
        match result {
            Ok(_rconfig) => {
                panic!("We should have recived an error because we have no valid config!");
            }
            Err(e) => {
                assert_eq!(
                    e.message,
                    String::from("No 'artifacts' array node found in build config")
                );
            }
        }
    }

    #[test]
    fn test_ws_build_data_artifact() {
        let default_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_build_config: &str = r#"
        {
            "type": "manifest",
            "name": "test-manifest",
            "content": {
                "key1": "value1",
                "key2": "value2",
                "data": {
                    "key3": "value3"
                }
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let ws_settings: WsSettingsHandler =
            WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let data: WsBuildData =
            WsBuildData::new("test", "tmp/deploy/", IndexMap::new(), &ws_settings)
                .expect("Failed to setup build data");
        let json_data: Value =
            JsonFileReader::parse(json_build_config).expect("Failed to parse json");
        let artifact: WsArtifactsHandler = data
            .get_artifact(&json_data, &task_build_dir)
            .expect("Failed to parse artifacts");
        assert_eq!(artifact.atype(), &AType::Manifest);
        assert_eq!(artifact.name(), "test-manifest");
    }

    #[test]
    fn test_ws_build_data_expand_artifact() {
        let ctx_variables: IndexMap<String, String> = indexmap! {
            "MANIFEST_FILE_NAME".to_string() => "test-manifest".to_string(),
        };
        let default_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_build_config: &str = r#"
        {
            "type": "manifest",
            "name": "${MANIFEST_FILE_NAME}",
            "content": {
                "key1": "value1",
                "key2": "value2",
                "data": {
                    "key3": "value3"
                }
            }
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let ws_settings: WsSettingsHandler =
            WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let data: WsBuildData =
            WsBuildData::new("test", "tmp/deploy/", ctx_variables, &ws_settings)
                .expect("Failed to setup build data");
        let json_data: Value =
            JsonFileReader::parse(json_build_config).expect("Failed to parse json");
        let artifact: WsArtifactsHandler = data
            .get_artifact(&json_data, &task_build_dir)
            .expect("Failed to parse artifacts");
        assert_eq!(artifact.atype(), &AType::Manifest);
        assert_eq!(artifact.name(), "test-manifest");
    }

    #[test]
    fn test_ws_build_data_artifacts() {
        let ctx_variables: IndexMap<String, String> = indexmap! {
            "ARCHIVE_NAME".to_string() => "test.zip".to_string(),
        };
        let default_settings: &str = r#"
        {
            "version": "4"
        }"#;
        let json_build_config: &str = r#"
        {
            "artifacts": [
                {
                    "source": "file1.txt",
                    "dest": "file1.txt"
                },
                {
                    "source": "file2.txt",
                    "dest": "file2.txt"
                }
            ]
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let task_build_dir: PathBuf = work_dir.clone().join("task/dir");
        let ws_settings: WsSettingsHandler =
            WsSettingsHandler::from_str(&work_dir, default_settings).unwrap();
        let data: WsBuildData =
            WsBuildData::new("test", "tmp/deploy/", ctx_variables, &ws_settings)
                .expect("Failed to setup build data");
        let json_data: Value =
            JsonFileReader::parse(json_build_config).expect("Failed to parse json");
        let artifacts: Vec<WsArtifactsHandler> = data
            .get_artifacts(&json_data, &task_build_dir)
            .expect("Failed to parse artifacts");
        assert!(!artifacts.is_empty());
        let mut i: usize = 1;
        artifacts.iter().for_each(|a| {
            assert_eq!(a.atype(), &AType::File);
            assert_eq!(a.source(), PathBuf::from(format!("/workspace/task/dir/file{}.txt", i)));
            i += 1;
        });
    }
}
