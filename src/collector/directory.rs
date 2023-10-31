use crate::collector::Collector;
use crate::cli::Cli;
use crate::error::BError;
use crate::data::AType;

pub struct DirectoryCollector {}

impl Collector for DirectoryCollector {
    fn collect(&self, cli: &Cli) -> Result<(), BError> {
        Ok(())
    }

    fn constructable(&self, data: &crate::data::WsArtifactData, children: &Vec<crate::workspace::WsArtifactsHandler>) -> bool {
        if data.atype() == &AType::Directory
            && !children.is_empty() { // <== maybe this check should be moved to the requires method
            return true;
        }
        false
    }

    fn requires(&self, data: &crate::data::WsArtifactData) -> Result<(), BError> {
        if data.name().is_empty() {
            return Err(BError::ValueError(String::from("Directory node requires a name attribute and a list of artifacts!")));
        }
        Ok(())
    }
}

impl DirectoryCollector {
    pub fn new() -> Self {
        DirectoryCollector {}
    }
}