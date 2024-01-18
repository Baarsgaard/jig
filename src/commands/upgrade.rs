use clap::Args;
use color_eyre::eyre::{Result, WrapErr};
use inquire::Select;
use self_update::{
    backends::github::{ReleaseList, Update},
    update::Release,
};
use std::fmt::Display;
use std::io::{stdout, IsTerminal};

#[derive(Args, Debug)]
pub struct Upgrade {
    /// Skip confirmation
    #[arg(short, long)]
    force: bool,

    /// Suppress all output
    /// Quiet implies force and disables verbose
    #[arg(short, long)]
    quiet: bool,

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

        let version = if self.select {
            let raw_releases = ReleaseList::configure()
                .repo_owner("baarsgaard")
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
            .repo_owner("baarsgaard")
            .repo_name("jig")
            .bin_name("jig")
            .show_output(!self.quiet)
            .current_version(version.as_str())
            .show_download_progress(!self.quiet)
            .no_confirm(do_confirm)
            .build()?
            .update()
            .wrap_err("Unable to replace binary")?;

        Ok(String::default())
    }
}

struct JigRelease(Release);
impl Display for JigRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.name)
    }
}
