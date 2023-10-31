use crate::collector::{
    Collector,
    FileCollector,
    DirectoryCollector,
    ManifestCollector,
    ArchiveCollector,
};
use crate::data::{WsArtifactData, AType};
use crate::workspace::WsArtifactsHandler;
use crate::error::BError;

use std::collections::HashMap;

pub struct CollectorFactory {}

impl CollectorFactory {
    pub fn create<'a>(artifact: &'a WsArtifactsHandler) -> Result<Box<dyn Collector + 'a>, BError> {
        let collector: Box<dyn Collector>;
        match artifact.data().atype() {
            AType::Archive => {
                collector = Box::new(ArchiveCollector::new(artifact));
            },
            AType::Directory => {
                collector = Box::new(DirectoryCollector::new(artifact));
            },
            AType::File => {
                collector = Box::new(FileCollector::new(artifact));
            },
            AType::Manifest => {
                collector = Box::new(ManifestCollector::new(artifact));
            }
        }
        collector.verify_attributes()?;
        Ok(collector)
    }
}