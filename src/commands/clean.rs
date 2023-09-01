use crate::commands::BCommand;

static BCOMMAND: &str = "clean";
static BCOMMAND_ABOUT: &str = "Clean one of the components";
pub struct CleanCommand {
    _cmd_str: String,
    _subcmd: clap::Command,
    // Your struct fields and methods here
}

impl BCommand for CleanCommand {
    fn cmd_str(&self) -> &str {
        &self._cmd_str
    }

    fn subcommand(&self) -> &clap::Command {
        &self._subcmd
    }
}

impl CleanCommand {
    pub fn new() -> Self {
        let subcmd: clap::Command = clap::Command::new(BCOMMAND)
            .short_flag('c')
            .long_flag(BCOMMAND)
            .about(BCOMMAND_ABOUT);
        /*
            .arg_required_else_help(true);
            .arg(
                clap::Arg::new("config")
                    .short('c')
                    .long("config")
                    .help("The build config defining all the components for the full build")
                    .value_name("path")
                    .required(true),
            );
        */
        // Initialize and return a new BuildCommand instance
        CleanCommand {
            // Initialize fields if any
            _cmd_str: String::from(BCOMMAND),
            _subcmd: subcmd,
        }
    }
}
