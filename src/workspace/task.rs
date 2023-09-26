use crate::configs::{TType, TaskConfig, ArtifactConfig};
use crate::workspace::{WsBuildConfigHandler, WsArtifactConfigHandler};

use std::path::{Path, PathBuf};

pub struct WsTaskConfigHandler<'a> {
    name: String,
    task_config: &'a TaskConfig,
    bb_build_dir: PathBuf,
    work_dir: PathBuf,
    artifacts_dir: PathBuf,
}

impl<'a> WsTaskConfigHandler<'a> {
    pub fn new(task_config: &'a TaskConfig, work_dir: &PathBuf, bb_build_dir: &PathBuf, artifacts_dir: &PathBuf) -> Self {
        WsTaskConfigHandler {
            name: task_config.name.to_string(),
            task_config,
            work_dir: work_dir.clone(),
            bb_build_dir: bb_build_dir.clone(),
            artifacts_dir: artifacts_dir.clone(),
        }
    }

    pub fn build_dir(&self) -> PathBuf {
        if self.task_config.ttype == TType::Bitbake {
            let task_build_dir: &str = &self.task_config.builddir;
            if task_build_dir.is_empty() {
                return self.bb_build_dir.clone();
            }
        }

        self.work_dir.clone().join(PathBuf::from(&self.task_config.builddir))
    }

    pub fn artifacts_dir(&self) -> PathBuf {
        self.artifacts_dir.clone()
    }

    pub fn ttype(&self) -> &TType {
        &self.task_config.ttype
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn build_cmd(&self) -> &str {
        &self.task_config.build
    }

    pub fn clean_cmd(&self) -> &str {
        &self.task_config.clean
    }

    pub fn docker(&self) -> &str {
        &self.task_config.docker
    }

    pub fn disabled(&self) -> bool {
        if self.task_config.disabled == "true" {
            return true;
        }
        return false;
    }

    pub fn recipes(&self) -> &Vec<String> {
        &self.task_config.recipes
    }

    pub fn condition(&self) -> bool {
        let condition: &str = &self.task_config.condition;

        if condition.is_empty() {
            return true;
        }

        match condition {
            "1" | "yes" | "y" | "Y" | "true" | "YES" | "TRUE" | "True" | "Yes" => return true,
            _ => return false,
        }
    }
    
    pub fn artifacts(&self) -> Vec<WsArtifactConfigHandler> {
        let mut artifacts: Vec<WsArtifactConfigHandler> = Vec::new();
        self.task_config.artifacts.iter().for_each(|config| {
            artifacts.push(WsArtifactConfigHandler::new(config, &self));
        });
        artifacts
    }
}

#[cfg(test)]
mod tests {
    use std::path::{PathBuf};
    use indexmap::{IndexMap, indexmap};

    use crate::helper::Helper;
    use crate::workspace::WsTaskConfigHandler;
    use crate::configs::{TType, TaskConfig, Context};

    #[test]
    fn test_ws_task_nonbitbake() {
        let variables: IndexMap<String, String> = indexmap! {
            "TASK_BUILD_DIR".to_string() => "task/build/dir".to_string(),
            "TASK_CONDITION".to_string() => "1".to_string(),
        };
        let ctx: Context = Context::new(&variables);
        let json_task_str: &str = r#"
        { 
            "index": "0",
            "name": "task-name",
            "type": "non-bitbake",
            "disabled": "false",
            "condition": "${TASK_CONDITION}",
            "builddir": "test/${TASK_BUILD_DIR}",
            "build": "build-cmd",
            "clean": "clean-cmd",
            "artifacts": []
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let bb_build_dir: PathBuf = PathBuf::from("/workspace/builds/tmp");
        let artifacts_dir: PathBuf = PathBuf::from("/workspace/artifacts");
        let mut config: TaskConfig = Helper::setup_task_config(json_task_str);
        config.expand_ctx(&ctx);
        let task: WsTaskConfigHandler = WsTaskConfigHandler::new(
            &config,
            &work_dir,
            &bb_build_dir,
            &artifacts_dir
        );
        assert_eq!(task.build_dir(), PathBuf::from("/workspace/test/task/build/dir"));
        assert!(task.condition());
        assert_eq!(task.name(), "task-name");
        assert_eq!(task.build_cmd(), "build-cmd");
        assert_eq!(task.clean_cmd(), "clean-cmd");
        assert_eq!(task.ttype(), &TType::NonBitbake);
        assert!(!task.disabled());
    }

    #[test]
    fn test_ws_task_bitbake() {
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
        let bb_build_dir: PathBuf = PathBuf::from("/workspace/builds/tmp");
        let artifacts_dir: PathBuf = PathBuf::from("/workspace/artifacts");
        let config: TaskConfig = Helper::setup_task_config(json_task_str);
        let task: WsTaskConfigHandler = WsTaskConfigHandler::new(
            &config,
            &work_dir,
            &bb_build_dir,
            &artifacts_dir
        );
        assert_eq!(task.build_dir(), bb_build_dir);
        assert!(task.condition());
        assert_eq!(task.name(), "task-name");
        assert_eq!(task.ttype(), &TType::Bitbake);
        assert_eq!(task.recipes(), &vec!["test-image".to_string()]);
        assert!(!task.disabled());
    }

    #[test]
    fn test_ws_task_bitbake_artifacts() {
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "bitbake",
            "recipes": [
                "test-image"
            ],
            "artifacts": [
                {
                    "source": "test/test.img"
                }
            ]
        }"#;
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let bb_build_dir: PathBuf = PathBuf::from("/workspace/builds/tmp");
        let artifacts_dir: PathBuf = PathBuf::from("/workspace/artifacts");
        let config: TaskConfig = Helper::setup_task_config(json_task_str);
        let task: WsTaskConfigHandler = WsTaskConfigHandler::new(
            &config,
            &work_dir,
            &bb_build_dir,
            &artifacts_dir
        );
        assert_eq!(task.build_dir(), bb_build_dir);
        assert!(task.condition());
        assert_eq!(task.name(), "task-name");
        assert_eq!(task.ttype(), &TType::Bitbake);
        assert_eq!(task.recipes(), &vec!["test-image".to_string()]);
        assert!(!task.disabled());
        assert_eq!(task.artifacts().get(0).unwrap().source(), bb_build_dir.join("test/test.img"));
        assert_eq!(task.artifacts().get(0).unwrap().dest(), artifacts_dir);
    }
}
