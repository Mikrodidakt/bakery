use std::fmt;

use crate::workspace::Workspace;
use crate::error::BError;
use crate::cli::Cli;

pub struct Docker<'a> {
    _workspace: &'a Workspace,
    image: &'a DockerImage,
    _interactive: bool,
}

#[derive(Clone)]
pub struct DockerImage {
    pub image: String,
    pub tag: String,
    pub registry: String, 
}

impl fmt::Display for DockerImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}:{}", self.registry, self.image, self.tag)
    }
}

impl<'a> Docker<'a> {
    pub fn new(workspace: &'a Workspace, image: &'a DockerImage, interactive: bool) -> Self {
        Docker {
            _workspace: workspace,
            image,
            _interactive: interactive,
        }
    }

    pub fn run_cmd(&self, cmd_line: String, _dir: String, cli: &Cli) -> Result<(), BError> {
        cli.info(format!("Execute inside docker image {} '{}'", self.image , cmd_line));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::commands::build;
    use crate::executers::{Docker, DockerImage, Executer};
    use crate::workspace::{Workspace, WsSettingsHandler, WsBuildConfigHandler};
    use crate::configs::{WsSettings, BuildConfig};
    use crate::cli::*;
    use crate::error::BError;
    use crate::helper::Helper;

    fn helper_test_docker(verification_str: &String, test_cmd: &String, test_work_dir: Option<String>, image: &DockerImage, workspace: &Workspace) -> Result<(), BError> {
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger.expect_info().with(mockall::predicate::eq(verification_str.clone())).once().returning(|_x|());
        let cli: Cli = Cli::new(Box::new(mocked_logger), clap::Command::new("bakery"));
        let docker: Docker = Docker::new(&workspace, image, true);
        docker.run_cmd(test_cmd.clone(), test_work_dir.unwrap(), &cli)
    }

    fn helper_test_executer(verification_str: &String, test_cmd: &String, test_build_dir: Option<String>, docker: Option<Docker>, workspace: &Workspace) -> Result<(), BError> {
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger.expect_info().with(mockall::predicate::eq(verification_str.clone())).once().returning(|_x|());
        let cli: Cli = Cli::new(Box::new(mocked_logger), clap::Command::new("bakery"));
        let exec: Executer = Executer::new(workspace, &cli);
        exec.execute(&test_cmd, std::env::vars(), test_build_dir, docker, true) 
    }

    #[test]
    fn test_executer_docker() {
        let test_work_dir: String = String::from("test_work_dir");
        let test_cmd: String = String::from("test_cmd");
        let docker_image: DockerImage = DockerImage {
            registry: String::from("test-registry"),
            image: String::from("test-image"),
            tag: String::from("0.1"),
        };
        let verification_str = format!("Execute inside docker image {} 'cd {} && {}'", docker_image, test_work_dir, test_cmd);
        let work_dir: PathBuf = PathBuf::from(&test_work_dir);
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
        let mut settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_ws_settings).expect("Failed to parse settings.json");
        let config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut settings).expect("Failed to parse build config");
        let workspace: Workspace = Workspace::new(Some(work_dir), Some(settings), Some(config)).expect("Failed to setup workspace");
        let docker: Docker = Docker::new(&workspace, &docker_image, true);
        let result: Result<(), BError> = helper_test_executer(
            &verification_str,
            &test_cmd,
            None,
            Some(docker),
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
    fn test_docker_run() {
        let test_work_dir = String::from("test_work_dir");
        let test_build_dir = String::from("test_build_dir");
        let test_cmd = format!("cd {} && test", test_build_dir);
        let docker_image: DockerImage = DockerImage {
            registry: String::from("test-registry"),
            image: String::from("test-image"),
            tag: String::from("0.1"),
        };
        let verification_str = format!("Execute inside docker image {} '{}'", docker_image, test_cmd);
        let work_dir: PathBuf = PathBuf::from(test_work_dir.clone());
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
        let mut settings: WsSettingsHandler = WsSettingsHandler::from_str(&work_dir, json_ws_settings).expect("Failed to parse settings.json");
        let config: WsBuildConfigHandler = WsBuildConfigHandler::from_str(json_build_config, &mut settings).expect("Failed to parse build config");
        let workspace: Workspace = Workspace::new(Some(work_dir), Some(settings), Some(config)).expect("Failed to setup workspace");
        let result = helper_test_docker(
            &verification_str,
            &test_cmd,
            Some(test_work_dir),
            &docker_image,
            &workspace
        );
        match result {
            Err(err) => {
                assert_eq!("Docker run failed", err.message);
            }
            Ok(()) => {}
        }
    }
}

