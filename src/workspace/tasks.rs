use crate::configs::Context;
use crate::executers::{Recipe, Executer, DockerImage};
use crate::workspace::WsArtifactsHandler;
use crate::error::BError;
use crate::fs::{JsonFileReader, BitbakeConf};
use crate::cli::Cli;
use crate::data::{
    WsBuildData,
    WsTaskData,
    WsBitbakeData,
    TType,
};

use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use serde_json::Value;
use std::path::PathBuf;

pub struct WsTaskHandler {
    data: WsTaskData,
    artifacts: Vec<WsArtifactsHandler>,
}

impl WsTaskHandler {
    pub fn from_str(json_config: &str, build_data: &WsBuildData) -> Result<Self, BError> {
        let data: Value = JsonFileReader::parse(json_config)?;
        Self::new(&data, build_data)
    }

    pub fn new(data: &Value, build_data: &WsBuildData) -> Result<Self, BError> {
        let mut task_data: WsTaskData = WsTaskData::from_value(data, build_data)?;
        // expand the context for the task config data
        // all the variables encapsulated insode ${} in the task config
        // will be expanded
        task_data.expand_ctx(build_data.context().ctx());
        let artifacts: Vec<WsArtifactsHandler> = build_data.get_artifacts(data, task_data.build_dir())?;
        
        Ok(WsTaskHandler {
            data: task_data,
            artifacts,
        })
    }

    fn bb_build_env(&self, build_data: &WsBuildData, _env_variables: &HashMap<String, String>) -> Result<HashMap<String, String>, BError> {
        //let task_env = self.env();
        //let os_env = env::vars();
        Ok(HashMap::new())
    }

    fn execute(&self, cli: &Cli, build_data: &WsBuildData, env: &HashMap<String, String>, interactive: bool) -> Result<(), BError> {
        let executer: Executer = Executer::new(build_data, cli);
        let mut docker_option: Option<DockerImage> = None;
        let mut cmd_line: Vec<String> = self.data.build_cmd().split(' ').map(|c| c.to_string()).collect();

        if !self.data.docker_image().is_empty() {
            let image: DockerImage = DockerImage::new(self.data.docker_image());
            docker_option = Some(image);
        }
        
        executer.execute(&mut cmd_line, env, Some(self.data.build_dir()), docker_option, interactive)?;
        Ok(())
    }

    fn execute_recipes(&self, cli: &Cli, build_data: &WsBuildData, env: &HashMap<String, String>, interactive: bool) -> Result<(), BError> {
        for r in self.data.recipes() {
            let recipe: Recipe = Recipe::new(r);
            let executer: Executer = Executer::new(build_data, cli);
            let mut docker_option: Option<DockerImage> = None;

            if !self.data.docker_image().is_empty() {
                let image: DockerImage = DockerImage::new(self.data.docker_image());
                docker_option = Some(image);
            }

            executer.execute(&mut recipe.bitbake_cmd(), env, Some(self.data.build_dir()), docker_option, interactive)?;
        }
        Ok(())
    }

    pub fn run<'a>(&self, cli: &'a Cli, build_data: &WsBuildData, bb_variables: &Vec<String>, env_variables: &HashMap<String, String>, dry_run: bool, interactive: bool) -> Result<(), BError> {
        if self.data.disabled() {
            cli.info(format!("Task '{}' is disabled in build config so execution is skipped", self.data.name()));
            return Ok(());
        }

        if !self.data.condition() {
            cli.info(format!("Task condition for task '{}' is not meet so execution is skipped", self.data.name()));
            return Ok(()); 
        }

        match self.data.ttype() {
            TType::Bitbake => {
                // if we are running a dry run we should always create the bb configs
                // when not a dry run it will be determined if it is needed or not to
                // regenerate the bb configs
                let force: bool = dry_run;
                let conf: BitbakeConf = BitbakeConf::new(build_data.bitbake(), bb_variables, force);
                conf.create_bitbake_configs(cli)?;

                if dry_run {
                    cli.info("Dry run. Skipping build!".to_string());
                    return Ok(());
                }

                let env: HashMap<String, String> = self.bb_build_env(build_data, env_variables)?;
                self.execute_recipes(cli, build_data, &env, interactive)?;
            }
            TType::NonBitbake => {
                self.execute(cli, build_data, env_variables, interactive)?;
            }
            _ => {
                return Err(BError::ValueError("Invalid task type".to_string()));
            }
        }
        Ok(())
    }

    pub fn expand_ctx(&mut self, ctx: &Context) {
        self.data.expand_ctx(ctx);
    }

    pub fn data(&self) -> &WsTaskData {
        &self.data
    }
    
    pub fn artifacts(&self) -> &Vec<WsArtifactsHandler> {
        &self.artifacts
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use tempdir::TempDir;
    
    use crate::cli::{BLogger, Cli, MockSystem, CallParams};
    use crate::workspace::{
        WsTaskHandler,
        WsArtifactsHandler,
    };
    use crate::data::{
        TType,
        AType, 
        WsBuildData,
    };
    use crate::helper::Helper;

    #[test]
    fn test_ws_task_nonbitbake() {
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let json_task_str: &str = r#"
        { 
            "index": "0",
            "name": "task-name",
            "type": "non-bitbake",
            "disabled": "false",
            "condition": "true",
            "builddir": "task/dir",
            "build": "build-cmd",
            "clean": "clean-cmd",
            "artifacts": []
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let task: WsTaskHandler = WsTaskHandler::from_str(json_task_str, &build_data).expect("Failed to parse Task config");
        assert_eq!(task.data().build_dir(), &PathBuf::from("/workspace/task/dir"));
        assert!(task.data().condition());
        assert_eq!(task.data().name(), "task-name");
        assert_eq!(task.data().build_cmd(), "build-cmd");
        assert_eq!(task.data().clean_cmd(), "clean-cmd");
        assert_eq!(task.data().ttype(), &TType::NonBitbake);
        assert!(!task.data().disabled());
    }

    #[test]
    fn test_ws_task_bitbake() {
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "bitbake",
            "recipes": [
                "test-image"
            ]
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let task: WsTaskHandler = WsTaskHandler::from_str(json_task_str, &build_data).expect("Failed to parse Task config");
        assert_eq!(task.data().build_dir(), &PathBuf::from("/workspace/builds/NA"));
        assert!(task.data().condition());
        assert_eq!(task.data().name(), "task-name");
        assert_eq!(task.data().ttype(), &TType::Bitbake);
        assert_eq!(task.data().recipes(), &vec!["test-image".to_string()]);
        assert!(!task.data().disabled());
    }

    #[test]
    fn test_ws_task_bb_build_dir() {
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "bitbake",
            "recipes": [
                "test-image"
            ]
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let task: WsTaskHandler = WsTaskHandler::from_str(json_task_str, &build_data).expect("Failed to parse Task config");
        assert_eq!(task.data().build_dir(), &PathBuf::from("/workspace/builds/NA"));
        assert!(task.data().condition());
        assert_eq!(task.data().name(), "task-name");
        assert_eq!(task.data().ttype(), &TType::Bitbake);
        assert_eq!(task.data().recipes(), &vec!["test-image".to_string()]);
        assert!(!task.data().disabled());
    }

    #[test]
    fn test_ws_task_nonbitbake_artifacts() {
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "non-bitbake",
            "builddir": "task/build/dir",
            "build": "build-cmd",
            "clean": "clean-cmd",
            "artifacts": [
                {
                    "type": "archive",
                    "name": "test.zip",
                    "artifacts": [
                        {
                            "source": "file3.txt",
                            "dest": "file4.txt"
                        },
                        {
                            "type": "directory",
                            "name": "dir-name",
                            "artifacts": [
                                {
                                    "source": "file1.txt"
                                },
                                {
                                    "source": "file2.txt"
                                },
                                {
                                    "source": "file3.txt"
                                }
                            ]
                        }
                    ]
                }
            ]
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let task: WsTaskHandler = WsTaskHandler::from_str(json_task_str, &build_data).expect("Failed to parse Task config");
        assert_eq!(task.data().build_dir(), &PathBuf::from("/workspace/task/build/dir"));
        assert!(task.data().condition());
        assert_eq!(task.data().name(), "task-name");
        assert_eq!(task.data().ttype(), &TType::NonBitbake);
        assert_eq!(task.data().build_cmd(), "build-cmd");
        assert_eq!(task.data().clean_cmd(), "clean-cmd");
        assert!(!task.data().disabled());
        let artifacts: &WsArtifactsHandler = task.artifacts().first().unwrap();
        assert_eq!(artifacts.data().atype(), &AType::Archive);
        assert_eq!(artifacts.data().name(), "test.zip");
        assert!(!artifacts.artifacts().is_empty());
        let archive_artifacts: &Vec<WsArtifactsHandler> = artifacts.artifacts();
        assert_eq!(archive_artifacts.get(0).unwrap().data().atype(), &AType::File);
        assert_eq!(archive_artifacts.get(0).unwrap().data().source(), &PathBuf::from("/workspace/task/build/dir/file3.txt"));
        assert_eq!(archive_artifacts.get(0).unwrap().data().dest(), &PathBuf::from("/workspace/artifacts/file4.txt"));
        assert_eq!(archive_artifacts.get(1).unwrap().data().atype(), &AType::Directory);
        assert_eq!(archive_artifacts.get(1).unwrap().data().name(), "dir-name");
        let dir_artifacts: &Vec<WsArtifactsHandler> = archive_artifacts.get(1).unwrap().artifacts();
        let mut i: usize = 1;
        dir_artifacts.iter().for_each(|f| {
            assert_eq!(f.data().atype(), &AType::File);
            assert_eq!(f.data().source(), &PathBuf::from(format!("/workspace/task/build/dir/file{}.txt", i)));
            assert_eq!(f.data().dest(), &PathBuf::from("/workspace/artifacts/"));
            i += 1;
        });
    }

    #[test]
    fn test_ws_task_expand_ctx() {
        let work_dir: PathBuf = PathBuf::from("/workspace");
        let json_build_config: &str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "context": [
                "ARCHIVE_NAME=test.zip",
                "DIR_NAME=dir-name",
                "FILE_NAME=file2.txt",
                "BITBAKE_IMAGE=test-image",
                "DEST_NAME=file-dest.txt",
                "DEST_FILE_NAME=${DEST_NAME}",
                "MANIFEST_FILE=test-manifest.json",
                "KEY_CONTEXT1=VAR1",
                "KEY_CONTEXT2=VAR2",
                "KEY_CONTEXT3=VAR3",
                "KEY_CONTEXT4=VAR4"
            ]
        }"#;
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "bitbake",
            "recipes": [
                "${BITBAKE_IMAGE}"
            ],
            "artifacts": [
                {
                    "type": "archive",
                    "name": "${ARCHIVE_NAME}",
                    "artifacts": [
                        {
                            "source": "file3.txt",
                            "dest": "file4.txt"
                        },
                        {
                            "type": "manifest",
                            "name": "${MANIFEST_FILE}",
                            "content": {
                                "${KEY_CONTEXT1}": "value1",
                                "${KEY_CONTEXT2}": "value2",
                                "${KEY_CONTEXT3}": "value3",
                                "data": {
                                    "${KEY_CONTEXT4}": "value4"
                                }
                            }
                        },
                        {
                            "type": "directory",
                            "name": "${DIR_NAME}",
                            "artifacts": [
                                {
                                    "source": "file1.txt",
                                    "dest": "${DEST_NAME}"
                                },
                                {
                                    "source": "${FILE_NAME}",
                                    "dest": "${DEST_NAME}"
                                },
                                {
                                    "source": "file3.txt",
                                    "dest": "${DEST_NAME}"
                                }
                            ]
                        }
                    ]
                }
            ]
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let mut task: WsTaskHandler = WsTaskHandler::from_str(json_task_str, &build_data).expect("Failed to parse Task config");
        task.expand_ctx(build_data.context().ctx());
        assert_eq!(task.data().build_dir(), &PathBuf::from("/workspace/builds/test-name"));
        assert!(task.data().condition());
        assert_eq!(task.data().name(), "task-name");
        assert_eq!(task.data().ttype(), &TType::Bitbake);
        assert_eq!(task.data().recipes(), &vec!["test-image".to_string()]);
        assert!(!task.data().disabled());
        let artifacts: &WsArtifactsHandler = task.artifacts().first().unwrap();
        assert_eq!(artifacts.data().atype(), &AType::Archive);
        assert_eq!(artifacts.data().name(), "test.zip");
        assert!(!artifacts.artifacts().is_empty());
        let archive_artifacts: &Vec<WsArtifactsHandler> = artifacts.artifacts();
        assert_eq!(archive_artifacts.get(0).unwrap().data().atype(), &AType::File);
        assert_eq!(archive_artifacts.get(0).unwrap().data().source(), &PathBuf::from("/workspace/builds/test-name/file3.txt"));
        assert_eq!(archive_artifacts.get(0).unwrap().data().dest(), &PathBuf::from("/workspace/artifacts/file4.txt"));
        assert_eq!(archive_artifacts.get(1).unwrap().data().name(), "test-manifest.json");
        assert!(!archive_artifacts.get(1).unwrap().data().manifest().is_empty());
        assert_eq!(archive_artifacts.get(1).unwrap().data().manifest(), "{\"VAR1\":\"value1\",\"VAR2\":\"value2\",\"VAR3\":\"value3\",\"data\":{\"VAR4\":\"value4\"}}");
        assert_eq!(archive_artifacts.get(2).unwrap().data().atype(), &AType::Directory);
        assert_eq!(archive_artifacts.get(2).unwrap().data().name(), "dir-name");
        let dir_artifacts: &Vec<WsArtifactsHandler> = archive_artifacts.get(2).unwrap().artifacts();
        let mut i: usize = 1;
        dir_artifacts.iter().for_each(|f| {
            assert_eq!(f.data().atype(), &AType::File);
            assert_eq!(f.data().source(), &PathBuf::from(format!("/workspace/builds/test-name/file{}.txt", i)));
            assert_eq!(f.data().dest(), &PathBuf::from("/workspace/artifacts/file-dest.txt"));
            i += 1;
        });
    }

    #[test]
    fn test_ws_task_run_bitbake() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let work_dir: PathBuf = PathBuf::from(path);
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "bitbake",
            "recipes": [
                "test-image"
            ]
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let task: WsTaskHandler = WsTaskHandler::from_str(json_task_str, &build_data).expect("Failed to parse Task config");
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec!["cd", &format!("{}/builds/NA", work_dir.to_string_lossy().to_string()), "&&", "bitbake", "test-image"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        let cli: Cli = Cli::new(
            Box::new(BLogger::new()),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            None,
        );
        task.run(&cli, &build_data, &vec![], &HashMap::new(), false, false).expect("Failed to run task!");
    }

    #[test]
    fn test_ws_task_run_docker() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let work_dir: PathBuf = PathBuf::from(path);
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "type": "bitbake",
            "docker": "test-registry/test-image:0.1",
            "recipes": [
                "test-image"
            ]
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let task: WsTaskHandler = WsTaskHandler::from_str(json_task_str, &build_data).expect("Failed to parse Task config");
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec!["docker", "run", "test-registry/test-image:0.1", "cd", &format!("{}/builds/NA", work_dir.to_string_lossy().to_string()), "&&", "bitbake", "test-image"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        let cli: Cli = Cli::new(
            Box::new(BLogger::new()),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            None,
        );
        task.run(&cli, &build_data, &vec![], &HashMap::new(), false, false).expect("Failed to run task!");
    }

    #[test]
    fn test_ws_task_run_recipes() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let path: &Path = temp_dir.path();
        let work_dir: PathBuf = PathBuf::from(path);
        let json_task_str: &str = r#"
        { 
            "index": "2",
            "name": "task-name",
            "recipes": [
                "image:sdk",
                "image"
            ]
        }"#;
        let build_data: WsBuildData = Helper::setup_build_data(&work_dir, None, None);
        let task: WsTaskHandler = WsTaskHandler::from_str(json_task_str, &build_data).expect("Failed to parse Task config");
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec!["cd", &format!("{}/builds/NA", work_dir.to_string_lossy().to_string()), "&&", "bitbake", "image"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec!["cd", &format!("{}/builds/NA", work_dir.to_string_lossy().to_string()), "&&", "bitbake", "image", "-c", "do_populate_sdk"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        let cli: Cli = Cli::new(
            Box::new(BLogger::new()),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            None,
        );
        task.run(&cli, &build_data, &vec![], &HashMap::new(), false, false).expect("Failed to run task!");
    }
}
