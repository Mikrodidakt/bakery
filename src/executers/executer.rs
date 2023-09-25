use std::env::Vars;

use crate::error::BError;
use crate::executers::Docker;
use crate::workspace::Workspace;
use crate::cli::Cli;

pub struct Executer<'a> {
    workspace: &'a Workspace,
    cli: &'a Cli,
}

impl<'a> Executer<'a> {
    pub fn new(workspace: &'a Workspace, cli: &'a Cli) -> Self {
        Executer {
            workspace: workspace,
            cli: cli,
        }
    }

    pub fn execute(&self, cmd: &str, _env: Vars, dir: Option<String>, docker: Option<Docker>, interactive: bool) -> Result<(), BError> {
        let mut cmd_line: String = String::from(cmd);
        let exec_dir: String;

        // If no directory is specified we should use the Workspace work directory
        // as the directory to execute the command from
        match dir {
            Some(directory) => {
                cmd_line = format!("cd {} && {}", directory, cmd_line);
                exec_dir = directory;
            },
            None => {
                cmd_line = format!("cd {} && {}", self.workspace.settings().work_dir().to_str().unwrap(), cmd_line);
                exec_dir = self.workspace.settings().work_dir().to_str().unwrap().to_string();
            }
        }

        match docker {
            Some(docker) => {
                docker.run_cmd(cmd_line, exec_dir, &self.cli)?;
            },
            None => {
                self.cli.info(format!("Execute '{}'", cmd_line));
            }  
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::commands::build;
    use crate::executers::{Docker, DockerImage, Executer};
    use crate::workspace::Workspace;
    use crate::configs::{WsSettings, BuildConfig};
    use crate::cli::*;
    use crate::error::BError;
    use crate::helper::Helper;

    fn helper_test_docker(verification_str: &String, test_cmd: &String, test_work_dir: Option<String>, image: &DockerImage, workspace: &Workspace) -> Result<(), BError> {
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger.expect_info().with(mockall::predicate::eq(verification_str.clone())).once().returning(|_x|());
        let cli: Cli = Cli::new(Box::new(mocked_logger));
        let docker: Docker = Docker::new(&workspace, image, true);
        docker.run_cmd(test_cmd.clone(), test_work_dir.unwrap(), &cli)
    }

    fn helper_test_executer(verification_str: &String, test_cmd: &String, test_build_dir: Option<String>, docker: Option<Docker>, workspace: &Workspace) -> Result<(), BError> {
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger.expect_info().with(mockall::predicate::eq(verification_str.clone())).once().returning(|_x|());
        let cli: Cli = Cli::new(Box::new(mocked_logger));
        let exec: Executer = Executer::new(workspace, &cli);
        exec.execute(&test_cmd, std::env::vars(), test_build_dir, docker, true) 
    }

    #[test]
    fn test_executer_build_dir() {
        let test_work_dir = String::from("/test_work_dir");
        let test_build_dir = String::from("test_build_dir");
        let test_cmd = String::from("test_cmd");
        let verification_str = format!("Execute 'cd {} && {}'", test_build_dir, test_cmd);
        let work_dir: PathBuf = PathBuf::from(test_work_dir);
        let json_ws_settings: &str = r#"
        {
            "version": "4",
            "builds": {
                "supported": [
                    "default"
                ]
            }
        }"#;
        let json_build_config: &str = r#"
        {                                                                                                                   
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {}
        }
        "#;
        let ws_config: WsSettings = Helper::setup_ws_settings(json_ws_settings);
        let build_config: BuildConfig = Helper::setup_build_config(json_build_config);
        let workspace: Workspace = Workspace::new(Some(work_dir), Some(ws_config), Some(build_config)).expect("Failed to setup workspace");
        let result: Result<(), BError> = helper_test_executer(
            &verification_str,
            &test_cmd,
            Some(test_build_dir),
            None,
            &workspace
        );
        match result {
            Err(err) => {
                assert_eq!("Executer failed", err.message);
            }
            Ok(()) => {}
        }
    }

    #[test]
    fn test_executer_no_build_dir() {
        let test_work_dir = String::from("test_work_dir");
        let test_cmd = String::from("test_cmd");
        let verification_str = format!("Execute 'cd {} && {}'", test_work_dir, test_cmd);
        let work_dir: PathBuf = PathBuf::from(test_work_dir);
        let json_ws_settings: &str = r#"
        {
            "version": "4",
            "builds": {
                "supported": [
                    "default"
                ]
            }
        }"#;
        let json_build_config: &str = r#"
        {
            "version": "4",
            "name": "test-name",
            "description": "Test Description",
            "arch": "test-arch",
            "bb": {}
        }
        "#;
        let ws_config: WsSettings = Helper::setup_ws_settings(json_ws_settings);
        let build_config: BuildConfig = Helper::setup_build_config(json_build_config);
        let workspace: Workspace = Workspace::new(Some(work_dir), Some(ws_config), Some(build_config)).expect("Failed to setup workspace");
        let result: Result<(), BError> = helper_test_executer(
            &verification_str,
            &test_cmd,
            None,
            None,
            &workspace
        );
        match result {
            Err(err) => {
                assert_eq!("Executer failed", err.message);
            }
            Ok(()) => {}
        }
    }
}