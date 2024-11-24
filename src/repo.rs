use crate::config::find_workspace;
use color_eyre::eyre::{eyre, Result, WrapErr};
use color_eyre::Section;
use gix::{Repository as Gix_Repository, ThreadSafeRepository};
use jira::models::{Issue, IssueKey};
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

    pub fn get_existing_branches(&self, partial_branch_name: &str) -> Result<Vec<String>> {
        Ok(self
            .repo
            .references()?
            .all()?
            .filter_map(|b| {
                let name = b.unwrap().name().shorten().to_string();
                if name.contains(partial_branch_name) {
                    Some(name.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>())
    }

    pub fn branch_name_from_issue(issue: &Issue, suffix: Option<String>) -> Result<String> {
        // Sanitize before suffix to ensure branch name is as long/descriptive as possible
        let mut initial_branch_name = Self::sanitize_branch_name(&issue.to_string());

        if let Some(suffix_val) = suffix {
            initial_branch_name =
                Self::suffix_branch_name(initial_branch_name, &issue.key, suffix_val)
        }

        let branch_name = Self::sanitize_branch_name(&initial_branch_name);

        // Test branch name is valid by retrieving Issue key from it.
        let _ = IssueKey::try_from(branch_name.clone())?;
        Ok(branch_name)
    }

    fn suffix_branch_name(
        mut branch_name: String,
        issue_key: &IssueKey,
        mut suffix: String,
    ) -> String {
        // If suffix plus ISSUEKEY-0000_ is longer than 50, discard extra characters to now overwrite issue_key
        // Plus 1 for '_'
        if issue_key.to_string().len() + 1 + suffix.len() > 50 {
            let _ = suffix.split_off(51 - (issue_key.to_string().len() + 1));
        }

        if branch_name.len() + suffix.len() > 50 {
            let _ = branch_name.split_off(51 - suffix.len());
        }

        branch_name.push_str(&suffix);
        branch_name
    }

    pub fn sanitize_branch_name(branch: &str) -> String {
        let mut branch_name = branch.replace(' ', "_");

        // Some of the following characters are valid in a branch name, but annoying when pasting
        branch_name = branch_name.replace(
            [' ', ':', '~', '^', '?', '*', '[', '\\', '\'', '"', '<', '>'],
            "",
        );

        loop {
            let diff_check = branch_name.clone();
            while branch_name.contains("..") {
                // ... -> .. -> .
                branch_name = branch_name.replace("..", ".");
            }
            while branch_name.contains("__") {
                // ___ -> __ -> _
                branch_name = branch_name.replace("__", "_");
            }
            while branch_name.contains("--") {
                // --- -> -- -> -
                branch_name = branch_name.replace("--", "-");
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
            branch_name.truncate(51);

            if branch_name == diff_check {
                break;
            }
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
    use jira::models::{IssueFields, IssueKey};

    fn test_issue(issue_key: Option<IssueKey>, summary: Option<&str>) -> Issue {
        Issue {
            fields: IssueFields {
                summary: Some(String::from(summary.unwrap_or("Example summary"))),
                ..IssueFields::default()
            },
            id: String::from("10001"),
            key: issue_key
                .unwrap_or(IssueKey::try_from(String::from("JB-1")).expect("Valid issue key")),
            self_ref: String::from("https://ddd.ddd.com/"),
            expand: Some(String::from("Don't remember")),
            names: None,
        }
    }

    #[test]
    fn branch_name_from_issue() {
        let branch_name =
            Repository::branch_name_from_issue(&test_issue(None, None), None).unwrap();
        assert_eq!(String::from("JB-1_Example_summary"), branch_name);
    }

    #[test]
    fn branch_name_from_issue_with_suffix() {
        let branch_name = Repository::branch_name_from_issue(
            &test_issue(None, None),
            Some(String::from("short suffix")),
        )
        .unwrap();
        assert_eq!(
            String::from("JB-1_Example_summaryshort_suffix"),
            branch_name
        );
    }

    #[test]
    fn branch_name_from_issue_with_too_long_suffix() {
        let branch_name = Repository::branch_name_from_issue(
            &test_issue(None, None),
            Some(String::from(
                "Clearly too long suffix causing no issues what so ever",
            )),
        )
        .unwrap();
        assert_eq!(
            String::from("JB-1_Clearly_too_long_suffix_causing_no_issues_what"),
            branch_name
        );
    }

    #[test]
    fn sanitize_branch_names_ending_with_dot_after_splitoff() {
        let branch_name = Repository::branch_name_from_issue(
            &test_issue(
                None,
                Some("Example summary with a dot at the cut point . which used to cause trouble"),
            ),
            None,
        )
        .unwrap();
        assert_eq!(
            String::from("JB-1_Example_summary_with_a_dot_at_the_cut_point"),
            branch_name
        );
    }

    #[test]
    fn sanitize_branch_long_name() {
        let branch_name = Repository::branch_name_from_issue(
            &test_issue(
                None,
                Some("Example summary that is over fifty characters long."),
            ),
            None,
        )
        .unwrap();
        assert_eq!(
            String::from("JB-1_Example_summary_that_is_over_fifty_characters"),
            branch_name
        );
    }

    #[test]
    fn branch_name_from_challenging_shit_summary() {
        let shit_summary = "ter rible/..bra nch.lock.lock/name$${{....causing/. issues/././";
        let branch_name =
            Repository::branch_name_from_issue(&test_issue(None, Some(shit_summary)), None)
                .unwrap();
        assert_eq!("JB-1_ter_rible/bra_nch/name.causing/_issues", branch_name);
    }
}
