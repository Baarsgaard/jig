use crate::{
    config::Config,
    interactivity::{issue_from_branch_or_prompt, query_issue_details},
    repo::Repository,
};
use clap::Args;
use color_eyre::eyre::{Result, WrapErr};
use jira::types::IssueKey;
use jira::JiraAPIClient;

use super::shared::{ExecCommand, UseFilter};

#[derive(Args, Debug)]
pub struct Branch {
    /// Skip querying Jira for Issue summary
    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,

    /// Only use ISSUE_KEY as branch name
    /// Inverts 'always_short_branch_names' setting
    #[arg(short = 's', long = "short")]
    toggle_short_name: bool,

    #[command(flatten)]
    use_filter: UseFilter,
}

impl ExecCommand for Branch {
    fn exec(self, cfg: &Config) -> Result<String> {
        let repo = Repository::open().wrap_err("Failed to open repo")?;
        let client = JiraAPIClient::new(&cfg.jira_cfg)?;

        let issue = if let Some(maybe_issue_key) = self.issue_key_input {
            let issue_key = IssueKey::try_from(maybe_issue_key)?;

            query_issue_details(&client, issue_key)?
        } else {
            issue_from_branch_or_prompt(&client, cfg, String::default(), self.use_filter)?
        };

        let mut use_short_name = cfg.always_short_branch_names.unwrap_or(false);
        if self.toggle_short_name {
            use_short_name = !use_short_name;
        }

        if let Ok(branch_name) = repo.issue_branch_exists(&issue) {
            repo.checkout_branch(&branch_name, false)
        } else {
            let branch_name = Repository::branch_name_from_issue(&issue, use_short_name);
            repo.checkout_branch(&branch_name, true)
        }
    }
}
