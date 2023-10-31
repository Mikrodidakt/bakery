use crate::collector::Collector;
use crate::cli::Cli;
use crate::error::BError;
use crate::data::{
    AType,
    WsArtifactData,
};
use crate::workspace::WsArtifactsHandler;

use std::path::PathBuf;

pub struct ManifestCollector<'a> {
    artifact: &'a WsArtifactsHandler,
}

impl<'a> Collector for ManifestCollector<'a> {
    fn collect(&self, src: &PathBuf, dest: &PathBuf) -> Result<Vec<PathBuf>, BError> {
        Ok(vec![])
    }

    fn verify_attributes(&self) -> Result<(), BError> {
        if self.artifact.data().name().is_empty()
            || self.artifact.data().manifest().is_empty() {
                return Err(BError::ValueError(String::from("Manifest node requires name and manifest content!")));
        }
        Ok(())
    }
}

impl<'a> ManifestCollector<'a> {
    pub fn new(artifact: &'a WsArtifactsHandler) -> Self {
        ManifestCollector {
            artifact,
        }
    }
}