use crate::collector::{
    Collector,
    FileCollector,
    DirectoryCollector,
    ManifestCollector,
    ArchiveCollector,
};
use crate::data::AType;
use crate::workspace::WsArtifactsHandler;
use crate::error::BError;
use crate::cli::Cli;

pub struct CollectorFactory {}

impl CollectorFactory {
    pub fn create<'a>(artifact: &'a WsArtifactsHandler, cli: Option<&'a Cli>) -> Result<Box<dyn Collector + 'a>, BError> {
        let collector: Box<dyn Collector>;
        match artifact.data().atype() {
            AType::Archive => {
                collector = Box::new(ArchiveCollector::new(artifact, cli));
            },
            AType::Directory => {
                collector = Box::new(DirectoryCollector::new(artifact, cli));
            },
            AType::File => {
                collector = Box::new(FileCollector::new(artifact, cli));
            },
            AType::Manifest => {
                collector = Box::new(ManifestCollector::new(artifact, cli));
            }
        }
        collector.verify_attributes()?;
        Ok(collector)
    }
}