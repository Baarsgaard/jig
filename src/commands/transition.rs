use crate::{config::Config, interactivity::issue_from_branch_or_prompt, repo::Repository};
use clap::Args;
use color_eyre::{
    Section,
    eyre::{Result, WrapErr, eyre},
};
use inquire::Select;
use jira::{
    JiraAPIClient,
    models::{IssueKey, PostTransitionBody, PostTransitionIdBody},
};

use super::shared::ExecCommand;

#[derive(Args, Debug)]
pub struct Transition {
    /// Skip querying Jira for Issue summary
    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,
}

impl ExecCommand for Transition {
    async fn exec(self, cfg: &Config) -> Result<String> {
        let client = JiraAPIClient::new(&cfg.jira_cfg)?;

        let maybe_repo = Repository::open().wrap_err("Failed to open repository");
        let head = match maybe_repo {
            Ok(repo) => repo.get_branch_name()?,
            Err(_) => String::default(),
        };

        let issue_key = if self.issue_key_input.is_some() {
            IssueKey::try_from(self.issue_key_input.unwrap())?
        } else {
            issue_from_branch_or_prompt(&client, cfg, head).await?.key
        };

        let transitions_response = client.get_transitions(&issue_key, None).await?;
        if transitions_response.transitions.is_empty() {
            return Err(eyre!("No valid transitions"));
        }

        let transitions = transitions_response.transitions;
        let selected_transition =
            if transitions.len() == 1 && cfg.one_transition_auto_move.unwrap_or(false) {
                transitions[0].clone()
            } else {
                Select::new("Move to:", transitions)
                    .prompt()
                    .wrap_err("No transition selected")?
            };

        // TODO implement terminal UI for handling this
        // Abort if there's required fields
        if selected_transition
            .fields
            .into_iter()
            .any(|(_, t)| t.required && !t.has_default_value.is_some_and(|v| v))
        {
            return Err(
                eyre!("Issue cannot be moved with Jig due to required fields.",).with_suggestion(
                    || format!("Open issue in your browser with: jig open {issue_key}"),
                ),
            );
        }

        let transition = PostTransitionBody {
            transition: PostTransitionIdBody {
                id: selected_transition.id,
            },
            fields: None,
            update: None,
        };

        client.post_transition(&issue_key, &transition).await?;
        Ok(String::default())
    }
}
