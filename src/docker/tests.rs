#[cfg(test)]
mod tests {
    use crate::docker::{Docker, DockerImage};
    use crate::workspace::Workspace;
    use crate::cli::*;
    use crate::error::BError;

    fn helper_test_docker(verification_str: &String, test_cmd: &String, test_work_dir: Option<String>, image: &DockerImage, workspace: &Workspace) -> Result<(), BError> {
        let mut mocked_logger: MockLogger = MockLogger::new();
        mocked_logger.expect_info().with(mockall::predicate::eq(verification_str.clone())).once().returning(|_x|());
        let cli: Cli = Cli::new(Box::new(mocked_logger));
        let docker: Docker = Docker::new(&workspace, image, true);
        docker.run_cmd(test_cmd.clone(), test_work_dir.unwrap(), &cli)
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
        let workspace: Workspace = Workspace{ _work_dir: test_work_dir.clone() };
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