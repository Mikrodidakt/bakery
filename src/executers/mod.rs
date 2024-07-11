pub mod docker;
pub mod recipe;
pub mod bitbake;
pub mod nonbitbake;
pub mod customsubcmd;

pub use docker::Docker;
pub use docker::DockerImage;
pub use recipe::Recipe;
pub use bitbake::{BBBuildExecuter, BBCleanExecuter};
pub use nonbitbake::{NonBBBuildExecuter, NonBBCleanExecuter};
pub use customsubcmd::CustomSubCmdExecuter;

use crate::error::BError;

use std::collections::HashMap;

pub trait TaskExecuter {
    fn exec(&self, _env_variables: &HashMap<String, String>, _dry_run: bool, _interactive: bool) -> Result<(), BError> {
        Ok(())
    }
}