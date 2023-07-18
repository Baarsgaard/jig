use crate::{
    config::Config,
    // interactivity,
    // jira::{self, types::IssueKey},
    // repo::Repository,
    ExecCommand,
};
// use anyhow::Context;
use clap::Args;

#[derive(Args, Debug)]
pub struct Create {}

impl ExecCommand for Create {
    fn exec(self, _cfg: &Config) -> anyhow::Result<String> {
        todo!()
    }
}
