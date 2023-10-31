use crate::collector::Collector;
use crate::cli::Cli;
use crate::error::BError;
use crate::data::{
    AType,
    WsArtifactData,
};
use crate::workspace::WsArtifactsHandler;

use std::path::PathBuf;

pub struct DirectoryCollector<'a> {
    artifact: &'a WsArtifactsHandler,
}

impl<'a> Collector for DirectoryCollector<'a> {
    fn collect(&self, src: &PathBuf, dest: &PathBuf) -> Result<Vec<PathBuf>, BError> {
        Ok(vec![])
    }

    fn verify_attributes(&self) -> Result<(), BError> {
        if self.artifact.data().name().is_empty()
            || self.artifact.children().is_empty() {
                return Err(BError::ValueError(String::from("Directory node requires name and list of artifacts!")));
        }
        Ok(())
    }
}

impl<'a> DirectoryCollector<'a> {
    pub fn new(artifact: &'a WsArtifactsHandler) -> Self {
        DirectoryCollector {
            artifact,
        }
    }
}