use std::fmt::Debug;

use crate::{
    config::Config,
    interactivity::{issue_from_branch_or_prompt, now},
    repo::Repository,
};
use chrono::{NaiveDate, Weekday};
use clap::{Args, ValueHint};
use color_eyre::eyre::{eyre, Result, WrapErr};
use inquire::DateSelect;
use jira::{
    models::{IssueKey, PostWorklogBody, WorklogDuration},
    JiraAPIClient,
};

use super::shared::{ExecCommand, UseFilter};

#[derive(Args, Debug)]
pub struct Worklog {
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

    /// Skip prompt by inputting date.
    /// Format: %Y-%m-%d -> 2005-02-18
    /// "now" and "today" is also valid input
    #[arg(short, long, value_hint = ValueHint::Unknown)]
    date: Option<String>,

    /// Formats: 30m, 1.5h, 2d, 1w
    /// Default unit, if omitted, is minutes.
    #[arg(value_name = "DURATION", value_hint = ValueHint::Unknown)]
    duration: String,

    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,

    #[command(flatten)]
    use_filter: UseFilter,
}

impl ExecCommand for Worklog {
    async fn exec(self, cfg: &Config) -> Result<String> {
        let client = JiraAPIClient::new(&cfg.jira_cfg)?;
        let maybe_repo = Repository::open().wrap_err("Failed to open repo");
        let head = match maybe_repo {
            Ok(repo) => repo.get_branch_name()?,
            Err(_) => String::default(),
        };

        // issue key
        let issue_key = match self.issue_key_input {
            Some(issue_key_input) => IssueKey::try_from(issue_key_input)?,
            None => {
                issue_from_branch_or_prompt(&client, cfg, head, self.use_filter)
                    .await?
                    .key
            }
        };

        // worklog date
        let worklog_date = if let Some(date) = self.date {
            match date.to_lowercase().as_str() {
                "now" | "today" => now(),
                _ => {
                    let date_input =
                        NaiveDate::parse_from_str(date.as_str(), "%Y-%m-%d")?.to_string();
                    // Prettier than a complex of format!()
                    date_input + &now()[10..]
                }
            }
        } else {
            let date_input = DateSelect::new("")
                .with_week_start(Weekday::Mon)
                .prompt()?
                .to_string();

            // Prettier than a complex of format!()
            date_input + &now()[10..]
        };

        // Parse comment input
        let initial_comment = if let Some(cli_comment) = self.comment_input {
            cli_comment
        } else if cfg.enable_comment_prompts.unwrap_or(false) {
            String::from("#PROMPT_FOR_COMMENT#")
        } else {
            String::default()
        };

        // Prompt for comment if default
        let comment = if initial_comment.eq("#PROMPT_FOR_COMMENT#") {
            inquire::Text::new("Worklog comment:")
                .prompt()
                .wrap_err("Worklog comment prompt cancelled")?
        } else {
            initial_comment
        };

        let wl = PostWorklogBody {
            comment,
            started: worklog_date,
            time_spent: None,
            time_spent_seconds: Some(
                WorklogDuration::try_from(self.duration)
                    .wrap_err("Cannot create worklog request body: missing field=time_spent")?
                    .to_string(),
            ),
        };

        match client.post_worklog(&issue_key, wl.clone()).await {
            Ok(r) if r.status().is_success() => Ok("Worklog created!".to_string()),
            Ok(r) => Err(eyre!(
                "Worklog creation failed!\n{:?}",
                r.error_for_status()
            )),
            Err(e) => Err(eyre!("Failed to create worklog:\n{:?}\n{:?}", wl, e)),
        }
    }
}
