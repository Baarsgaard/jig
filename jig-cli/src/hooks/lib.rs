use std::path::PathBuf;

use super::*;

pub trait Hook {
    fn new() -> Self;
    fn exec(self, cfg: &Config) -> Result<()>;
}

pub fn is_git_hook() -> Option<impl Hook> {
    let bin_path = PathBuf::from(
        std::env::args()
            .next()
            .expect("First argument to be path to commit_msg_file"),
    );

    match bin_path.file_name().unwrap().to_str() {
        Some("commit-msg") => Some(CommitMsg::new()),
        // Some("applypatch-msg") => Ok(None),
        // Some("pre-applypatch") => Ok(None),
        // Some("post-applypatch") => Ok(None),
        // Some("pre-commit") => Ok(None),
        // Some("pre-merge-commit") => Ok(None),
        // Some("prepare-commit-msg") => Ok(None),
        // Some("post-commit") => Ok(None),
        // Some("pre-rebase") => Some(),
        // Some("post-checkout") => Ok(None),
        // Some("post-merge") => Ok(None),
        // Some("pre-push") => Ok(None),
        // Some("pre-receive") => Ok(None),
        // Some("update") => Ok(None),
        // Some("proc-receive") => Ok(None),
        // Some("post-update") => Ok(None),
        // Some("reference-transaction") => Ok(None),
        // Some("push-to-checkout") => Ok(None),
        // Some("pre-auto-gc") => Ok(None),
        // Some("post-rewrite") => Ok(None),
        // Some("sendemail-validate") => Ok(None),
        // Some("fsmonitor-watchman") => Ok(None),
        // Some("p4-changelist") => Ok(None),
        // Some("p4-prepare-changelist") => Ok(None),
        // Some("p4-post-changelist") => Ok(None),
        // Some("p4-pre-submit") => Ok(None),
        // Some("post-index-change") => Ok(None),
        _ => None,
    }
}
