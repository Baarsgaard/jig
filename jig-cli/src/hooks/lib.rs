use color_eyre::eyre::{ContextCompat, Result};
use std::path::PathBuf;

use super::*;

pub trait Hook {
    fn new() -> Self;
    fn exec(self, cfg: &Config) -> Result<()>;
}

pub fn is_git_hook() -> Result<Option<impl Hook>> {
    let bin_path = PathBuf::from(
        std::env::args()
            .next()
            .wrap_err("First argument to be path to commit_msg_file")?,
    );

    Ok(match bin_path.file_name().unwrap().to_str() {
        Some("commit-msg") => Some(CommitMsg::new()),
        // Some("applypatch-msg") => None,
        // Some("pre-applypatch") => None,
        // Some("post-applypatch") => None,
        // Some("pre-commit") => None,
        // Some("pre-merge-commit") => None,
        // Some("prepare-commit-msg") => None,
        // Some("post-commit") => None,
        // Some("pre-rebase") => None,
        // Some("post-checkout") => None,
        // Some("post-merge") => None,
        // Some("pre-push") => None,
        // Some("pre-receive") => None,
        // Some("update") => None,
        // Some("proc-receive") => None,
        // Some("post-update") => None,
        // Some("reference-transaction") => None,
        // Some("push-to-checkout") => None,
        // Some("pre-auto-gc") => None,
        // Some("post-rewrite") => None,
        // Some("sendemail-validate") => None,
        // Some("fsmonitor-watchman") => None,
        // Some("p4-changelist") => None,
        // Some("p4-prepare-changelist") => None,
        // Some("p4-post-changelist") => None,
        // Some("p4-pre-submit") => None,
        // Some("post-index-change") => None,
        _ => None,
    })
}
