use crate::collector::Collector;
use crate::cli::Cli;
use crate::error::BError;
use crate::data::{
    AType,
    WsArtifactData,
};
use crate::workspace::WsArtifactsHandler;

use std::path::PathBuf;

pub struct FileCollector<'a> {
    artifact: &'a WsArtifactsHandler,
}

impl<'a> Collector for FileCollector<'a> {
    fn collect(&self, src: &PathBuf, dest: &PathBuf) -> Result<Vec<PathBuf>, BError> {
        Ok(vec![])
    }
}

impl<'a> FileCollector<'a> {
    pub fn new(artifact: &'a WsArtifactsHandler) -> Self {
        FileCollector {
            artifact,
        }
    }
}