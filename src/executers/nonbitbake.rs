use crate::cli::Cli;
use crate::error::BError;
use crate::data::WsTaskData;
use crate::executers::{
    TaskExecuter,
    Docker,
    DockerImage,
};

use std::collections::HashMap;

pub struct NonBitbakeExecuter<'a> {
    cli: &'a Cli,
    task_data: &'a WsTaskData,
}

impl<'a> TaskExecuter for NonBitbakeExecuter<'a> {
    fn exec(&self, env_variables: &HashMap<String, String>, dry_run: bool, interactive: bool) -> Result<(), BError> {
        let force: bool = dry_run;

        if dry_run {
            self.cli.info("Dry run. Skipping build!".to_string());
            return Ok(());
        }

        let exec_dir: String = self.task_data.build_dir().to_string_lossy().to_string();
        let mut cmd_line: Vec<String> = vec!["cd".to_string(), exec_dir.clone(), "&&".to_string()];
        let mut cmd: Vec<String> = self.task_data.build_cmd().split(' ').map(|c| c.to_string()).collect();
        cmd_line.append(&mut cmd);

        if !self.task_data.docker_image().is_empty() && self.task_data.docker_image() != "NA" {
            let image: DockerImage = DockerImage::new(self.task_data.docker_image());
            let docker: Docker = Docker::new(image, interactive);
            docker.run_cmd(&mut cmd_line, env_variables, exec_dir, &self.cli)?;
        } else {
            self.cli.check_call(&cmd_line, env_variables, true)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempdir::TempDir;

    use crate::cli::*;
    use crate::executers::{NonBitbakeExecuter, TaskExecuter};
    use crate::data::{WsBuildData, WsTaskData};
    use crate::helper::Helper;

    #[test]
    fn test_nonbitbake_executer() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = temp_dir.into_path();
        let build_dir: PathBuf = work_dir.join("test-dir");
        let env_variables: HashMap<String, String> = HashMap::new();
        let json_build_config: &str = r#"
        {
            "version": "4",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch"
        }"#;
        let json_task_config: &str = r#"
        {
            "index": "1",
            "name": "task-name",
            "type": "non-bitbake",
            "builddir": "test-dir",
            "build": "test.sh",
            "clean": "rm -rf test-dir"
        }"#;
        let data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let task_data: WsTaskData = WsTaskData::from_str(json_task_config, &data).expect("Failed to parse task config");
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec!["cd", &build_dir.to_string_lossy().to_string(), "&&", "test.sh"]
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
            Some(vec!["bakery"]),
        );
        let executer: NonBitbakeExecuter = NonBitbakeExecuter::new(&cli, &task_data);
        executer.exec(&env_variables, false, true).expect("Failed to execute task");
    }

    #[test]
    fn test_nonbitbake_executer_dry_run() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = temp_dir.into_path();
        let env_variables: HashMap<String, String> = HashMap::new();
        let json_build_config: &str = r#"
        {
            "version": "4",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch"
        }"#;
        let json_task_config: &str = r#"
        {
            "index": "1",
            "name": "task-name",
            "type": "non-bitbake",
            "builddir": "test-dir",
            "build": "test.sh",
            "clean": "rm -rf test-dir"
        }"#;
        let data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let task_data: WsTaskData = WsTaskData::from_str(json_task_config, &data).expect("Failed to parse task config");
        let mut mocked_logger: MockLogger = MockLogger::new();
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
        let executer: NonBitbakeExecuter = NonBitbakeExecuter::new(&cli, &task_data);
        executer.exec(&env_variables, true, true).expect("Failed to execute task");
    }

    #[test]
    fn test_bitbake_executer_docker() {
        let temp_dir: TempDir = TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = temp_dir.into_path();
        let build_dir: PathBuf = work_dir.join("test-dir");
        let env_variables: HashMap<String, String> = HashMap::new();
        let json_build_config: &str = r#"
        {
            "version": "4",
            "name": "default",
            "description": "Test Description",
            "arch": "test-arch"
        }"#;
        let json_task_config: &str = r#"
        {
            "index": "1",
            "name": "task-name",
            "type": "non-bitbake",
            "docker": "test-registry/task-docker:0.1",
            "builddir": "test-dir",
            "build": "test.sh",
            "clean": "rm -rf test-dir"
        }"#;
        let data: WsBuildData = Helper::setup_build_data(&work_dir, Some(json_build_config), None);
        let task_data: WsTaskData = WsTaskData::from_str(json_task_config, &data).expect("Failed to parse task config");
        let mut mocked_system: MockSystem = MockSystem::new();
        mocked_system
            .expect_check_call()
            .with(mockall::predicate::eq(CallParams {
                cmd_line: vec!["docker", "run", "test-registry/task-docker:0.1", "cd", &build_dir.to_string_lossy().to_string(), "&&", "test.sh"]
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
            Some(vec!["bakery"]),
        );
        let executer: NonBitbakeExecuter = NonBitbakeExecuter::new(&cli, &task_data);
        executer.exec(&env_variables, false, true).expect("Failed to execute task");
    }
}

impl<'a> NonBitbakeExecuter<'a> {
    pub fn new(cli: &'a Cli, task_data: &'a WsTaskData) -> Self {
        NonBitbakeExecuter {
            cli,
            task_data,
        }
    }
}