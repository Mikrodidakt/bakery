use crate::workspace::WsBuildConfigHandler;
use crate::configs::{TaskConfig, TType};

use std::path::{PathBuf, Path};

pub struct WsTaskConfigHandler<'a> {
    name: String,
    task_config: &'a TaskConfig,
    ws_config: &'a WsBuildConfigHandler,
}

impl<'a> WsTaskConfigHandler<'a> {
    pub fn new(task_config: &'a TaskConfig, ws_config: &'a WsBuildConfigHandler) -> Self {
        WsTaskConfigHandler {
            name: task_config.name().to_string(),
            task_config,
            ws_config,
        }
    }

    pub fn build_dir(&self) -> PathBuf {
        if self.task_config.ttype() == TType::Bitbake {
            let task_build_dir: &str = self.task_config.builddir();
            if task_build_dir.is_empty() {
                return self.ws_config.bb_build_dir();
            }
        }
        return self.ws_config.context().expand_path(
                    &self.ws_config.work_dir()
                        .join(PathBuf::from(self.task_config.builddir())));
    }

    pub fn ttype(&self) -> TType {
        self.task_config.ttype()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn build_cmd(&self) -> String {
        self.ws_config.context().expand_str(self.task_config.build())
    }

    pub fn clean_cmd(&self) -> String {
        self.ws_config.context().expand_str(self.task_config.clean())
    }

    pub fn docker(&self) -> String {
        self.ws_config.context().expand_str(self.task_config.docker())
    }

    pub fn disabled(&self, build: &str) -> bool {
        if self.task_config.disabled() == "true" {
            return true;
        }
        return false;
    }

    /*
    pub fn recipes(&self) -> Vec<String> {
        self.ws_config.context().expand_vec(self.task_config.recipes())
    }
    */

    pub fn condition(&self) -> bool {
        let condition: &str = self.task_config.condition();
        
        if condition.is_empty() {
            return true;
        }

        match self.ws_config.context().expand_str(condition).as_str() {
            "1"|"yes"|"y"|"true"|"YES"|"TRUE"|"True"|"Yes" => return true,
            _ => return false,
        }
    }
}


    /*
    pub fn artifacts(&self, build: &str) -> IndexMap<String, String> {}
    */