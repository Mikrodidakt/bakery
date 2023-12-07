use clap::ArgMatches;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use crate::cli::Cli;
use crate::error::BError;

pub struct Docker {
    image: DockerImage,
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

impl DockerImage {
    pub fn new(image_str: &str) -> Self {
        let mut split: Vec<String> = image_str.split('/').map(|c| c.to_string()).collect();
        let registry: String = split[0].clone();
        split = split[1].split(':').map(|c| c.to_string()).collect();
        let image: String = split[0].clone();
        let tag: String = split[1].clone();
        DockerImage {
            registry,
            image,
            tag,
        }
    }
}

impl Docker {
    pub fn inside_docker() -> bool {
        let path: PathBuf = PathBuf::from("/.dockerenv");
        // Potentially it would be better to use try_exists
        // for now lets just use exists
        path.exists()
    }

    pub fn new(image: DockerImage, interactive: bool) -> Self {
        Docker {
            image,
            _interactive: interactive,
        }
    }

    pub fn bootstrap_bakery(&self, _args: &ArgMatches) -> Result<(), BError> {
        Ok(())
    }

    pub fn docker_cmd_line(&self, cmd_line: &mut Vec<String>, _dir: String) -> Vec<String> {
        let mut docker_cmd: Vec<String> = vec!["docker".to_string(), "run".to_string()];
        docker_cmd.push(format!("{}", self.image));
        docker_cmd.append(cmd_line);
        docker_cmd
    }

    pub fn run_cmd(&self, cmd_line: &mut Vec<String>, env: &HashMap<String, String>, _dir: String, cli: &Cli,) -> Result<(), BError> {
        cli.check_call(&self.docker_cmd_line(cmd_line, _dir), &env, true)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::cli::*;
    use crate::error::BError;
    use crate::executers::{Docker, DockerImage};

    fn helper_test_docker(verification_str: &String, test_cmd: &String, test_work_dir: Option<String>,
        image: DockerImage) -> Result<(), BError> {
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
        let docker: Docker = Docker::new(image, true);
        let mut cmd: Vec<String> = test_cmd.split(' ').map(|c| c.to_string()).collect();
        docker.run_cmd(&mut cmd, &HashMap::new(), test_work_dir.unwrap(), &cli)
    }

    #[test]
    fn test_executer_docker_cmd_line() {
        let test_work_dir: String = String::from("test_work_dir");
        let test_cmd: String = String::from("test_cmd");
        let docker_image: DockerImage = DockerImage::new("test-registry/test-image:0.1");
        let docker: Docker = Docker::new(docker_image.clone(), true);
        let cmd: Vec<String> = docker.docker_cmd_line(&mut vec!["cd".to_string(), test_work_dir.clone(), test_cmd.clone()], test_work_dir.clone());
        assert_eq!(cmd, vec!["docker".to_string(), "run".to_string(), format!("{}", docker_image), "cd".to_string(), test_work_dir.clone(), test_cmd])
    }

    #[test]
    fn test_docker_run() {
        let test_work_dir = String::from("test_work_dir");
        let test_build_dir = String::from("test_build_dir");
        let test_cmd: String = format!("cd {} && test", test_build_dir);
        let docker_image: DockerImage = DockerImage::new("test-registry/test-image:0.1");
        let verification_str = format!("docker run {} {}", docker_image, test_cmd);
        let result = helper_test_docker(
            &verification_str,
            &test_cmd,
            Some(test_work_dir),
            docker_image
        );
        match result {
            Err(err) => {
                assert_eq!("Docker run failed", err.to_string());
            }
            Ok(()) => {}
        }
    }
}
