use crate::{config::Config, interactivity::issue_from_branch_or_prompt, repo::Repository};
use clap::Args;
use color_eyre::eyre::{eyre, Result, WrapErr};
use jira::types::{IssueKey, PostCommentBody};
use jira::JiraAPIClient;

use super::shared::{ExecCommand, UseFilter};

#[derive(Args, Debug)]
pub struct Comment {
    #[arg(value_name = "COMMENT")]
    comment_input: String,

    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,

    #[command(flatten)]
    use_filter: UseFilter,
}

impl ExecCommand for Comment {
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

        let comment_body = PostCommentBody {
            body: self.comment_input,
        };

        let response = client.post_comment(&issue_key, comment_body)?;
        if response.status().is_success() {
            Ok("Comment posted!".to_string())
        } else {
            Err(eyre!(
                "Posting comment failed!\n{:?}",
                response.error_for_status()
            ))
        }
    }
}
