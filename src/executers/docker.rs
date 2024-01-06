use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use users::Groups;

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
    fn env_home(&self) -> String {
        match std::env::var_os("HOME") {
            Some(var) => { 
                return var.into_string().or::<String>(Ok(String::from(""))).unwrap();
            },
            None => {
                return String::new();
            }
        }    
    }

    fn user(&self) -> Vec<String> {
        vec![
            String::from("-u"),
            format!("{}:{}", users::get_current_uid(), users::get_current_gid()),
        ]
    }

    fn etc_files(&self) -> Vec<String> {
        vec![
            String::from("-v"),
            String::from("/etc/passwd:/etc/passwd"),
            String::from("-v"),
            String::from("/etc/group:/etc/group"),
        ]
    }

    fn hidden_home_files(&self) -> Vec<String> {
        vec![
            String::from("-v"),
            format!("{}/.gitconfig:{}/.gitconfig", self.env_home(), self.env_home()),
            String::from("-v"),
            format!("{}/.ssh:{}/.ssh", self.env_home(), self.env_home()),
            String::from("-v"),
            format!("{}/.bashrc:{}/.bashrc", self.env_home(), self.env_home()),
            String::from("-v"),
            format!("{}/.docker:{}/.docker", self.env_home(), self.env_home()),
        ]
    }

    fn home_dir(&self) -> Vec<String> {
        vec![
            String::from("-v"),
            format!("{}:{}", self.env_home(), self.env_home()),
        ]
    }

    fn work_dir(&self, dir: &PathBuf) -> Vec<String> {
        vec![
            String::from("-w"),
            format!("{}", dir.display()),
        ]
    }

    fn docker_sock(&self) -> Vec<String> {
        vec![
            String::from("-v"),
            String::from("/var/run/docker.sock:/var/run/docker.sock"),
        ]
    }

    fn group(&self) -> Vec<String> {
        let cache: users::UsersCache = users::UsersCache::new();
        vec![
            String::from("--group-add"),
            cache.get_group_by_name("docker").unwrap().gid().to_string(),
        ]
    }

    fn env_file(&self) -> Vec<String> {
        vec![]
    }

    fn volumes(&self, volumes: &Vec<String>) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        volumes.iter().for_each(|e| {
            println!("{}", e);
            v.append(&mut vec![
                String::from("-v"),
                e.to_string(),
            ]);
        });
        v.append(&mut self.etc_files());
        v.append(&mut self.hidden_home_files());
        v.append(&mut self.docker_sock());
        v
    }

    fn container_name(&self, name: &str) -> Vec<String> {
        vec![
            String::from("--name"),
            format!("{}-{}", name, std::process::id()),
        ]
    }

    fn top_dir(&self, dir: &PathBuf) -> Vec<String> {
        vec![
            String::from("-v"),
            format!("{}:{}", dir.display(), dir.display()),
        ]
    }

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

    pub fn bootstrap_cmd_line(&self, cmd_line: &Vec<String>, docker_top_dir: &PathBuf, work_dir: &PathBuf, docker_args: &Vec<String>, volumes: &Vec<String>) -> Vec<String> {
        let mut docker_cmd: Vec<String> = vec!["docker".to_string(), "run".to_string()];
        docker_cmd.append(&mut self.container_name("bakery-workspace"));
        docker_cmd.append(&mut vec!["-t".to_string(), "--rm".to_string()]);
        if self._interactive {
            docker_cmd.push("-i".to_string());
        }
        docker_cmd.append(&mut self.group());
        docker_cmd.append(&mut self.volumes(volumes));
        docker_cmd.append(&mut self.user());
        docker_cmd.append(&mut self.top_dir(docker_top_dir));
        docker_cmd.append(&mut self.work_dir(work_dir));
        if !docker_args.is_empty() {
            docker_cmd.append(&mut docker_args.clone());
        }
        docker_cmd.push(format!("{}", self.image));
        docker_cmd.append(&mut cmd_line.clone());
        docker_cmd
    }

    pub fn cmd_line(&self, cmd_line: &mut Vec<String>, dir: &PathBuf) -> Vec<String> {
        let mut docker_cmd: Vec<String> = vec!["docker".to_string(), "run".to_string()];
        docker_cmd.append(&mut self.user());
        docker_cmd.append(&mut self.etc_files());
        docker_cmd.append(&mut self.home_dir());
        docker_cmd.append(&mut self.work_dir(dir));
        docker_cmd.append(&mut vec!["-t".to_string(), "--rm".to_string()]);
        if self._interactive {
            docker_cmd.push("-i".to_string());
        }
        docker_cmd.append(&mut self.group());
        docker_cmd.append(&mut self.env_file());
        docker_cmd.push(format!("{}", self.image));
        docker_cmd.append(cmd_line);
        docker_cmd
    }

    pub fn bootstrap_bakery(&self, cli: &Cli, docker_top_dir: &PathBuf, work_dir: &PathBuf, docker_args: &Vec<String>, volumes: &Vec<String>) -> Result<(), BError> {
        cli.check_call(&self.bootstrap_cmd_line(&mut cli.get_cmd_line(), docker_top_dir, work_dir, docker_args, volumes), &HashMap::new(), true)?;
        Ok(())
    }

    pub fn run_cmd(&self, cmd_line: &mut Vec<String>, env: &HashMap<String, String>, exec_dir: &PathBuf, cli: &Cli) -> Result<(), BError> {
        cli.check_call(&self.cmd_line(cmd_line, exec_dir), &env, true)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use tempdir::TempDir;
    use users::Groups;

    use crate::executers::{Docker, DockerImage};
    use crate::helper::Helper;

    fn helper_env_home() -> String {
        match std::env::var_os("HOME") {
            Some(var) => { 
                return var.into_string().or::<String>(Ok(String::from(""))).unwrap();
            },
            None => {
                return String::new();
            }
        }    
    }

    fn helper_docker_bootstrap_string(interactive: bool, args: &Vec<String>, volumes: &Vec<String>, top_dir: &PathBuf, work_dir: &PathBuf, image: &DockerImage, cmd: &Vec<String>) -> Vec<String>{
        let mut cmd_line: Vec<String> = vec![
            String::from("docker"),
            String::from("run"),
            String::from("--name"),
            format!("bakery-workspace-{}", std::process::id()),
            String::from("-t"),
            String::from("--rm"),
        ];
        if interactive {
            cmd_line.push("-i".to_string());
        }
        let cache: users::UsersCache = users::UsersCache::new();
        cmd_line.append(&mut vec![
            String::from("--group-add"),
            cache.get_group_by_name("docker").unwrap().gid().to_string(),
        ]);
        if !volumes.is_empty() {
            volumes.iter().for_each(|v| {
                cmd_line.append(&mut vec![
                    String::from("-v"),
                    v.to_string(),
                ]);
            })
        }
        cmd_line.append(&mut vec![
            String::from("-v"),
            String::from("/etc/passwd:/etc/passwd"),
            String::from("-v"),
            String::from("/etc/group:/etc/group"),
            String::from("-v"),
            format!("{}/.gitconfig:{}/.gitconfig", helper_env_home(), helper_env_home()),
            String::from("-v"),
            format!("{}/.ssh:{}/.ssh", helper_env_home(), helper_env_home()),
            String::from("-v"),
            format!("{}/.bashrc:{}/.bashrc", helper_env_home(), helper_env_home()),
            String::from("-v"),
            format!("{}/.docker:{}/.docker", helper_env_home(), helper_env_home()),
            String::from("-v"),
            String::from("/var/run/docker.sock:/var/run/docker.sock"),
            String::from("-u"),
            format!("{}:{}", users::get_current_uid(), users::get_current_gid()),
            String::from("-v"),
            format!("{}:{}", top_dir.display(), top_dir.display()),
        ]);
        cmd_line.append(&mut vec![
            String::from("-w"),
            format!("{}", work_dir.display()),
        ]);
        if !args.is_empty() {
            cmd_line.append(&mut args.clone());
        }
        cmd_line.push(format!("{}", image));
        cmd_line.append(&mut cmd.clone());
        cmd_line
    }

    #[test]
    fn test_docker_bootstrap_cmdline() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let docker_top_dir: PathBuf = work_dir.clone();
        let test_build_dir: PathBuf = work_dir.join(PathBuf::from("test_build_dir"));
        let test_cmd: Vec<String> = vec![
            String::from("cd"),
            format!("{}", test_build_dir.display()),
            String::from("&&"),
            String::from("test"),
        ];
        let volumes: Vec<String> = vec![];
        let interactive: bool = false;
        let docker_args: Vec<String> = vec![];
        let image: DockerImage = DockerImage::new("test-registry/test-image:0.1");
        let docker: Docker = Docker::new(image.clone(), interactive);
        let result: Vec<String> = docker.bootstrap_cmd_line(&test_cmd, &docker_top_dir, &work_dir, &docker_args, &volumes);
        let cmd_line: Vec<String> = Helper::docker_bootstrap_string(interactive, &docker_args, &volumes, &docker_top_dir, &work_dir, &image, &test_cmd);
        assert_eq!(result, cmd_line);
    }

    #[test]
    fn test_docker_bootstrap_cmdline_interactive() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let docker_top_dir: PathBuf = work_dir.clone();
        let test_build_dir: PathBuf = work_dir.join(PathBuf::from("test_build_dir"));
        let test_cmd: Vec<String> = vec![
            String::from("cd"),
            format!("{}", test_build_dir.display()),
            String::from("&&"),
            String::from("test"),
        ];
        let volumes: Vec<String> = vec![];
        let interactive: bool = true;
        let docker_args: Vec<String> = vec![];
        let image: DockerImage = DockerImage::new("test-registry/test-image:0.1");
        let docker: Docker = Docker::new(image.clone(), interactive);
        let result: Vec<String> = docker.bootstrap_cmd_line(&test_cmd, &docker_top_dir, &work_dir, &docker_args, &volumes);
        let cmd_line: Vec<String> = Helper::docker_bootstrap_string(interactive, &docker_args, &volumes, &docker_top_dir, &work_dir, &image, &test_cmd);
        assert_eq!(result, cmd_line);
    }

    #[test]
    fn test_docker_bootstrap_args() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let docker_top_dir: PathBuf = work_dir.clone();
        let test_build_dir: PathBuf = work_dir.join(PathBuf::from("test_build_dir"));
        let test_cmd: Vec<String> = vec![
            String::from("cd"),
            format!("{}", test_build_dir.display()),
            String::from("&&"),
            String::from("test"),
        ];
        let volumes: Vec<String> = vec![];
        let interactive: bool = false;
        let docker_args: Vec<String> = vec![
            String::from("--test"),
            String::from("test")
        ];
        let image: DockerImage = DockerImage::new("test-registry/test-image:0.1");
        let docker: Docker = Docker::new(image.clone(), interactive);
        let result: Vec<String> = docker.bootstrap_cmd_line(&test_cmd, &docker_top_dir, &work_dir, &docker_args, &volumes);
        let cmd_line: Vec<String> = Helper::docker_bootstrap_string(interactive, &docker_args, &volumes, &docker_top_dir, &work_dir, &image, &test_cmd);
        assert_eq!(result, cmd_line);
    }

    #[test]
    fn test_docker_bootstrap_volumes() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let docker_top_dir: PathBuf = work_dir.clone();
        let test_build_dir: PathBuf = work_dir.join(PathBuf::from("test_build_dir"));
        let test_cmd: Vec<String> = vec![
            String::from("cd"),
            format!("{}", test_build_dir.display()),
            String::from("&&"),
            String::from("test"),
        ];
        let volumes: Vec<String> = vec![
            String::from("/test/testdir:/test/testdir"),
        ];
        let interactive: bool = false;
        let docker_args: Vec<String> = vec![];
        let image: DockerImage = DockerImage::new("test-registry/test-image:0.1");
        let docker: Docker = Docker::new(image.clone(), interactive);
        let result: Vec<String> = docker.bootstrap_cmd_line(&test_cmd, &docker_top_dir, &work_dir, &docker_args, &volumes);
        let cmd_line: Vec<String> = Helper::docker_bootstrap_string(interactive, &docker_args, &volumes, &docker_top_dir, &work_dir, &image, &test_cmd);
        assert_eq!(result, cmd_line);
    }

    #[test]
    fn test_docker_bootstrap_top_dir() {
        let temp_dir: TempDir =
            TempDir::new("bakery-test-dir").expect("Failed to create temp directory");
        let work_dir: PathBuf = PathBuf::from(temp_dir.path());
        let docker_top_dir: PathBuf = work_dir.clone().join(PathBuf::from("../../"));
        let test_build_dir: PathBuf = work_dir.clone().join(PathBuf::from("test_build_dir"));
        let test_cmd: Vec<String> = vec![
            String::from("cd"),
            format!("{}", test_build_dir.display()),
            String::from("&&"),
            String::from("test"),
        ];
        let volumes: Vec<String> = vec![];
        let interactive: bool = false;
        let docker_args: Vec<String> = vec![];
        let image: DockerImage = DockerImage::new("test-registry/test-image:0.1");
        let docker: Docker = Docker::new(image.clone(), interactive);
        let result: Vec<String> = docker.bootstrap_cmd_line(&test_cmd, &docker_top_dir, &work_dir, &docker_args, &volumes);
        let cmd_line: Vec<String> = Helper::docker_bootstrap_string(interactive, &docker_args, &volumes, &docker_top_dir, &work_dir, &image, &test_cmd);
        assert_eq!(result, cmd_line);
    }
}
