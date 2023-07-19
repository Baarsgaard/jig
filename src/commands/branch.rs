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
    #[arg(short, long)]
    short_name: bool,
}

impl ExecCommand for Branch {
    fn exec(self, cfg: &Config) -> anyhow::Result<String> {
        let repo = Repository::open().context("Failed to open repo")?;

        let branch_name = if let Some(maybe_issue_key) = self.issue_key_input {
            let issue_key = IssueKey::try_from(maybe_issue_key)?;

            if !self.short_name {
                let client = jira::client::JiraAPIClient::new(cfg)?;
                interactivity::query_issue_details(&client, issue_key)?.to_string()
            } else {
                issue_key.to_string()
            }
        } else {
            let client = jira::client::JiraAPIClient::new(cfg)?;
            let issues = interactivity::query_issues_with_retry(&client, cfg)?;
            let issue = interactivity::prompt_user_with_issue_select(issues)?;

            if self.short_name {
                issue.key.to_string()
            } else {
                let branch_name = issue.to_string().replace(' ', "_");
                match branch_name.len() {
                    n if n > 50 => branch_name.split_at(51).0.to_owned(),
                    _ => branch_name,
                }
            }
        };

        repo.create_branch(branch_name.clone())
    }
}
