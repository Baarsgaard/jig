use crate::{config::Config, interactivity::issue_from_branch_or_prompt, repo::Repository};
use clap::Args;
use color_eyre::eyre::{Result, WrapErr};
use inquire::Select;
use jira::{
    types::{
        IssueKey, PostTransitionBody, PostTransitionFieldBody, PostTransitionIdBody,
        PostTransitionUpdateField,
    },
    JiraAPIClient,
};
use std::collections::HashMap;

use super::shared::{ExecCommand, UseFilter};

#[derive(Args, Debug)]
pub struct Transition {
    /// Skip querying Jira for Issue summary
    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,

    /// Iterate all fields and prompt for input.
    /// Default: Skip optional
    #[arg(short = 'a', long = "all")]
    iterate_optional_fields: bool,

    #[command(flatten)]
    use_filter: UseFilter,
}

impl ExecCommand for Transition {
    fn exec(self, cfg: &Config) -> Result<String> {
        let client = JiraAPIClient::new(&cfg.jira_cfg)?;

        let maybe_repo = Repository::open().wrap_err("Failed to open repo");
        let head = match maybe_repo {
            Ok(repo) => repo.get_branch_name()?,
            Err(_) => String::default(),
        };

        let issue_key = if self.issue_key_input.is_some() {
            IssueKey::try_from(self.issue_key_input.unwrap())?
        } else {
            issue_from_branch_or_prompt(&client, cfg, head, self.use_filter)?.key
        };

        let transitions = client.get_transitions(&issue_key, true)?;
        let transition = if transitions.len() == 1 && cfg.one_transition_auto_move.unwrap_or(false)
        {
            transitions[0].clone()
        } else {
            Select::new("Move to:", transitions)
                .prompt()
                .wrap_err("No transition selected")?
        };

        // TODO check returned structure and deduct all types of
        // items in a scheme and implement dynamic hashmap of fields and
        // build it with (m)selects and text prompts

        let fields: Option<HashMap<String, PostTransitionFieldBody>> = None;
        let update: Option<PostTransitionUpdateField> = None;

        transition
            .fields
            .iter()
            .filter(|(_, f)| self.iterate_optional_fields || f.required)
            .for_each(|(key, field)| {
                let schema_type = field.schema.schema_type.clone();
                let operations = field.operations.clone();

                match schema_type.as_str() {
                    "User" => todo!(),
                    "Array" => todo!(),
                    _ => todo!(),
                };
            });

        let t = PostTransitionBody {
            transition: PostTransitionIdBody { id: transition.id },
            fields,
            update,
        };

        client.post_transition(&issue_key, &t)?;
        Ok(String::default())
    }
}
