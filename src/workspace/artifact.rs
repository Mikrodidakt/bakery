use crate::configs::{AType, TaskConfig, ArtifactConfig};
use crate::workspace::WsTaskConfigHandler;

use std::path::{Path, PathBuf};

pub struct WsArtifactConfigHandler<'a> {
    artifact_config: &'a ArtifactConfig,
    task_config: &'a WsTaskConfigHandler<'a>,
}

impl<'a> WsArtifactConfigHandler<'a> {
    pub fn new(artifact_config: &'a ArtifactConfig, task_config: &'a WsTaskConfigHandler) -> Self {
        WsArtifactConfigHandler {
            artifact_config,
            task_config,
        }
    }

    pub fn name(&self) -> &str {
        &self.artifact_config.name
    }

    pub fn ttype(&self) -> &AType {
        &self.artifact_config.atype
    }

    pub fn source(&self) -> PathBuf {
        let mut path_buf: PathBuf = self.task_config.build_dir();
        path_buf.join(&self.artifact_config.source)
    }

    pub fn dest(&self) -> PathBuf {
        let mut path_buf: PathBuf = self.task_config.artifacts_dir();
        path_buf.join(&self.artifact_config.dest)
    }

    pub fn manifest(&self) -> &str {
        &self.artifact_config.manifest
    }

    pub fn artifacts(&self) -> Vec<WsArtifactConfigHandler> {
        let artifacts: Vec<WsArtifactConfigHandler> = self.artifact_config.artifacts.iter().map(|config| {
            WsArtifactConfigHandler::new(config, &self.task_config)
        }).collect();
        artifacts
    }
}