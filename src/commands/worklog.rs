use crate::{
    config::Config,
    interactivity,
    jira::{
        self,
        types::{IssueKey, PostWorklogBody, WorklogDuration},
    },
    repo::Repository,
    ExecCommand,
};
use anyhow::{anyhow, Context};
use clap::Args;

#[derive(Args, Debug)]
pub struct Worklog {
    /// Inverts 'always_confirm_date' setting
    #[arg(short, long)]
    date: bool,

    #[arg(value_name = "DURATION")]
    duration: String,

    /// Include Worklog comment
    #[arg(short, long = "comment", value_name = "COMMENT")]
    comment_input: Option<String>,

    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,
}

impl ExecCommand for Worklog {
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

        let comment = if let Some(cli_comment) = self.comment_input {
            cli_comment
        } else if cfg.enable_comment_prompts.unwrap_or(false) {
            inquire::Text::new("Worklog comment:")
                .prompt()
                .context("Worklog comment prompt cancelled")?
        } else {
            String::default()
        };

        let wl = PostWorklogBody {
            comment,
            started: interactivity::get_date(cfg, self.date)
                .context("Cannot create worklog request body: missing field=date")?,
            time_spent: WorklogDuration::try_from(self.duration)
                .context("Cannot create worklog request body: missing field=time_spent")?
                .to_string(),
        };

        match client.post_worklog(&issue_key, wl.clone()) {
            Ok(r) if r.status().is_success() => Ok("Worklog created!".to_string()),
            Ok(r) => Err(anyhow!(
                "Worklog creation failed!\n{:?}",
                r.error_for_status()
            )),
            Err(e) => Err(anyhow!("Failed to create worklog:\n{:?}\n{:?}", wl, e)),
        }
    }
}
