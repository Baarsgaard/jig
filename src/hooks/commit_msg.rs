use super::lib::Hook;
use crate::interactivity::{prompt_user_with_issue_select, query_issues_with_retry};
use crate::{
    config::Config,
    repo::{self, Repository},
};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use jira::types::IssueKey;
use jira::JiraAPIClient;
use regex::Regex;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Debug)]
pub struct CommitMsg {
    commit_msg_file: PathBuf,
    repo: Repository,
}

impl CommitMsg {
    fn write_commit(self, commit_msg: String) -> Result<()> {
        std::fs::write(self.commit_msg_file, commit_msg).wrap_err("Failed to write new commit_msg")
    }
}

impl Display for CommitMsg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Self::hook_name())
    }
}

impl Hook for CommitMsg {
    fn hook_name() -> String {
        String::from("commit-msg")
    }
    fn new() -> CommitMsg {
        let commit_msg_file = PathBuf::from(
            std::env::args()
                .nth(1)
                .expect("Expected commit_msg_file as first argument"),
        );
        let repo = repo::Repository::open()
            .wrap_err("Failed to open repository")
            .unwrap();

        CommitMsg {
            commit_msg_file,
            repo,
        }
    }

    async fn exec(self, cfg: &Config) -> Result<()> {
        let commit_msg = std::fs::read_to_string(self.commit_msg_file.clone()).unwrap();
        let branch = self.repo.get_branch_name()?;

        // Pre-checks to verify commit should be processed
        if branch.is_empty() {
            // Sanity check
            return Err(eyre!("Branch is empty, how?"));
        } else if branch == *"HEAD" {
            // Allow rebasing
            return Ok(());
        }
        let fixup_commit_re = Regex::new(r"^(squash|fixup|amend|Revert)!?.*")
            .wrap_err("Unable to compile fixup_commits_re")?;
        if fixup_commit_re.captures(&commit_msg).is_some() {
            // Allow fixup commits without messages
            return Ok(());
        }

        // Processing starts
        let branch_issue_key = IssueKey::try_from(branch.clone());
        let commit_issue_key = IssueKey::try_from(commit_msg.clone());

        let (issue_key, mut msg) = match (branch_issue_key, commit_issue_key) {
            // Most common case
            (Ok(bik), Err(_)) => Ok((bik.0, commit_msg)),
            (Ok(bik), Ok(cik)) => {
                if (bik.to_string() != cik.to_string())
                    || !branch.starts_with(&bik.0)
                    || !commit_msg.starts_with(&cik.0)
                {
                    Err(eyre!(
                        "Jira issue key in message does not match '{}' in the branch name!",
                        bik.0
                    ))
                } else if branch.starts_with(cik.to_string().as_str()) {
                    let mut msg = commit_msg.clone();
                    msg.replace_range(..cik.to_string().len(), "");
                    if msg.starts_with(':') {
                        msg = msg.trim().to_string();
                    }
                    Ok((bik.0, msg))
                } else {
                    Err(eyre!("Issue "))
                }
            }
            (Err(_), Ok(cik)) => {
                // Allow branches without issue key, off by default
                if !cfg.hooks_cfg.allow_branch_missing_issue_key {
                    Err(eyre!(
                        "Branch is missing Issue key, create branch from issue with:\njig branch"
                    ))
                } else {
                    let mut msg = commit_msg.clone();
                    msg.replace_range(..cik.to_string().len(), "");
                    Ok((cik.0, msg.trim().to_string()))
                }
            }
            (Err(_), Err(_)) => {
                if !cfg.hooks_cfg.allow_branch_missing_issue_key {
                    Err(eyre!(
                        "Branch is missing Issue key, create branch from issue with:\njig branch"
                    ))
                } else {
                    let client = JiraAPIClient::new(&cfg.jira_cfg)?;
                    let issues = query_issues_with_retry(&client, cfg).await?;
                    let issue_key = prompt_user_with_issue_select(issues)?.key;
                    Ok((issue_key.0, commit_msg))
                }
            }
        }?;

        let first_char = msg.chars().next().unwrap();
        if first_char.is_ascii_alphabetic() && first_char.is_lowercase() {
            msg.replace_range(..1, &first_char.to_ascii_uppercase().to_string());
        }

        let final_msg = format!("{} {}", issue_key, msg);

        // TODO Consider optional colons after issue key: ':?'
        let commit_msg_re = Regex::new(r"^([A-Z]{2,}-[0-9]+) [A-Z0-9].*")
            .wrap_err("Unable to compile commit_msg_re")?;
        if !commit_msg_re.is_match(&final_msg) {
            return Err(eyre!(format!(
                "Commit message not conforming to regex: '{}'",
                commit_msg_re.to_string()
            )));
        }

        // TODO Copy error from work script
        CommitMsg::write_commit(self, final_msg)
    }
}
