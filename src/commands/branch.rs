use crate::{
    config::Config,
    interactivity,
    jira::{self, types::IssueKey},
    repo::Repository,
    ExecCommand,
};
use anyhow::Context;
use clap::Args;

#[derive(Args, Debug)]
pub struct Branch {
    /// Skip querying Jira for Issue summary
    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,

    /// Only use ISSUE_KEY as branch name
    /// Inverts 'always_short_branch_names' setting
    #[arg(short = 's', long = "short")]
    toggle_short_name: bool,
}

impl ExecCommand for Branch {
    fn exec(self, cfg: &Config) -> anyhow::Result<String> {
        let repo = Repository::open().context("Failed to open repo")?;

        let issue = if let Some(maybe_issue_key) = self.issue_key_input {
            let issue_key = IssueKey::try_from(maybe_issue_key)?;
            let client = jira::client::JiraAPIClient::new(cfg)?;

            interactivity::query_issue_details(&client, issue_key)?
        } else {
            let client = jira::client::JiraAPIClient::new(cfg)?;
            let issues = interactivity::query_issues_with_retry(&client, cfg)?;
            interactivity::prompt_user_with_issue_select(issues)?
        };

        let mut use_short_name = cfg.always_short_branch_names.unwrap_or(false);
        if self.toggle_short_name {
            use_short_name = !use_short_name;
        }

        if let Ok(branch_name) = repo.issue_branch_exists(&issue) {
            repo.checkout_branch(&branch_name, false)
        } else {
            let branch_name = repo.branch_name_from_issue(&issue, use_short_name);
            repo.checkout_branch(&branch_name, true)
        }
    }
}
