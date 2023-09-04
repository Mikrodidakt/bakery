use std::fmt;

use crate::workspace::Workspace;
use crate::error::BError;

pub struct Docker<'a> {
    workspace: &'a Workspace,
    image: &'a DockerImage,
    interactive: bool, 
}

pub struct DockerImage {
    pub image: String,
    pub tag: String,
    pub registry: String, 
}

impl fmt::Display for DockerImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}:{}", self.registry, self.image, self.tag)
    }
}

impl<'a> Docker<'a> {
    pub fn new(workspace: &'a Workspace, image: &'a DockerImage, interactive: bool) -> Self {
        Docker {
            workspace: workspace,
            image: image,
            interactive: interactive,
        }
    }

    pub fn run_cmd(&self, cmd_line: String, _dir: String) -> Result<(), BError> {
        println!("Execute inside docker image {} '{}'", self.image , cmd_line);
        Ok(())
    }
}

