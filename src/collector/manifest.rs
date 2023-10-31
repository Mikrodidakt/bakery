use crate::collector::Collector;
use crate::cli::Cli;
use crate::error::BError;
use crate::data::AType;

pub struct ManifestCollector {}

impl Collector for ManifestCollector {
    fn collect(&self, cli: &Cli) -> Result<(), BError> {
        Ok(())
    }

    fn constructable(&self, data: &crate::data::WsArtifactData, children: &Vec<crate::workspace::WsArtifactsHandler>) -> bool {
        if data.atype() == &AType::Manifest
            && children.is_empty() {
            return true;
        }
        false
    }

    fn requires(&self, data: &crate::data::WsArtifactData) -> Result<(), BError> {
        if data.manifest().is_empty() || data.name().is_empty() {
            return Err(BError::ValueError(String::from("Manifest node requires a manifest attribute and a name attribute!")));
        } 
        Ok(())
    }
}

impl ManifestCollector {
    pub fn new() -> Self {
        ManifestCollector {}
    }
}