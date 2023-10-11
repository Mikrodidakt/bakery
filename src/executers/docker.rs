use clap::ArgMatches;
use std::collections::HashMap;
use std::env::Vars;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::cli::Cli;
use crate::error::BError;
use crate::workspace::Workspace;

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
    pub fn inside_docker() -> bool {
        let path: PathBuf = PathBuf::from("/.dockerenv");
        // Potentially it would be better to use try_exists
        // for now lets just use exists
        path.exists()
    }

    pub fn new(workspace: &'a Workspace, image: &'a DockerImage, interactive: bool) -> Self {
        Docker {
            _workspace: workspace,
            image,
            _interactive: interactive,
        }
    }

    pub fn bootstrap_bakery(&self, args: &ArgMatches) -> Result<(), BError> {
        Ok(())
    }

    pub fn docker_cmd_line(&self, cmd_line: &mut Vec<String>, dir: String) -> Vec<String> {
        let mut docker_cmd: Vec<String> = vec!["docker".to_string(), "run".to_string()];
        docker_cmd.push(format!("{}", self.image));
        docker_cmd.append(cmd_line);
        docker_cmd
    }

    pub fn run_cmd(
        &self,
        cmd_line: &mut Vec<String>,
        env: Vars,
        _dir: String,
        cli: &Cli,
    ) -> Result<(), BError> {
        let env: HashMap<String, String> = env.map(|(key, value)| (key, value)).collect();

        cli.check_call(&self.docker_cmd_line(cmd_line, _dir), &env, true)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::cli::*;
    use crate::error::BError;
    use crate::executers::{Docker, DockerImage, Executer};
    use crate::workspace::{Workspace, WsBuildConfigHandler, WsSettingsHandler};

    fn helper_test_docker(
        verification_str: &String,
        test_cmd: &String,
        test_work_dir: Option<String>,
        image: &DockerImage,
        workspace: &Workspace,
    ) -> Result<(), BError> {
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq(verification_str.clone()))
            .once()
            .returning(|_x| ());
        let cli: Cli = Cli::new(
            Box::new(mocked_logger),
            Box::new(BSystem::new()),
            clap::Command::new("bakery"),
            None,
        );
        let docker: Docker = Docker::new(&workspace, image, true);
        let mut cmd: Vec<String> = test_cmd.split(' ').map(|c| c.to_string()).collect();
        docker.run_cmd(&mut cmd, std::env::vars(), test_work_dir.unwrap(), &cli)
    }

    fn helper_test_executer(
        verification_str: &String,
        test_cmd: &String,
        test_build_dir: Option<String>,
        docker: Option<Docker>,
        workspace: &Workspace,
    ) -> Result<(), BError> {
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger
            .expect_info()
            .with(mockall::predicate::eq(verification_str.clone()))
            .once()
            .returning(|_x| ());
        let cli: Cli = Cli::new(
            Box::new(mocked_logger),
            Box::new(BSystem::new()),
            clap::Command::new("bakery"),
            None,
        );
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
        let verification_str = format!(
            "docker run {} cd {} && {}",
            docker_image, test_work_dir, test_cmd
        );
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
        let mut settings: WsSettingsHandler =
            WsSettingsHandler::from_str(&work_dir, json_ws_settings)
                .expect("Failed to parse settings.json");
        let config: WsBuildConfigHandler =
            WsBuildConfigHandler::from_str(json_build_config, &mut settings)
                .expect("Failed to parse build config");
        let workspace: Workspace = Workspace::new(Some(work_dir), Some(settings), Some(config))
            .expect("Failed to setup workspace");
        let docker: Docker = Docker::new(&workspace, &docker_image, true);
        let result: Result<(), BError> =
            helper_test_executer(&verification_str, &test_cmd, None, Some(docker), &workspace);
        match result {
            Err(err) => {
                assert_eq!("Executer failed", err.to_string());
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
        let verification_str = format!("docker run {} {}", docker_image, test_cmd);
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
        let mut settings: WsSettingsHandler =
            WsSettingsHandler::from_str(&work_dir, json_ws_settings)
                .expect("Failed to parse settings.json");
        let config: WsBuildConfigHandler =
            WsBuildConfigHandler::from_str(json_build_config, &mut settings)
                .expect("Failed to parse build config");
        let workspace: Workspace = Workspace::new(Some(work_dir), Some(settings), Some(config))
            .expect("Failed to setup workspace");
        let result = helper_test_docker(
            &verification_str,
            &test_cmd,
            Some(test_work_dir),
            &docker_image,
            &workspace,
        );
        match result {
            Err(err) => {
                assert_eq!("Docker run failed", err.to_string());
            }
            Ok(()) => {}
        }
    }
}
