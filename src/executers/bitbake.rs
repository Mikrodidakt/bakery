use crate::cli::Cli;
use crate::data::{WsBitbakeData, WsTaskData};
use crate::error::BError;
use crate::executers::{
    TaskExecuter,
    Recipe,
    Docker,
    DockerImage,
};
use crate::fs::BitbakeConf;

use std::collections::HashMap;

pub struct BitbakeExecuter<'a> {
    bb_data: &'a WsBitbakeData,
    task_data: &'a WsTaskData,
    bb_variables: &'a Vec<String>,
    cli: &'a Cli,
}

impl<'a> TaskExecuter for BitbakeExecuter<'a> {
    fn exec(&self, env_variables: &HashMap<String, String>, dry_run: bool, interactive: bool) -> Result<(), BError> {
        let force: bool = dry_run;
        let env: HashMap<String, String> = self.bb_build_env(self.task_data, env_variables)?;
        // if we are running a dry run we should always create the bb configs.
        // When not a dry run it will be determined if it is needed or not to
        // regenerate the bb configs based on the content of the existing configs
        // comparted to the new content
        let conf: BitbakeConf = BitbakeConf::new(self.bb_data, self.bb_variables, force);
        conf.create_bitbake_configs(self.cli)?;

        if dry_run {
            self.cli.info("Dry run. Skipping build!".to_string());
            return Ok(());
        }

        for r in self.task_data.recipes() {
            let recipe: Recipe = Recipe::new(r);
            let mut cmd: Vec<String> = recipe.bitbake_cmd();
            let exec_dir: String = self.bb_data.build_dir().to_string_lossy().to_string();
            let mut cmd_line: Vec<String> = vec![];
            cmd_line.append(&mut vec![
                "cd".to_string(),
                exec_dir.clone(),
                "&&".to_string(),
            ]);
            cmd_line.append(&mut cmd);

            // If docker image is set specifically for the task we use that if not we check and
            // see if there is a docker image set in the bb node for the build config which will
            // then be used for all the bitbake tasks. If that is not set then we skip execute
            // docker
            let mut docker_str: &str = "";
            if !self.task_data.docker_image().is_empty() && self.task_data.docker_image() != "NA" {
                docker_str = self.task_data.docker_image();
            } else if !self.bb_data.docker_image().is_empty() && self.bb_data.docker_image() != "NA" {
                docker_str = self.bb_data.docker_image();
            }
            
            if !docker_str.is_empty() {
                let image: DockerImage = DockerImage::new(docker_str);
                let docker: Docker = Docker::new(image, interactive);
                docker.run_cmd(&mut cmd_line, &env, exec_dir, &self.cli)?;
            } else {
                self.cli.check_call(&cmd_line, &env, true)?;
            }
        }
        Ok(())
    }
}

impl<'a> BitbakeExecuter<'a> {
    fn bb_build_env(&self, _task_data: &WsTaskData, _env_variables: &HashMap<String, String>) -> Result<HashMap<String, String>, BError> {
        //let task_env = self.env();
        //let os_env = env::vars();
        Ok(HashMap::new())
    }

    pub fn new(cli: &'a Cli, task_data: &'a WsTaskData, bb_data: &'a WsBitbakeData, bb_variables: &'a Vec<String>) -> Self {
        BitbakeExecuter {
            cli,
            bb_data,
            task_data,
            bb_variables,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempdir::TempDir;

    use crate::cli::*;
    use crate::executers::{BitbakeExecuter, TaskExecuter};
    use crate::data::{WsBuildData, WsTaskData};
    use crate::helper::Helper;

    #[test]
    fn test_bitbake_executer() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = temp_dir.into_path();
        let build_dir: PathBuf = work_dir.join("builds/default");
        let bb_variables: Vec<String> = vec![];
        let env_variables: HashMap<String, String> = HashMap::new();
        let json_build_config: &str = r#"
        {
            "version": "4",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {
                "machine": "raspberrypi3",
                "variant": "release",
                "distro": "strix",
                "bblayersconf": [
                    "LCONF_VERSION=\"7\"",
                    "BBPATH=\"${TOPDIR}\""
                ],
                "localconf": [
                    "BB_NUMBER_THREADS ?= \"${@oe.utils.cpu_count()}\"",
                    "PARALLEL_MAKE ?= \"-j ${@oe.utils.cpu_count()}\""
                ]
            }
        }"#;
        let json_task_config: &str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "recipes": [
                "test-image"
            ]
        }"#;
        let data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let task_data: WsTaskData = WsTaskData::from_str(json_task_config, &data).expect("Failed to parse task config");
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate local.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate bblayers.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq(format!("cd {} && bitbake test-image", &build_dir.to_string_lossy().to_string())))
            .once()
            .returning(|_x| ());
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec!["cd", &build_dir.to_string_lossy().to_string(), "&&", "bitbake", "test-image"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        let cli: Cli = Cli::new(
            Box::new(mocked_logger),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            Some(vec!["bakery"]),
        );
        let executer: BitbakeExecuter = BitbakeExecuter::new(&cli, &task_data, data.bitbake(), &bb_variables);
        executer.exec(&env_variables, false, true).expect("Failed to execute task");
    }

    #[test]
    fn test_bitbake_executer_dry_run() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = temp_dir.into_path();
        let build_dir: PathBuf = work_dir.join("builds/default");
        let bb_variables: Vec<String> = vec![];
        let env_variables: HashMap<String, String> = HashMap::new();
        let json_build_config: &str = r#"
        {
            "version": "4",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {
                "machine": "raspberrypi3",
                "variant": "release",
                "distro": "strix",
                "bblayersconf": [
                    "LCONF_VERSION=\"7\"",
                    "BBPATH=\"${TOPDIR}\""
                ],
                "localconf": [
                    "BB_NUMBER_THREADS ?= \"${@oe.utils.cpu_count()}\"",
                    "PARALLEL_MAKE ?= \"-j ${@oe.utils.cpu_count()}\""
                ]
            }
        }"#;
        let json_task_config: &str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "recipes": [
                "test-image"
            ]
        }"#;
        let data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let task_data: WsTaskData = WsTaskData::from_str(json_task_config, &data).expect("Failed to parse task config");
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate local.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate bblayers.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Dry run. Skipping build!".to_string()))
            .once()
            .returning(|_x| ());
        let cli: Cli = Cli::new(
            Box::new(mocked_logger),
            Box::new(BSystem::new()),
            clap::Command::new("bakery"),
            Some(vec!["bakery"]),
        );
        let executer: BitbakeExecuter = BitbakeExecuter::new(&cli, &task_data, data.bitbake(), &bb_variables);
        executer.exec(&env_variables, true, true).expect("Failed to execute task");
    }

    #[test]
    fn test_bitbake_executer_docker() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = temp_dir.into_path();
        let build_dir: PathBuf = work_dir.join("builds/default");
        let bb_variables: Vec<String> = vec![];
        let env_variables: HashMap<String, String> = HashMap::new();
        let json_build_config: &str = r#"
        {
            "version": "4",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {
                "machine": "raspberrypi3",
                "variant": "release",
                "distro": "strix",
                "bblayersconf": [
                    "LCONF_VERSION=\"7\"",
                    "BBPATH=\"${TOPDIR}\""
                ],
                "localconf": [
                    "BB_NUMBER_THREADS ?= \"${@oe.utils.cpu_count()}\"",
                    "PARALLEL_MAKE ?= \"-j ${@oe.utils.cpu_count()}\""
                ]
            }
        }"#;
        let json_task_config: &str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "docker": "test-registry/task-docker:0.1",
            "recipes": [
                "test-image"
            ]
        }"#;
        let data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let task_data: WsTaskData = WsTaskData::from_str(json_task_config, &data).expect("Failed to parse task config");
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate local.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate bblayers.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq(format!("docker run test-registry/task-docker:0.1 cd {} && bitbake test-image", &build_dir.to_string_lossy().to_string())))
            .once()
            .returning(|_x| ());
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec!["docker", "run", "test-registry/task-docker:0.1", "cd", &build_dir.to_string_lossy().to_string(), "&&", "bitbake", "test-image"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        let cli: Cli = Cli::new(
            Box::new(mocked_logger),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            Some(vec!["bakery"]),
        );
        let executer: BitbakeExecuter = BitbakeExecuter::new(&cli, &task_data, data.bitbake(), &bb_variables);
        executer.exec(&env_variables, false, true).expect("Failed to execute task");
    }

    #[test]
    fn test_bitbake_executer_bb_docker() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = temp_dir.into_path();
        let build_dir: PathBuf = work_dir.join("builds/default");
        let bb_variables: Vec<String> = vec![];
        let env_variables: HashMap<String, String> = HashMap::new();
        let json_build_config: &str = r#"
        {
            "version": "4",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {
                "machine": "raspberrypi3",
                "variant": "release",
                "distro": "strix",
                "docker": "test-registry/bb-docker:0.1",
                "bblayersconf": [
                    "LCONF_VERSION=\"7\"",
                    "BBPATH=\"${TOPDIR}\""
                ],
                "localconf": [
                    "BB_NUMBER_THREADS ?= \"${@oe.utils.cpu_count()}\"",
                    "PARALLEL_MAKE ?= \"-j ${@oe.utils.cpu_count()}\""
                ]
            }
        }"#;
        let json_task_config: &str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "recipes": [
                "test-image"
            ]
        }"#;
        let data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let task_data: WsTaskData = WsTaskData::from_str(json_task_config, &data).expect("Failed to parse task config");
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate local.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate bblayers.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq(format!("docker run test-registry/bb-docker:0.1 cd {} && bitbake test-image", &build_dir.to_string_lossy().to_string())))
            .once()
            .returning(|_x| ());
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec!["docker", "run", "test-registry/bb-docker:0.1", "cd", &build_dir.to_string_lossy().to_string(), "&&", "bitbake", "test-image"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        let cli: Cli = Cli::new(
            Box::new(mocked_logger),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            Some(vec!["bakery"]),
        );
        let executer: BitbakeExecuter = BitbakeExecuter::new(&cli, &task_data, data.bitbake(), &bb_variables);
        executer.exec(&env_variables, false, true).expect("Failed to execute task");
    }

    #[test]
    fn test_bitbake_executer_docker_order() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = temp_dir.into_path();
        let build_dir: PathBuf = work_dir.join("builds/default");
        let bb_variables: Vec<String> = vec![];
        let env_variables: HashMap<String, String> = HashMap::new();
        let json_build_config: &str = r#"
        {
            "version": "4",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {
                "machine": "raspberrypi3",
                "variant": "release",
                "distro": "strix",
                "docker": "test-registry/bb-docker:0.1",
                "bblayersconf": [
                    "LCONF_VERSION=\"7\"",
                    "BBPATH=\"${TOPDIR}\""
                ],
                "localconf": [
                    "BB_NUMBER_THREADS ?= \"${@oe.utils.cpu_count()}\"",
                    "PARALLEL_MAKE ?= \"-j ${@oe.utils.cpu_count()}\""
                ]
            }
        }"#;
        let json_task_config: &str = r#"
        {
            "index": "0",
            "name": "task1-name",
            "docker": "test-registry/task-docker:0.1",
            "recipes": [
                "test-image"
            ]
        }"#;
        let data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let task_data: WsTaskData = WsTaskData::from_str(json_task_config, &data).expect("Failed to parse task config");
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate local.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq("Autogenerate bblayers.conf".to_string()))
            .once()
            .returning(|_x| ());
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq(format!("docker run test-registry/task-docker:0.1 cd {} && bitbake test-image", &build_dir.to_string_lossy().to_string())))
            .once()
            .returning(|_x| ());
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec!["docker", "run", "test-registry/task-docker:0.1", "cd", &build_dir.to_string_lossy().to_string(), "&&", "bitbake", "test-image"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                env: HashMap::new(),
                shell: true,
            }))
            .once()
            .returning(|_x| Ok(()));
        let cli: Cli = Cli::new(
            Box::new(mocked_logger),
            Box::new(mocked_system),
            clap::Command::new("bakery"),
            Some(vec!["bakery"]),
        );
        let executer: BitbakeExecuter = BitbakeExecuter::new(&cli, &task_data, data.bitbake(), &bb_variables);
        executer.exec(&env_variables, false, true).expect("Failed to execute task");
    }
}