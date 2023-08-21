use clap::Args;
use color_eyre::eyre::{Result, WrapErr};
use inquire::Select;
use self_update::{
    backends::github::{ReleaseList, Update},
    update::Release,
};
use std::io::{stdout, IsTerminal};
use std::{fmt::Display, process::Command, time::Duration};

#[derive(Args, Debug)]
pub struct Upgrade {
    /// Skip confirmation
    #[arg(short, long)]
    force: bool,

    /// Suppress all output
    /// Quiet implies force and disables verbose
    #[arg(short, long)]
    quiet: bool,

    /// Extra progress output
    #[arg(short, long)]
    verbose: bool,

    /// Manually select release Github release
    #[arg(short, long)]
    select: bool,
}

impl Upgrade {
    pub fn exec(self) -> Result<String> {
        // If any pattern matches, do not prompt.
        let do_confirm = !matches!(
            (stdout().is_terminal(), self.quiet, self.force),
            (false, _, _) | (true, true, _) | (true, _, true)
        );
        let do_verbose = self.verbose && !self.quiet;

        let version = if self.select {
            let raw_releases = ReleaseList::configure()
                .repo_owner("raunow")
                .repo_name("jig")
                .build()?
                .fetch()
                .wrap_err("Unable to fetch list of releases")?;

            let releases = raw_releases
                .iter()
                .map(|r| JigRelease(r.to_owned()))
                .collect();

            Select::new("Release: ", releases).prompt()?.0.version
        } else {
            #[cfg(debug_assertions)]
            let version = "0.0.0";
            #[cfg(not(debug_assertions))]
            let version = self_update::cargo_crate_version!();

            version.to_string()
        };

        let _status = Update::configure()
            .repo_owner("raunow")
            .repo_name("jig")
            .bin_name("jig")
            .show_output(do_verbose)
            .current_version(version.as_str())
            .show_download_progress(!self.quiet)
            .no_confirm(do_confirm)
            .build()?
            .update()?;

        std::thread::sleep(Duration::from_millis(100));
        if !self.quiet {
            println!("\n$ jig --version");
        }
        let output = Command::new("jig")
            .args(["--version"])
            .spawn()
            .wrap_err("Failed to execute: jig --version")?
            .wait_with_output()?;

        Ok(String::from_utf8(output.stdout)?)
    }
}

struct JigRelease(Release);
impl Display for JigRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.name)
    }
}
