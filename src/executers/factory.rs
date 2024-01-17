use crate::cli::Cli;
use crate::data::{
    WsTaskData,
    TType, WsBuildData
};
use crate::executers::{
    TaskExecuter,
    BitbakeExecuter,
    NonBitbakeExecuter,
};

use super::DeployExecuter;

pub struct ExecuterFactory {}

impl ExecuterFactory {
    pub fn create<'a>(task_data: &'a WsTaskData, data: &'a WsBuildData, bb_variables: &'a Vec<String>, cli: &'a Cli) -> Box<dyn TaskExecuter + 'a> {
        let executer: Box<dyn TaskExecuter>;
        match task_data.ttype() {
            TType::Bitbake => {
                executer = Box::new(BitbakeExecuter::new(cli, task_data, data.bitbake(), bb_variables));
            },
            TType::NonBitbake => {
                executer = Box::new(NonBitbakeExecuter::new(cli, task_data));
            }
        }
        executer
    }
}