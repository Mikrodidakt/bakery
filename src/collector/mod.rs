pub mod factory;
pub mod file;
pub mod directory;
pub mod manifest;
pub mod archive;
pub mod link;
pub mod conditional;

pub use factory::CollectorFactory;
pub use file::FileCollector;
pub use directory::DirectoryCollector;
pub use manifest::ManifestCollector;
pub use archive::ArchiveCollector;
pub use link::LinkCollector;
pub use conditional::ConditionalCollector;

use crate::error::BError;
use crate::cli::Cli;

use std::path::PathBuf;

#[derive(PartialEq, Debug)]
pub struct Collected {
    pub src: PathBuf,
    pub dest: PathBuf,
}

pub trait Collector {
    fn collect(&self, _src: &PathBuf, _dest: &PathBuf) -> Result<Vec<Collected>, BError> {
        Ok(vec![])
    }

    fn verify_attributes(&self) -> Result<(), BError> {
        Ok(())
    }

    fn info(&self, cli: Option<&Cli>, message: String) {
        match cli {
            Some(c) => {
                c.info(message);
            },
            None => {
                println!("{}", message);
            }
        }
    }
}