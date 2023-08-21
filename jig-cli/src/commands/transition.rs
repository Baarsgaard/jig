use crate::{config::Config, interactivity::issue_from_branch_or_prompt, repo::Repository};
use clap::Args;
use color_eyre::eyre::{Result, WrapErr};
use inquire::Select;
use jira::{types::IssueKey, JiraAPIClient};

use super::shared::{ExecCommand, UseFilter};

#[derive(Args, Debug)]
pub struct Transition {
    /// Skip querying Jira for Issue summary
    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,

    #[command(flatten)]
    use_filter: UseFilter,
}

impl ExecCommand for Transition {
    fn exec(self, cfg: &Config) -> Result<String> {
        let client = JiraAPIClient::new(&cfg.jira_cfg)?;

        let maybe_repo = Repository::open().wrap_err("Failed to open repo");
        let head = match maybe_repo {
            Ok(repo) => repo.get_branch_name(),
            Err(_) => String::default(),
        };

        let issue_key = if self.issue_key_input.is_some() {
            IssueKey::try_from(self.issue_key_input.unwrap())?
        } else {
            issue_from_branch_or_prompt(&client, cfg, head, self.use_filter)?.key
        };

        let transitions = client.get_transitions(&issue_key)?;
        let transition = if transitions.len() == 1 && cfg.one_transition_auto_move.unwrap_or(false)
        {
            transitions[0].clone()
        } else {
            Select::new("Move to:", transitions)
                .prompt()
                .wrap_err("No transition selected")?
        };

        client.post_transition(&issue_key, transition)?;
        Ok(String::default())
    }
}
