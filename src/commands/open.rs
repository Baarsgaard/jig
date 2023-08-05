use crate::{
    config::Config,
    interactivity,
    jira::{self, types::IssueKey},
    repo::Repository,
    ExecCommand,
};
use clap::Args;
use color_eyre::eyre::{Result, WrapErr};
use std::env;
use std::process::Command;

#[derive(Args, Debug)]
pub struct Open {
    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,
}

impl ExecCommand for Open {
    fn exec(self, cfg: &Config) -> Result<String> {
        let browser = env::var("BROWSER").wrap_err("Failed to open, missing 'BROWSER' env var")?;
        let client = jira::client::JiraAPIClient::new(cfg)?;

        let maybe_repo = Repository::open().wrap_err("Failed to open repo");
        let head = match maybe_repo {
            Ok(repo) => repo.get_branch_name(),
            Err(_) => String::default(),
        };

        let issue_key = if self.issue_key_input.is_some() {
            IssueKey::try_from(self.issue_key_input.unwrap())?
        } else {
            interactivity::issue_from_branch_or_prompt(&client, cfg, head)?.key
        };

        let result = Command::new(browser.clone())
            .args([format!("{}/browse/{}", client.url, issue_key)])
            .spawn();
        match result {
            Ok(_) => Ok(String::default()),
            Err(e) => Err(e).wrap_err(format!("Failed to open {} using {}", issue_key, browser)),
        }
    }
}
