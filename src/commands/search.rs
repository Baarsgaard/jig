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
pub struct Search {
    /// Prompt for filter to use a default_query
    #[arg(short = 'f', long = "filter")]
    use_filter: bool,
}

impl ExecCommand for Search {
    fn exec(self, _cfg: &Config) -> Result<String> {
        todo!()
    }
}
