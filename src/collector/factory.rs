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
    fn collectors() -> HashMap<AType, Box<dyn Collector>> {
        let mut supported_collectors: HashMap<AType, Box<dyn Collector>> = HashMap::new();
    
        // Add supported collectors to the HashMap
        supported_collectors.insert(AType::File, Box::new(FileCollector::new()));
        supported_collectors.insert(AType::Directory, Box::new(DirectoryCollector::new()));
        supported_collectors.insert(AType::Manifest, Box::new(ManifestCollector::new()));
        supported_collectors.insert(AType::Archive, Box::new(ArchiveCollector::new()));
    
        // Add more commands as needed
    
        supported_collectors
    }

    pub fn create(data: &WsArtifactData, children: &Vec<WsArtifactsHandler>) -> Result<Box<dyn Collector>, BError> {
        for (atype, collector) in Self::collectors() {
            if collector.constructable(data, children) {
                collector.requires(data)?;
                return Ok(collector);
            }
        }

        Err(BError::CollectorError(String::from("Failed to collect artifacts")))
    }
}