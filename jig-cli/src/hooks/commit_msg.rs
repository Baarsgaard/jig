use crate::interactivity::{prompt_user_with_issue_select, query_issues_with_retry};
use crate::{
    config::Config,
    repo::{self, Repository},
};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use jira::types::IssueKey;
use jira::JiraAPIClient;
use regex::Regex;
use std::path::PathBuf;

use super::lib::Hook;

pub struct CommitMsg {
    commit_msg_file: PathBuf,
    repo: Repository,
}

impl CommitMsg {
    fn write_commit(self, commit_msg: String) -> Result<()> {
        std::fs::write(self.commit_msg_file, commit_msg).wrap_err("Failed to write new commit_msg")
    }
    // fn validate(commit_msg: &String) -> Result<()> {
    //     Ok(())
    // }
}

impl Hook for CommitMsg {
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

    fn exec(self, cfg: &Config) -> Result<()> {
        let mut commit_msg = std::fs::read_to_string(self.commit_msg_file.clone()).unwrap();
        let branch = self.repo.get_branch_name();

        // Pre-checks to verify commit should be processed
        if branch.is_empty() {
            // Sanity check
            return Err(eyre!("Branch is empty, how?"));
        } else if branch == *"HEAD" {
            // Allow rebasing
            return Ok(());
        }
        let fixup_commit_re =
            Regex::new(r"^(squash|fixup|amend)!.*").expect("Unable to compile fixup_commits_re");
        if fixup_commit_re.captures(&commit_msg).is_some() {
            // Allow fixup commits without messages
            return Ok(());
        }

        // Processing starts
        let branch_issue_key = IssueKey::try_from(branch.clone());
        let commit_issue_key = IssueKey::try_from(commit_msg.clone());

        let final_msg = match (branch_issue_key, commit_issue_key) {
            // Most common case
            (Ok(bik), Err(_)) => {
                // Easy uppercase when commit does not contain valid issue key
                let first_char = commit_msg.chars().next().unwrap();
                if first_char.is_ascii_alphabetic() && first_char.is_lowercase() {
                    commit_msg.replace_range(..1, &first_char.to_ascii_uppercase().to_string());
                }

                Ok(format!("{} {}", bik, commit_msg))
            }
            (Ok(bik), Ok(cik)) => {
                if (bik.to_string() != cik.to_string())
                    || !branch.starts_with(&bik.0)
                    || !commit_msg.starts_with(&cik.0)
                {
                    Err(eyre!("Branch and commit 'Issue key' mismatch\nEnsure branch and commit message are *prefixed* with the same Issue key"))
                } else if branch.starts_with(cik.to_string().as_str()) {
                    Ok(commit_msg)
                } else {
                    Err(eyre!("Issue "))
                }
            }
            (Err(_), Ok(_cik)) => {
                // Allow branches without issue key, off by default
                if !cfg.hooks_cfg.allow_branch_missing_issue_key {
                    Err(eyre!(
                        "Branch is missing Issue key, cannot infer commit Issue key"
                    ))
                } else {
                    Ok(commit_msg)
                }
            }
            (Err(_), Err(_)) => {
                // TODO if config allows bik and cik is empty, prompt to select issue
                if !cfg.hooks_cfg.allow_branch_missing_issue_key {
                    Err(eyre!(
                        "Branch is missing Issue key, cannot infer commit Issue key"
                    ))
                } else {
                    let client = JiraAPIClient::new(&cfg.jira_cfg)?;
                    let issues = query_issues_with_retry(&client, cfg)?;
                    let issue_key = prompt_user_with_issue_select(issues)?.key;
                    Ok(format!("{} {}", issue_key, commit_msg))
                }
            }
        }?;

        // TODO Check result against various regexes in a check function to ensure Conformity, if failing, attempt to fix and retry conformity checks
        // TODO Copy error from work script
        // let res = CommitMsg::validate(&final_msg)?;
        CommitMsg::write_commit(self, final_msg)
    }
}
