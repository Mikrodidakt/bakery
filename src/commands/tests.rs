#[cfg(test)]
mod tests {
    use crate::commands::*;
    use crate::error::BError;
    use crate::cli::*;
    use crate::workspace::Workspace;
    use crate::docker::{Docker, DockerImage};
    
    fn helper_test_executer(verification_str: &String, test_cmd: &String, test_build_dir: Option<String>, docker: Option<Docker>, workspace: &Workspace) -> Result<(), BError> {
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger.expect_info().with(mockall::predicate::eq(verification_str.clone())).once().returning(|_x|());
        let cli: Cli = Cli::new(Box::new(mocked_logger));
        let exec: Executer = Executer::new(workspace, &cli);
        exec.execute(&test_cmd, std::env::vars(), test_build_dir, docker, true) 
    }

    #[test]
    fn test_build_command() {
        let cmd: BuildCommand = BuildCommand::new();
        assert_eq!(cmd.cmd_str(), "build");
    }

    #[test]
    fn test_clean_command() {
        let cmd: CleanCommand = CleanCommand::new();
        assert_eq!(cmd.cmd_str(), "clean");
    }

    #[test]
    fn test_get_build_command() {
        let cmd_handler: CmdHandler = CmdHandler::new();
        let cmd: Result<&Box<dyn BCommand>, BError> = cmd_handler.get_cmd("build");

        match cmd {
            Ok(command) => {
                assert_eq!(command.cmd_str(), "build");
            }
            Err(err_msg) => {
                assert!(false, "Expected OK result, but got an error '{}'", err_msg);
            }
        } 
    }

    #[test]
    fn test_get_clean_command() {
        let cmd_handler: CmdHandler = CmdHandler::new();
        let cmd: Result<&Box<dyn BCommand>, BError> = cmd_handler.get_cmd("clean");

        match cmd {
            Ok(command) => {
                assert_eq!(command.cmd_str(), "clean");
            }
            Err(err) => {
                assert!(false, "Expected OK result, but got an error '{}'", err);
            }
        } 
    }

    #[test]
    fn test_get_invalid_command() {
        let cmd_handler: CmdHandler = CmdHandler::new();
        let cmd: Result<&Box<dyn BCommand>, BError> = cmd_handler.get_cmd("invalid");

        match cmd {
            Ok(command) => {
                assert!(false, "Expected an error, but got an command '{}'", command.cmd_str());
            }
            Err(err) => {
                // TODO: we should make sure that BError is using PartialEq and Eq Traits
                assert_eq!("Invalid command", err.message);
            }
        } 
    }

    #[test]
    fn test_executer_build_dir() {
        let test_work_dir = String::from("test_work_dir");
        let test_build_dir = String::from("test_build_dir");
        let test_cmd = String::from("test_cmd");
        let verification_str = format!("Execute 'cd {} && {}'", test_build_dir, test_cmd);
        let workspace: Workspace = Workspace{ _work_dir: test_work_dir };
        let result: Result<(), BError> = helper_test_executer(&verification_str, &test_cmd, Some(test_build_dir), None, &workspace);
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
        let workspace: Workspace = Workspace{ _work_dir: test_work_dir };
        let result: Result<(), BError> = helper_test_executer(&verification_str, &test_cmd, None, None, &workspace);
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
        let workspace: Workspace = Workspace{ _work_dir: test_work_dir };
        let docker: Docker = Docker::new(&workspace, &docker_image, true);
        let result: Result<(), BError> = helper_test_executer(&verification_str, &test_cmd, None, Some(docker), &workspace);
        match result {
            Err(err) => {
                assert_eq!("Executer failed", err.message);
            }
            Ok(()) => {}
        }
    }
}