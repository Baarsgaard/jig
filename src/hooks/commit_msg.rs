use super::lib::Hook;
use crate::{
    config::Config,
    interactivity::{prompt_user_with_issue_select, query_issues_empty_err},
    repo::{self, Repository},
};
use color_eyre::{eyre::eyre, eyre::WrapErr, Result, Section};
use jira::{models::IssueKey, JiraAPIClient};
use regex::Regex;
use std::{fmt::Display, path::PathBuf};

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
            // Compat: rebase operations
            return Ok(());
        }
        let fixup_commit_re = Regex::new(r"^(squash|fixup|amend|Revert)!?.*")
            .wrap_err("Unable to compile fixup_commits_re")?;
        if fixup_commit_re.captures(&commit_msg).is_some() {
            // Compat: fixup commits
            return Ok(());
        }

        // Processing starts
        let branch_issue_key = IssueKey::try_from(branch.clone());
        let commit_issue_key = IssueKey::try_from(commit_msg.clone());

        let (issue_key, mut msg) =
            match (branch_issue_key, commit_issue_key) {
                // Fail if keys do not match unless allowed
                (Ok(bik), Ok(cik))
                    if bik != cik && !cfg.hooks_cfg.allow_branch_and_commit_msg_mismatch =>
                {
                    Err(eyre!(
                        "Issue key in commit message does not match '{bik}' in the branch name!",
                    )
                    .with_note(|| {
                        "Enabling 'allow_branch_and_commit_msg_mismatch' will skip this check"
                    }))
                }
                // Fail if branch is missing issue key unless allowed
                (Err(_), _) if !cfg.hooks_cfg.allow_branch_missing_issue_key => {
                    Err(eyre!("Issue key not found in branch name")
                        .with_suggestion(|| "create a branch using: jig branch")
                        .with_note(|| {
                            "Enabling 'allow_branch_missing_issue_key' will skip this check"
                        }))
                }

                // Happy path
                (Ok(bik), Err(_)) => Ok((bik, commit_msg)),

                // Remove key from commit message and re-add after
                (_, Ok(cik)) if commit_msg.starts_with(cik.to_string().as_str()) => {
                    let mut msg = commit_msg;
                    msg.replace_range(..cik.to_string().len(), "");

                    Ok((cik, msg.trim().to_string()))
                }
                // Key present in msg but incorrect msg format, move key to front
                (_, Ok(cik)) => {
                    let msg = commit_msg
                        .replace(cik.to_string().as_str(), "")
                        .trim()
                        .to_string();
                    Ok((cik, msg))
                }

                // Commit msg should ALWAYS have an issue key, should only be hit if second Failure condition is skipped
                (Err(_), Err(_)) => {
                    let client = JiraAPIClient::new(&cfg.jira_cfg)?;
                    let issues = query_issues_empty_err(&client, &cfg.issue_query).await?;
                    let issue_key = prompt_user_with_issue_select(issues)?.key;
                    Ok((issue_key, commit_msg))
                }
            }
            .with_suggestion(|| "Skip check with: --no-verify")?;

        let first_char = msg.chars().nth(0).unwrap();
        if first_char.is_ascii_alphabetic() && first_char.is_lowercase() {
            msg.replace_range(..1, &first_char.to_ascii_uppercase().to_string());
        }

        let commit_msg_re = Regex::new(r"^([A-Z]{2,}-[0-9]+) [A-Z0-9].*")
            .wrap_err("Unable to compile commit_msg_re")?;
        let final_msg = format!("{} {}", issue_key, msg);

        // Final sanity check
        if !commit_msg_re.is_match(&final_msg) {
            return Err(eyre!(format!(
                "Commit message not conforming to regex: '{}'",
                commit_msg_re.to_string()
            )));
        }

        CommitMsg::write_commit(self, final_msg)
    }
}
