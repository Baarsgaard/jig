use crate::{
    config::Config,
    interactivity::{get_date, issue_from_branch_or_prompt},
    repo::Repository,
};
use clap::Args;
use color_eyre::eyre::{eyre, Result, WrapErr};
use jira::{
    types::{IssueKey, PostWorklogBody, WorklogDuration},
    JiraAPIClient,
};

use super::shared::{ExecCommand, UseFilter};

#[derive(Args, Debug)]
pub struct Worklog {
    /// Inverts 'always_confirm_date' setting
    #[arg(short, long)]
    date: bool,

    #[arg(value_name = "DURATION")]
    duration: String,

    /// Include Worklog comment, specifying flag with no value opens prompt
    #[arg(
        short,
        long = "comment",
        value_name = "COMMENT",
        num_args = 0..=1,
        default_value = None,
        default_missing_value = Some("#PROMPT_FOR_COMMENT#")
    )]
    comment_input: Option<String>,

    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,

    #[command(flatten)]
    use_filter: UseFilter,
}

impl ExecCommand for Worklog {
    fn exec(self, cfg: &Config) -> Result<String> {
        let client = JiraAPIClient::new(&cfg.jira_cfg)?;
        let maybe_repo = Repository::open().wrap_err("Failed to open repo");
        let head = match maybe_repo {
            Ok(repo) => repo.get_branch_name(),
            Err(_) => String::default(),
        };

        let issue_key = match self.issue_key_input {
            Some(issue_key_input) => IssueKey::try_from(issue_key_input)?,
            None => issue_from_branch_or_prompt(&client, cfg, head, self.use_filter)?.key,
        };

        let initial_comment = if let Some(cli_comment) = self.comment_input {
            cli_comment
        } else if cfg.enable_comment_prompts.unwrap_or(false) {
            String::from("#PROMPT_FOR_COMMENT#")
        } else {
            String::default()
        };

        let comment = if initial_comment.eq("#PROMPT_FOR_COMMENT#") {
            inquire::Text::new("Worklog comment:")
                .prompt()
                .wrap_err("Worklog comment prompt cancelled")?
        } else {
            initial_comment
        };

        let wl = PostWorklogBody {
            comment,
            started: get_date(cfg, self.date)
                .wrap_err("Cannot create worklog request body: missing field=date")?,
            time_spent: WorklogDuration::try_from(self.duration)
                .wrap_err("Cannot create worklog request body: missing field=time_spent")?
                .to_string(),
        };

        match client.post_worklog(&issue_key, wl.clone()) {
            Ok(r) if r.status().is_success() => Ok("Worklog created!".to_string()),
            Ok(r) => Err(eyre!(
                "Worklog creation failed!\n{:?}",
                r.error_for_status()
            )),
            Err(e) => Err(eyre!("Failed to create worklog:\n{:?}\n{:?}", wl, e)),
        }
    }
}
