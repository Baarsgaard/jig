use crate::{config::find_workspace, jira::types::Issue};
use anyhow::{anyhow, Context, Result};
use gix::{Remote, Repository as Gix_Repository, ThreadSafeRepository};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Repository {
    repo: Gix_Repository,
}

impl Repository {
    pub fn open() -> Result<Self> {
        let repo = ThreadSafeRepository::open(find_workspace())
            .context("Repository load error")?
            .to_thread_local();

        Ok(Repository { repo })
    }

    pub fn get_branch_name(&self) -> String {
        let head_ref = self.repo.head_ref().unwrap();
        let head_commit = self.repo.head_commit().unwrap();

        match head_ref {
            Some(reference) => reference.name().shorten().to_string(),
            None => head_commit.id.to_hex_with_len(8).to_string(),
        }
    }

    pub fn issue_branch_exists(&self, issue: &Issue) -> Result<String> {
        let full_name = self.branch_name_from_issue(issue, false);
        if self.branch_exists(full_name.clone()) {
            Ok(full_name)
        } else if self.branch_exists(issue.key.to_string()) {
            Ok(issue.key.to_string())
        } else {
            Err(anyhow!("Issue branch does not exist"))
        }
    }

    pub fn get_origin(&self) -> Result<Remote> {
        let maybe_remote = self
            .repo
            .find_default_remote(gix::remote::Direction::Fetch)
            .transpose()
            .context("Failed to find default remote")?;

        match maybe_remote {
            Some(remote) => Ok(remote),
            None => Err(anyhow!("Failed to parse remote")),
        }
    }

    pub fn branch_exists(&self, branch_name: String) -> bool {
        if self.repo.refs.find(&branch_name).is_ok() {
            return true;
        }

        let origin = match self.get_origin() {
            Ok(o) => o,
            Err(_) => return false,
        };

        let remote_branch_name = match origin.name() {
            Some(origin) => format!("{}/{}", origin.as_bstr(), branch_name),
            None => return false,
        };

        self.repo.refs.find(&remote_branch_name).is_ok()
    }

    pub fn branch_name_from_issue(&self, issue: &Issue, use_short: bool) -> String {
        branch_name_from_issue(issue, use_short)
    }

    pub fn checkout_branch(&self, branch_name: &str, create_new: bool) -> Result<String> {
        let mut args = vec!["checkout"];
        if create_new {
            args.push("-b");
        }
        args.push(branch_name);

        match Command::new("git").args(args).spawn() {
            Ok(_) => Ok(String::default()),
            Err(e) => Err(e).context(anyhow!("Failed to checkout branch: {}", branch_name)),
        }
    }
}

pub fn branch_name_from_issue(issue: &Issue, use_short: bool) -> String {
    if use_short {
        issue.key.to_string()
    } else {
        let branch_name = issue.to_string().replace(' ', "_");
        match branch_name.len() {
            n if n > 50 => branch_name.split_at(51).0.to_owned(),
            _ => branch_name,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::jira::types::{IssueKey, PostIssueQueryResponseBodyFields};

    #[test]
    fn issue_branch_name() {
        let issue_key = IssueKey(String::from("JB-1"));
        let issue = Issue {
            fields: PostIssueQueryResponseBodyFields {
                summary: String::from("Example summary"),
            },
            id: String::from("10001"),
            key: issue_key,
            self_reference: String::from("https://ddd.ddd.com/"),
            expand: String::from("Don't remember"),
        };
        let branch_name = branch_name_from_issue(&issue, false);
        assert_eq!(String::from("JB-1_Example_summary"), branch_name);
    }

    #[test]
    fn long_issue_branch_name() {
        let issue_key = IssueKey(String::from("JB-1"));
        let issue = Issue {
            fields: PostIssueQueryResponseBodyFields {
                summary: String::from(
                    "Example summary that is really long not really but over 50 characters",
                ),
            },
            id: String::from("10001"),
            key: issue_key,
            self_reference: String::from("https://ddd.ddd.com/"),
            expand: String::from("Don't remember"),
        };
        let branch_name = branch_name_from_issue(&issue, false);
        assert_eq!(
            String::from("JB-1_Example_summary_that_is_really_long_not_really"),
            branch_name
        );
    }

    #[test]
    fn short_issue_branch_name() {
        let issue_key = IssueKey(String::from("JB-1"));
        let issue = Issue {
            fields: PostIssueQueryResponseBodyFields {
                summary: String::from("Example summary"),
            },
            id: String::from("10001"),
            key: issue_key,
            self_reference: String::from("https://ddd.ddd.com/"),
            expand: String::from("Don't remember"),
        };

        let branch_name = branch_name_from_issue(&issue, true);
        assert_eq!(String::from("JB-1"), branch_name);
    }
}
