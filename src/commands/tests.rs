#[cfg(test)]
mod tests {
    use crate::commands::*;
    
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
        let cmd: Result<&Box<dyn BCommand>, &'static str> = cmd_handler.get_cmd("build");

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
        let cmd: Result<&Box<dyn BCommand>, &'static str> = cmd_handler.get_cmd("clean");

        match cmd {
            Ok(command) => {
                assert_eq!(command.cmd_str(), "clean");
            }
            Err(err_msg) => {
                assert!(false, "Expected OK result, but got an error '{}'", err_msg);
            }
        } 
    }

    #[test]
    fn test_get_invalid_command() {
        let cmd_handler: CmdHandler = CmdHandler::new();
        let cmd: Result<&Box<dyn BCommand>, &'static str> = cmd_handler.get_cmd("invalid");

        match cmd {
            Ok(command) => {
                assert!(false, "Expected an error, but got an command '{}'", command.cmd_str());
            }
            Err(err_msg) => {
                assert_eq!("Invalid command", err_msg);
            }
        } 
    }
}