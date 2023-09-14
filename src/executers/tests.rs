#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::executers::{Docker, DockerImage, Executer};
    use crate::workspace::Workspace;
    use crate::configs::WorkspaceSettings;
    use crate::cli::*;
    use crate::error::BError;

    fn helper_settings_from_str(json_test_str: &str) -> WorkspaceSettings {
        let result: Result<WorkspaceSettings, BError> = WorkspaceSettings::from_str(json_test_str);
        let settings: WorkspaceSettings;
        match result {
            Ok(rsettings) => {
                settings = rsettings;
            }
            Err(e) => {
                eprintln!("Error parsing JSON: {}", e);
                panic!();
            } 
        }
        settings
    }

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
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let config: WorkspaceSettings = helper_settings_from_str(json_test_str);
        let work_dir: PathBuf = PathBuf::from(test_work_dir);
        let workspace: Workspace = Workspace::new(Some(work_dir), config);
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
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let config: WorkspaceSettings = helper_settings_from_str(json_test_str);
        let work_dir: PathBuf = PathBuf::from(test_work_dir);
        let workspace: Workspace = Workspace::new(Some(work_dir), config);
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

    #[test]
    fn test_executer_docker() {
        let test_work_dir = String::from("test_work_dir");
        let test_cmd = String::from("test_cmd");
        let docker_image: DockerImage = DockerImage {
            registry: String::from("test-registry"),
            image: String::from("test-image"),
            tag: String::from("0.1"),
        };
        let verification_str = format!("Execute inside docker image {} 'cd {} && {}'", docker_image, test_work_dir, test_cmd);
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let config: WorkspaceSettings = helper_settings_from_str(json_test_str);
        let work_dir: PathBuf = PathBuf::from(test_work_dir);
        let workspace: Workspace = Workspace::new(Some(work_dir), config);
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
        let json_test_str = r#"
        {
            "version": "4"
        }"#;
        let config: WorkspaceSettings = helper_settings_from_str(json_test_str);
        let work_dir: PathBuf = PathBuf::from(test_work_dir.clone());
        let workspace: Workspace = Workspace::new(Some(work_dir), config);
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