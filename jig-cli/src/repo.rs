use crate::config::find_workspace;
use color_eyre::eyre::{eyre, Result, WrapErr};
use color_eyre::Section;
use gix::{Remote, Repository as Gix_Repository, ThreadSafeRepository};
use jira::types::{Issue, IssueKey};
use std::{path::PathBuf, process::Command};

#[derive(Debug, Clone)]
pub struct Repository {
    repo: Gix_Repository,
}

impl Repository {
    pub fn open() -> Result<Self> {
        let (path, _is_repo) = find_workspace();
        Ok(Repository {
            repo: ThreadSafeRepository::open(path)?.to_thread_local(),
        })
    }

    pub fn get_branch_name(&self) -> Result<String> {
        let head_ref = self.repo.head_ref()?;
        let head_commit = self.repo.head_commit()?;

        match head_ref {
            Some(reference) => Ok(reference.name().shorten().to_string()),
            None => Ok(head_commit.id.to_hex_with_len(8).to_string()),
        }
    }

    pub fn issue_branch_exists(&self, issue: &Issue, suffix: Option<String>) -> Result<String> {
        let full_name = Repository::branch_name_from_issue(issue, false, suffix);
        if self.branch_exists(full_name.clone()) {
            Ok(full_name)
        } else if self.branch_exists(issue.key.to_string()) {
            Ok(issue.key.to_string())
        } else {
            Err(eyre!("Issue branch does not exist"))
        }
    }

    pub fn get_origin(&self) -> Result<Remote> {
        let maybe_remote = self
            .repo
            .find_default_remote(gix::remote::Direction::Fetch)
            .transpose()
            .wrap_err("Failed to find default remote")?;

        match maybe_remote {
            Some(remote) => Ok(remote),
            None => Err(eyre!("Failed to parse remote")),
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

    pub fn branch_name_from_issue(
        issue: &Issue,
        use_short: bool,
        suffix: Option<String>,
    ) -> String {
        if use_short {
            issue.key.to_string()
        } else {
            // Sanitize before cut to potentially produce a longer branch name
            // Sanitize after cut ensure branch didn't become invalid (ending with a . or similar)
            let mut initial_branch_name = Self::sanitize_branch_name(&issue.to_string());
            initial_branch_name.truncate(50);

            Self::sanitize_branch_name(&if let Some(suffix_val) = suffix {
                Self::overwriting_suffixer(initial_branch_name, &issue.key, suffix_val)
            } else {
                initial_branch_name
            })
        }
    }

    pub fn overwriting_suffixer(
        mut branch_name: String,
        issue_key: &IssueKey,
        mut suffix: String,
    ) -> String {
        // If suffix plus issue_key_ is longer than 50, discard extra characters to now overwrite issue_key
        if issue_key.0.len() + "_".len() + suffix.len() > 50 {
            let _ = suffix.split_off(51 - (issue_key.0.len() + "_".len()));
        }

        if branch_name.len() + suffix.len() > 50 {
            let _ = branch_name.split_off(51 - suffix.len());
        }

        branch_name.push_str(&suffix);
        branch_name
    }

    pub fn sanitize_branch_name(branch: &str) -> String {
        let mut branch_name = branch.replace([' ', ':', '~', '^', '?', '*', '[', '\\'], "_");
        while branch_name.contains("..") {
            // .... -> .. -> .
            branch_name = branch_name.replace("..", ".");
        }
        while branch_name.contains("${") {
            // $${{ -> $( ->
            branch_name = branch_name.replace("${", "");
        }
        while branch_name.contains(".lock/") {
            // .lock.lock/ -> .lock/ -> /
            branch_name = branch_name.replace(".lock/", "/");
        }

        // /.. will never happen due to .. removal above
        branch_name = branch_name.replace("/.", "/");

        while branch_name.ends_with(['.', '/', '_']) {
            // ././ ->
            branch_name.pop();
        }
        branch_name
    }

    pub fn checkout_branch(&self, branch_name: &str, create_new: bool) -> Result<String> {
        let mut args = vec!["checkout"];
        if create_new {
            args.push("-b");
        }
        args.push(branch_name);

        match Command::new("git").args(args).spawn() {
            Ok(_) => Ok(String::default()),
            Err(e) => Err(e).wrap_err(eyre!("Failed to checkout branch: {}", branch_name)),
        }
    }

    pub fn get_hooks_path() -> Result<PathBuf> {
        let args = vec!["config", "--get", "core.hooksPath"];

        match Command::new("git").args(args).output() {
            Ok(o) => match String::from_utf8_lossy(&o.stdout) {
                output if !output.is_empty() => Ok(PathBuf::from(output.to_string().trim())),
                _ => Self::default_hooks_path(),
            },
            Err(_e) => Self::default_hooks_path(),
        }
    }

    fn default_hooks_path() -> Result<PathBuf> {
        let (workspace_path, is_repo) = find_workspace();

        if is_repo {
            let mut hooks_path = workspace_path;
            hooks_path.push(".git");
            hooks_path.push("hooks");
            Ok(hooks_path)
        } else {
            Err(eyre!("Unable to decide on install location")).wrap_err(
                "Current directory is not a Git repository and global core.hooksPath is undefined",
            ).with_suggestion(|| "Try configuring a global core.hooksPath: git config --global set core.hooksPath <directory>")
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use jira::types::{IssueKey, PostIssueQueryResponseBodyFields};

    fn test_issue(issue_key: Option<IssueKey>, summary: Option<&str>) -> Issue {
        Issue {
            fields: PostIssueQueryResponseBodyFields {
                summary: String::from(summary.unwrap_or("Example summary")),
            },
            id: String::from("10001"),
            key: issue_key.unwrap_or(IssueKey(String::from("JB-1"))),
            self_reference: String::from("https://ddd.ddd.com/"),
            expand: String::from("Don't remember"),
        }
    }

    #[test]
    fn issue_branch_name() {
        let branch_name = Repository::branch_name_from_issue(&test_issue(None, None), false, None);
        assert_eq!(String::from("JB-1_Example_summary"), branch_name);
    }

    #[test]
    fn suffix_branch_name() {
        let branch_name = Repository::branch_name_from_issue(
            &test_issue(None, None),
            false,
            Some(String::from("short suffix")),
        );
        assert_eq!(
            String::from("JB-1_Example_summaryshort_suffix"),
            branch_name
        );
    }

    #[test]
    fn too_long_suffix_branch_name() {
        let branch_name = Repository::branch_name_from_issue(
            &test_issue(None, None),
            false,
            Some(String::from(
                "Clearly too long suffix causing no issues what so ever",
            )),
        );
        assert_eq!(
            String::from("JB-1_Clearly_too_long_suffix_causing_no_issues_what"),
            branch_name
        );
    }

    #[test]
    fn tricky_dot_before_cut_branch_name() {
        let branch_name = Repository::branch_name_from_issue(
            &test_issue(
                None,
                Some("Example summary with a dot at the cut point . which used to cause trouble"),
            ),
            false,
            None,
        );
        assert_eq!(
            String::from("JB-1_Example_summary_with_a_dot_at_the_cut_point"),
            branch_name
        );
    }

    #[test]
    fn long_issue_branch_name() {
        let branch_name = Repository::branch_name_from_issue(
            &test_issue(
                None,
                Some("Example summary that is really long not really but over 50 characters"),
            ),
            false,
            None,
        );
        assert_eq!(
            String::from("JB-1_Example_summary_that_is_really_long_not_reall"),
            branch_name
        );
    }

    #[test]
    fn short_issue_branch_name() {
        let branch_name = Repository::branch_name_from_issue(&test_issue(None, None), true, None);
        assert_eq!(String::from("JB-1"), branch_name);
    }

    #[test]
    fn sanitize_problematic_branch_name() {
        let shit_summary = "ter rible/..bra nch.lock.lock/name$${{....causing/. issues/././";
        let branch_name =
            Repository::branch_name_from_issue(&test_issue(None, Some(shit_summary)), false, None);
        assert_eq!("JB-1_ter_rible/bra_nch/name.causing/_issues", branch_name);
    }
}
