use std::fmt::{Display, Formatter};

use crate::{config::Config, interactivity::issue_from_branch_or_prompt, repo::Repository};
use clap::{Args, ValueHint};
use color_eyre::eyre::{Result, WrapErr, eyre};
use inquire::{Select, Text};
use jira::{
    JiraAPIClient,
    models::{GetAssignableUserParams, IssueKey, User},
};

use super::shared::{ExecCommand, UseFilter};

#[derive(Args, Debug)]
pub struct Assign {
    /// Skip querying Jira for Issue summary
    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,

    /// Skip user selection prompt
    #[arg(short, long, value_name = if cfg!(feature = "cloud") {"ACCOUNT_ID"} else {"NAME"}, value_hint = ValueHint::Unknown)]
    user: Option<String>,

    #[command(flatten)]
    use_filter: UseFilter,
}

impl ExecCommand for Assign {
    async fn exec(self, cfg: &Config) -> Result<String> {
        let client = JiraAPIClient::new(&cfg.jira_cfg)?;

        let maybe_repo = Repository::open().wrap_err("Failed to open repository");
        let head = match maybe_repo {
            Ok(repo) => Some(repo.get_branch_name()?),
            Err(_) => None,
        };

        let issue_key = if self.issue_key_input.is_some() {
            IssueKey::try_from(self.issue_key_input.unwrap())?
        } else {
            issue_from_branch_or_prompt(
                &client,
                cfg,
                head.unwrap_or(String::default()),
                self.use_filter,
            )
            .await?
            .key
        };

        // Disable query and prompt if --user is supplied
        let user = if let Some(user) = self.user {
            client.get_user(&user).await?
        } else {
            let username = Text::new("User search:")
                .prompt()
                .wrap_err("No user selected")?;

            let users = client
                .get_assignable_users(&GetAssignableUserParams {
                    username: Some(username),
                    project: None,
                    issue_key: Some(issue_key.clone()),
                    max_results: None,
                })
                .await?;

            if users.is_empty() {
                Err(eyre!("No users found in search"))
            } else if users.len() == 1 {
                match users.first() {
                    Some(user) => Ok(user.to_owned()),
                    None => Err(eyre!("List of one user had unwrapped to None?")),
                }
            } else {
                Ok(Select::new(
                    "Pick user",
                    users.iter().map(|u| JiraUser(u.to_owned())).collect(),
                )
                .prompt()
                .wrap_err("Select prompt interrupted")?
                .0)
            }?
        };

        client.post_assign_user(&issue_key, &user).await?;

        Ok(format!("Assigned {} to {}", issue_key, JiraUser(user)))
    }
}

// Necessary to allow also searching server:name and cloud:email
struct JiraUser(User);

impl Display for JiraUser {
    #[cfg(feature = "cloud")]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.0.display_name, self.0.account_id)
    }
    #[cfg(not(feature = "cloud"))]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.0.display_name, self.0.name)
    }
}
