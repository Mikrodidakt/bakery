use std::collections::HashMap;
use std::env::Vars;
use std::path::PathBuf;

use indexmap::IndexMap;

use crate::cli::Cli;
use crate::error::BError;
use crate::executers::{Docker, DockerImage};
use crate::workspace::{Workspace, WsBuildData};

pub struct Executer<'a> {
    data: &'a WsBuildData,
    cli: &'a Cli,
}

impl<'a> Executer<'a> {
    pub fn new(build_data: &'a WsBuildData, cli: &'a Cli) -> Self {
        Executer {
            data: build_data,
            cli: cli,
        }
    }

    pub fn execute(
        &self, cmd: &mut Vec<String>, env: &HashMap<String, String>, dir: Option<PathBuf>,
        docker_image: Option<DockerImage>, interactive: bool,) -> Result<(), BError> {
        let mut cmd_line: Vec<String> = vec![];
        let exec_dir: String;

        // If no directory is specified we should use the Workspace work directory
        // as the directory to execute the command from
        match dir {
            Some(directory) => {
                exec_dir = directory.to_string_lossy().to_string();
                cmd_line.append(&mut vec![
                    "cd".to_string(),
                    exec_dir.clone(),
                    "&&".to_string(),
                ]);
                cmd_line.append(cmd);
            }
            None => {
                exec_dir = self.data.settings().work_dir().to_string_lossy().to_string();
                cmd_line.append(&mut vec![
                    "cd".to_string(),
                    exec_dir.clone(),
                    "&&".to_string(),
                ]);
                cmd_line.append(cmd);
            }
        }

        match docker_image {
            Some(image) => {
                let docker: Docker = Docker::new(image, interactive);
                docker.run_cmd(&mut cmd_line, env, exec_dir, &self.cli)?;
            }
            None => {
                self.cli.check_call(&cmd_line, env, true)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use indexmap::IndexMap;

    use crate::cli::*;
    use crate::error::BError;
    use crate::executers::{DockerImage, Executer};
    use crate::workspace::{WsBuildData, WsSettingsHandler};

    fn helper_test_executer(
        verification_str: &String,
        test_cmd: String,
        test_build_dir: Option<PathBuf>,
        image: Option<DockerImage>,
        build_data: &WsBuildData,
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
        let exec: Executer = Executer::new(build_data, &cli);
        exec.execute(&mut vec![test_cmd], &HashMap::new(), test_build_dir, image, true)
    }

    #[test]
    fn test_executer_build_dir() {
        let test_work_dir: String = String::from("/test_work_dir");
        let test_build_dir: PathBuf = PathBuf::from("test_build_dir");
        let test_cmd: String = String::from("test_cmd");
        let verification_str: String = format!("cd {} && {}", test_build_dir.to_string_lossy().to_string(), test_cmd);
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
        let build_data: WsBuildData = WsBuildData::new("", "tmp/deploy/", IndexMap::new(), &settings).expect("Failed to setup build data");    
        let result: Result<(), BError> = helper_test_executer(
            &verification_str,
            test_cmd,
            Some(test_build_dir),
            None,
            &build_data,
        );
        match result {
            Err(err) => {
                assert_eq!("Executer failed", err.to_string());
            }
            Ok(()) => {}
        }
    }

    #[test]
    fn test_executer_no_build_dir() {
        let test_work_dir = String::from("test_work_dir");
        let test_cmd = String::from("test_cmd");
        let verification_str = format!("cd {} && {}", test_work_dir, test_cmd);
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
        let build_data: WsBuildData = WsBuildData::new("", "tmp/deploy/", IndexMap::new(), &settings).expect("Failed to setup build data");    
        
        let result: Result<(), BError> =
            helper_test_executer(&verification_str, test_cmd, None, None, &build_data);
        match result {
            Err(err) => {
                assert_eq!("Executer failed", err.to_string());
            }
            Ok(()) => {}
        }
    }
}
