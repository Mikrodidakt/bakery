#[cfg(test)]
mod tests {
    use crate::commands::*;
    use crate::error::BError;
    
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
}