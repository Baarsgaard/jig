use crate::{config::Config, interactivity::issue_from_branch_or_prompt, repo::Repository};
use clap::Args;
use color_eyre::eyre::{eyre, Result, WrapErr};
use inquire::{autocompletion::Replacement, Autocomplete, Text};
use jira::{
    types::{GetAssignableUserParams, IssueKey, User},
    JiraAPIClient,
};
use std::{
    fmt::{Display, Formatter},
    rc::Rc,
};

use super::shared::{ExecCommand, UseFilter};

#[derive(Args, Debug)]
pub struct Assign {
    /// Skip querying Jira for Issue summary
    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,

    #[command(flatten)]
    use_filter: UseFilter,
}

impl ExecCommand for Assign {
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

        let mut completer = AssignableUsersCompleter {
            client: client.clone(),
            input: String::default(),
            users: Rc::new(Vec::default()),
            params: GetAssignableUserParams {
                username: None,
                project: None,
                issue_key: Some(issue_key.clone()),
                max_results: None,
            },
        };

        completer.users = Rc::new(
            client
                .get_assignable_users(&completer.params)?
                .iter()
                .map(|u| JiraUser(u.to_owned()))
                .collect(),
        );

        let username = Text::new("Assign:")
            .with_autocomplete(completer)
            .prompt()
            .wrap_err("No user selected")?;

        let maybe_user = client.get_assignable_users(&GetAssignableUserParams {
            username: Some(username),
            project: None,
            issue_key: None,
            max_results: None,
        })?;

        if let Some(user) = maybe_user.first() {
            client.post_assign_user(&issue_key, &user.clone())?;
            Ok(format!("Assigned {} to {}", user.display_name, issue_key))
        } else {
            Err(eyre!("Invalid username, user not found"))
        }
    }
}

#[derive(Clone)]
struct AssignableUsersCompleter {
    client: JiraAPIClient,
    input: String,
    users: Rc<Vec<JiraUser>>,
    params: GetAssignableUserParams,
}

impl AssignableUsersCompleter {
    pub fn stringify_users(&self) -> Vec<String> {
        self.users
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<String>>()
    }
}

impl Autocomplete for AssignableUsersCompleter {
    fn get_suggestions(
        &mut self,
        input: &str,
    ) -> std::result::Result<Vec<String>, inquire::CustomUserError> {
        if input == self.input {
            return Ok(self.stringify_users());
        }

        let users = self.client.get_assignable_users(&self.params)?;
        self.users = Rc::new(users.iter().map(|u| JiraUser(u.to_owned())).collect());
        Ok(self.stringify_users())
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted_suggestion: Option<String>,
    ) -> std::result::Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        Ok(match highlighted_suggestion {
            Some(suggestion) => Replacement::Some(suggestion),
            None => self.users.first().map(|user| user.0.display_name.clone()),
        })
    }
}

// Necessary to allow also searching server:name and cloud:email
struct JiraUser(User);
#[cfg(not(feature = "cloud"))]
impl Display for JiraUser {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0.name, self.0.display_name)
    }
}

#[cfg(feature = "cloud")]
impl Display for JiraUser {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0.display_name, self.0.email_address)
    }
}
