use std::path::{PathBuf, Path};
use std::env;
use std::io::Error;

use crate::workspace::Settings;
pub struct Workspace<'a> {
    pub work_dir: PathBuf,
    settings: &'a Settings<'a>,
}

impl<'a> Workspace<'a> {
    pub fn new(workdir: Option<PathBuf>, settings: &'a Settings) -> Self {
        let mut work_dir: PathBuf = PathBuf::new();
        match workdir {
            Some(w_dir) => {
                work_dir.push(w_dir);
            },
            None => {
                let path: Result<PathBuf, Error> = env::current_dir();
                match path {
                    Ok(w_dir) => {
                        work_dir.push(w_dir);
                    },
                    Err(_e) => {
                        panic!("{}", _e.to_string());
                    }
                }
            } 
        }
        Workspace {
            work_dir,
            settings,
        }
    }

    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }
}