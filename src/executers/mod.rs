pub mod bitbake;
pub mod customsubcmd;
pub mod docker;
pub mod nonbitbake;
pub mod recipe;

pub use bitbake::{BBBuildExecuter, BBCleanExecuter};
pub use customsubcmd::CustomSubCmdExecuter;
pub use docker::Docker;
pub use docker::DockerImage;
pub use nonbitbake::{NonBBBuildExecuter, NonBBCleanExecuter};
pub use recipe::Recipe;

use crate::error::BError;

use std::collections::HashMap;

pub trait TaskExecuter {
    fn exec(
        &self,
        _env_variables: &HashMap<String, String>,
        _dry_run: bool,
        _interactive: bool,
    ) -> Result<(), BError> {
        Ok(())
    }
}
