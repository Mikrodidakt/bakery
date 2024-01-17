pub mod docker;
pub mod recipe;
pub mod factory;
pub mod bitbake;
pub mod nonbitbake;
pub mod deploy;
pub mod upload;

pub use docker::Docker;
pub use docker::DockerImage;
pub use recipe::Recipe;
pub use factory::ExecuterFactory;
pub use bitbake::BitbakeExecuter;
pub use nonbitbake::NonBitbakeExecuter;
pub use deploy::DeployExecuter;
pub use upload::UploadExecuter;

use crate::error::BError;

use std::collections::HashMap;

pub trait TaskExecuter {
    fn exec(&self, env_variables: &HashMap<String, String>, dry_run: bool, interactive: bool) -> Result<(), BError> {
        Ok(())
    }
}