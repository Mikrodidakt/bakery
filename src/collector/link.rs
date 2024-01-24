use crate::collector::{
    Collector,
    Collected,
};
use crate::cli::Cli;
use crate::error::BError;
use crate::workspace::WsArtifactsHandler;

use std::path::PathBuf;
use std::os::unix::fs;

pub struct LinkCollector<'a> {
    artifact: &'a WsArtifactsHandler,
    cli: Option<&'a Cli>,
}

impl<'a> Collector for LinkCollector<'a> {
    fn collect(&self, src: &PathBuf, dest: &PathBuf) -> Result<Vec<Collected>, BError> {
        let mut collected: Vec<Collected> = vec![];
        let dest_str: &str = self.artifact.data().dest();
        let src_path: PathBuf = src.join(PathBuf::from(self.artifact.data().source()));
        let dest_path: PathBuf = dest.join(PathBuf::from(dest_str));

        fs::symlink(src_path, dest_path)?;

        Ok(collected)
    }

    fn verify_attributes(&self) -> Result<(), BError> {
        if self.artifact.data().source().is_empty() {
            return Err(BError::ValueError(String::from("Link node requires source attribute!")));
        }

        if self.artifact.data().dest().is_empty() {
            return Err(BError::ValueError(String::from("Link node requires dest attribute!")));
        }

        Ok(())
    }
}

impl<'a> LinkCollector<'a> {
    pub fn new(artifact: &'a WsArtifactsHandler, cli: Option<&'a Cli>) -> Self {
        LinkCollector {
            artifact,
            cli,
        }
    }
}