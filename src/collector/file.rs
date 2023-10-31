use crate::collector::Collector;
use crate::cli::Cli;
use crate::data::AType;
use crate::error::BError;

pub struct FileCollector {}

impl Collector for FileCollector {
    fn collect(&self, cli: &Cli) -> Result<(), BError> {
        Ok(())
    }

    fn constructable(&self, data: &crate::data::WsArtifactData, children: &Vec<crate::workspace::WsArtifactsHandler>) -> bool {
        if data.atype() == &AType::File
            && children.is_empty() {
            return true;
        }
        false
    }

    fn requires(&self, data: &crate::data::WsArtifactData) -> Result<(), BError> {
        if data.source().to_string_lossy().is_empty() || data.source().to_string_lossy().is_empty() {
            return Err(BError::ValueError(String::from("File node requires a source attribute and dest attribute!")));
        } 
        Ok(())
    }
}

impl FileCollector {
    pub fn new() -> Self {
        FileCollector {}
    }
}