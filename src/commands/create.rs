use crate::{
    config::Config,
    // interactivity,
    // jira::{self, types::IssueKey},
    // repo::Repository,
    ExecCommand,
};
use clap::Args;
use color_eyre::eyre::Result;

#[derive(Args, Debug)]
pub struct Create {}

impl ExecCommand for Create {
    fn exec(self, _cfg: &Config) -> Result<String> {
        todo!()
    }
}
