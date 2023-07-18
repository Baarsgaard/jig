use crate::{
    config::Config,
    interactivity,
    jira::{
        self,
        types::{IssueKey, PostCommentBody},
    },
    repo::Repository,
    ExecCommand,
};
use anyhow::{anyhow, Context};
use clap::Args;

#[derive(Args, Debug)]
pub struct Comment {
    #[arg(value_name = "COMMENT")]
    comment_input: String,

    #[arg(value_name = "ISSUE_KEY")]
    issue_key_input: Option<String>,
}

impl ExecCommand for Comment {
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

        let comment_body = PostCommentBody {
            body: self.comment_input,
        };

        let response = client.post_comment(&issue_key, comment_body)?;
        if response.status().is_success() {
            Ok("Comment posted!".to_string())
        } else {
            Err(anyhow!(
                "Posting comment failed!\n{:?}",
                response.error_for_status()
            ))
        }
    }
}
