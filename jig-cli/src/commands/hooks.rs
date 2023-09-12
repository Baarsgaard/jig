use crate::repo::Repository;
use clap::Args;
use color_eyre::eyre::{eyre, Result, WrapErr};
use std::env::{current_exe, var};
use std::path::PathBuf;

#[cfg(target_os = "linux")]
use std::os::unix::fs;
#[cfg(target_os = "windows")]
use std::os::windows::fs;

#[derive(Args, Debug)]
pub struct Hooks {}

impl Hooks {
    pub fn install(self) -> Result<String> {
        let bin_path = current_exe().wrap_err("Unable to obtain path of executable (jig)")?;

        let repo = Repository::open().wrap_err("Failed to open repo")?;
        let mut hooks_path = repo.get_hooks_path();
        hooks_path.push("commit-msg");
        if hooks_path.starts_with("~") {
            let expanded_path = match hooks_path.to_str() {
                Some(p) => p.replace('~', &var("HOME")?),
                None => Err(eyre!("Unable to parse hooks path"))?,
            };
            hooks_path = PathBuf::from(expanded_path);
        }

        // TODO Add bin and hooks paths to error message
        fs::symlink(bin_path, hooks_path).wrap_err("Unable to create symbolic link")?;

        Ok(String::from("Installed 'commit-msg' hook"))
    }
}
