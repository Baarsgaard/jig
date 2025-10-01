use clap::Args;
use color_eyre::Section;
use color_eyre::eyre::{Result, WrapErr};
use inquire::Select;
use self_update::{
    backends::github::{ReleaseList, Update},
    update::Release,
};
use std::{env, fmt::Display, thread};

#[derive(Args, Debug)]
pub struct Upgrade {
    /// Suppress all output
    /// Quiet implies force and disables verbose
    #[arg(short, long)]
    quiet: bool,

    /// Manually select release Github release
    #[arg(short, long)]
    select: bool,
}

impl Upgrade {
    pub async fn upgrade(self) -> Result<String> {
        thread::spawn(move || {
            let token = env::var("GITHUB_TOKEN").unwrap_or_default();
            let target_ver = if self.select {
                let mut builder = ReleaseList::configure();
                let mut releases_cfg = builder.repo_owner("baarsgaard").repo_name("jig");
                if !token.is_empty() {
                    releases_cfg = releases_cfg.auth_token(&token);
                }

                let raw_releases = releases_cfg
                    .build()?
                    .fetch()
                    .wrap_err("Unable to fetch list of releases")
                    .with_suggestion(
                        || "If ratelimited: export GITHUB_TOKEN='insert_token_here'",
                    )?;

                let releases = raw_releases
                    .iter()
                    .map(|r| JigRelease(r.to_owned()))
                    .collect();

                Select::new("Release: ", releases).prompt()?.0.version
            } else {
                String::default()
            };

            let current_ver = if cfg!(debug_assertions) {
                "0.0.0"
            } else {
                self_update::cargo_crate_version!()
            };

            let mut builder = Update::configure();
            let mut cfg = builder
                .repo_owner("baarsgaard")
                .repo_name("jig")
                .bin_name("jig")
                .show_output(!self.quiet)
                .current_version(current_ver)
                .show_download_progress(!self.quiet)
                .no_confirm(true);
            if !target_ver.is_empty() {
                cfg = cfg.target_version_tag(&format!("v{target_ver}"));
            }
            if !token.is_empty() {
                cfg = cfg.auth_token(&token);
            }

            let _ = cfg
                .build()?
                .update()
                .with_suggestion(|| "If ratelimited: export GITHUB_TOKEN='insert_token_here'")?;

            Ok(String::default())
        })
        .join()
        .expect("Self update wrapper thread panicked")
    }
}

struct JigRelease(Release);
impl Display for JigRelease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.name)
    }
}
