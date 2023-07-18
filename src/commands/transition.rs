use crate::{
    config::Config,
    interactivity,
    jira::{self, types::IssueKey},
    repo::Repository,
    ExecCommand,
};
use anyhow::Context;
use clap::Args;
use inquire::Select;

#[derive(Args, Debug)]
pub struct Transition {
    /// Skip querying Jira for Issue summary
    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,
}

impl ExecCommand for Transition {
    fn exec(self, cfg: &Config) -> anyhow::Result<String> {
        let client = jira::client::JiraAPIClient::new(cfg)?;

        let maybe_repo = Repository::open().context("Failed to open repo");
        let head = match maybe_repo {
            Ok(repo) => repo.get_branch_name(),
            Err(_) => String::default(),
        };

        let issue_key = if self.issue_key_input.is_some() {
            IssueKey::try_from(self.issue_key_input.unwrap())?
        } else {
            interactivity::issue_from_branch_or_prompt(&client, cfg, head)?.key
        };

        let transitions = client.get_transitions(&issue_key)?;
        let transition = if transitions.len() == 1 && cfg.one_transition_auto_move.unwrap_or(false)
        {
            transitions[0].clone()
        } else {
            Select::new("Move to:", transitions)
                .prompt()
                .context("No transition selected")?
        };

        client.post_transition(&issue_key, transition)?;
        Ok(String::default())
    }
}
