use crate::repo::Repository;
use clap::Args;
use color_eyre::eyre::{eyre, Result, WrapErr};
use color_eyre::Section;
use inquire::Confirm;
use std::env::{current_exe, var};
use std::path::PathBuf;

#[cfg(target_os = "linux")]
use std::os::unix::fs::symlink;
#[cfg(target_os = "windows")]
use std::os::windows::fs::symlink_file;

#[derive(Args, Debug)]
pub struct Hooks {
    /// Skip confirmation
    #[arg(short, long)]
    pub force: bool,
}

impl Hooks {
    pub fn install(self) -> Result<String> {
        let bin_path = current_exe().wrap_err("Unable to obtain path of executable (jig)")?;

        let mut hooks_path = Repository::get_hooks_path()?;
        if hooks_path.starts_with("~") {
            let expanded_path = match hooks_path.to_str() {
                Some(p) => p.replace('~', &var("HOME")?),
                None => Err(eyre!("Unable to parse hooks path"))?,
            };
            hooks_path = PathBuf::from(expanded_path);
        }

        hooks_path.push("commit-msg");
        if hooks_path.exists() {
            let replace_hook = match self.force {
                true => true,
                false => Confirm::new(
                    format!(
                        "Hook already exists, replace: {}",
                        hooks_path.clone().to_str().unwrap()
                    )
                    .as_str(),
                )
                .with_default(true)
                .with_help_message("--force to skip this prompt")
                .prompt()?,
            };

            if replace_hook {
                std::fs::remove_file(hooks_path.clone())?;
            }
        }

        #[cfg(target_os = "linux")]
        symlink(bin_path, hooks_path.clone())
            .wrap_err("Unable to create symbolic link")
            .with_note(|| format!("target: {}", hooks_path.to_str().unwrap()))?;

        #[cfg(target_os = "windows")]
        symlink_file(bin_path, hooks_path.clone())
            .wrap_err("Unable to create symbolic link, try to run as administrator.")
            .with_note(|| format!("target: {}", hooks_path.to_str().unwrap()))?;

        Ok(String::from("Installed 'commit-msg' hook"))
    }
}
