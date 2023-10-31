pub mod factory;
pub mod file;
pub mod directory;
pub mod manifest;
pub mod archive;

pub use factory::CollectorFactory;
pub use file::FileCollector;
pub use directory::DirectoryCollector;
pub use manifest::ManifestCollector;
pub use archive::ArchiveCollector;

use crate::cli::Cli;
use crate::data::WsArtifactData;
use crate::error::BError;
use crate::workspace::WsArtifactsHandler;

pub trait Collector {
    fn collect(&self, cli: &Cli) -> Result<(), BError> {
        Ok(())
    }

    fn constructable(&self, data: &WsArtifactData, children: &Vec<WsArtifactsHandler>) -> bool {
        true
    }

    fn requires(&self, data: &WsArtifactData) -> Result<(), BError> {
        Ok(())
    }
}