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

use crate::error::BError;

use std::path::PathBuf;

pub trait Collector {
    fn collect(&self, src: &PathBuf, dest: &PathBuf) -> Result<Vec<PathBuf>, BError> {
        Ok(vec![])
    }

    fn verify_attributes(&self) -> Result<(), BError> {
        Ok(())
    }
}