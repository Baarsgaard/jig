use std::fmt::{Display, Formatter};

use crate::{
    config::Config, interactivity::issue_from_branch_or_prompt, repo::Repository, ExecCommand,
};
use clap::Args;
use color_eyre::eyre::{Result, WrapErr};
use inquire::Select;
use jira::{
    types::{IssueKey, User},
    JiraAPIClient,
};

#[derive(Args, Debug)]
pub struct Assign {
    /// Skip querying Jira for Issue summary
    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,

    /// Prompt for filter to use a default_query
    #[arg(short = 'f', long = "filter")]
    #[cfg(feature = "cloud")]
    use_filter: bool,
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
            #[cfg(feature = "cloud")]
            let issue_key = issue_from_branch_or_prompt(&client, cfg, head, self.use_filter)?.key;
            #[cfg(not(feature = "cloud"))]
            let issue_key = issue_from_branch_or_prompt(&client, cfg, head)?.key;
            issue_key
        };

        let users = client
            .get_assignable_users(&issue_key)?
            .iter()
            .map(|u| JiraUser(u.to_owned()))
            .collect();
        let user = Select::new("Assign:", users)
            .prompt()
            .wrap_err("No user selected")?;

        client.post_assign_user(&issue_key, &user.0)?;
        Ok(format!("Assigned {} to {}", user, issue_key))
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
