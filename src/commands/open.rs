use crate::interactivity::issue_from_branch_or_prompt;
use crate::{config::Config, repo::Repository};
use clap::Args;
use color_eyre::eyre::{Result, WrapErr};
use jira::types::IssueKey;
use jira::JiraAPIClient;
use std::env;
use std::process::Command;

use super::shared::{ExecCommand, UseFilter};

#[derive(Args, Debug)]
pub struct Open {
    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,

    #[command(flatten)]
    use_filter: UseFilter,
}

impl Open {
    pub fn open_issue(client: &JiraAPIClient, issue_key: IssueKey) -> Result<String> {
        let url = client.url.join(format!("/browse/{}", issue_key).as_str())?;
        let (browser, args) = match cfg!(target_os = "windows") {
            false => (
                env::var("BROWSER").wrap_err("Failed to open, missing 'BROWSER' env var")?,
                vec![url.to_string()],
            ),
            true => (
                String::from("powershell.exe"),
                vec![String::from("-c"), format!("start('{}')", url.to_string())],
            ),
        };

        match Command::new(browser.clone()).args(args).spawn() {
            Ok(_) => Ok(String::default()),
            Err(e) => Err(e).wrap_err(format!("Failed to open {} using {}", issue_key, browser)),
        }
    }
}

impl ExecCommand for Open {
    async fn exec(self, cfg: &Config) -> Result<String> {
        let client = JiraAPIClient::new(&cfg.jira_cfg)?;

        let maybe_repo = Repository::open().wrap_err("Failed to open repo");
        let head = match maybe_repo {
            Ok(repo) => repo.get_branch_name()?,
            Err(_) => String::default(),
        };

        let issue_key = if self.issue_key_input.is_some() {
            IssueKey::try_from(self.issue_key_input.unwrap())?
        } else {
            issue_from_branch_or_prompt(&client, cfg, head, self.use_filter)
                .await?
                .key
        };

        Self::open_issue(&client, issue_key)
    }
}
