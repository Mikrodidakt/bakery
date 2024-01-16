use std::collections::HashMap;
use std::path::PathBuf;
use clap::ArgMatches;

use crate::commands::{CmdHandler, BCommand};
use crate::error::BError;
use crate::cli::{Logger, System, CallParams};

pub struct Cli {
    cmd_line: Vec<String>,
    args: ArgMatches,
    cmd_handler: CmdHandler,
    logger: Box<dyn Logger>,
    system: Box<dyn System>,
}

impl Cli {
    pub fn new(logger: Box<dyn Logger>, system: Box<dyn System>, cmd: clap::Command, cmd_line: Option<Vec<&str>>) -> Self {
        let cmd_handler: CmdHandler = CmdHandler::new();
        let args: ArgMatches;
        let c: Vec<String>;
        match cmd_line {
            Some(cline) => {
                args = cmd_handler.build_cli(cmd).get_matches_from(cline.clone());
                c = cline.iter().map(|s|{
                    s.to_string()
                }).collect();
            },
            None => {
                args = cmd_handler.build_cli(cmd).get_matches();
                c = std::env::args().into_iter().map(|s|{
                    s
                }).collect();
            }
        }
        Cli {
            cmd_line: c,
            args,
            cmd_handler,
            logger,
            system,
        }
    }

    pub fn check_call(&self, cmd_line: &Vec<String>, env: &HashMap<String, String>, shell: bool) -> Result<(), BError> {
        let mut cmd: String = String::new();
        cmd_line.iter().for_each(|c|{
            cmd.push_str(c);
            cmd.push(' ');
        });
        //println!("log {}", String::from(cmd.as_str().trim_end()));
        self.info(String::from(cmd.as_str().trim_end()));
        self.system.check_call(&CallParams{ cmd_line: cmd_line.to_owned(), env: env.to_owned(), shell })?;
        //self.system.test(String::from(cmd.as_str().trim_end()))?;
        Ok(())
    }

    pub fn source_init_env(&self, init_file: &PathBuf, build_dir: &PathBuf) -> Result<HashMap<String, String>, BError> {
        self.system.init_env_file(init_file, build_dir)
    }

    pub fn get_command(&self, name: &str) -> Result<&Box<dyn BCommand>, BError> {
        self.cmd_handler.get_cmd(&name)
    }

    pub fn get_args(&self) -> &ArgMatches {
        &self.args
    }

    pub fn get_cmd_line(&self) -> Vec<String> {
        self.cmd_line.clone()
    }

    pub fn get_home_dir(&self) -> PathBuf {
        match std::env::var_os("HOME") {
            Some(var) => {
                return PathBuf::from(var.into_string().or::<String>(Ok(String::from(""))).unwrap());
            },
            None => {
                return PathBuf::from("");
            }
        }
    }

    pub fn get_curr_dir(&self) -> PathBuf {
        match std::env::current_dir() {
            Ok(path) => {
                return path;
            },
            Err(_e) => {
                return PathBuf::from("");
            }
        }
    }

    //pub fn build_cli(&self, cmd: &clap::Command) -> clap::ArgMatches {
    //    self.cmd_handler.build_cli(cmd.clone()).get_matches()
    //}

    pub fn _get_logger(&self) -> &Box<dyn Logger> {
        &self.logger
    }

    pub fn info(&self, message: String) {
        (*self.logger).info(message);
    }

    pub fn _warn(&self, message: String) {
        (*self.logger).warn(message);
    }

    pub fn error(&self, message: String) {
        (*self.logger).error(message);
    }

    pub fn stdout(&self, message: String) {
        (*self.logger).stdout(message);
    }
}