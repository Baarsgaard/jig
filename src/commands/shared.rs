use crate::config::Config;
use clap::Args;
use color_eyre::eyre::Result;

pub trait ExecCommand {
    async fn exec(self, cfg: &Config) -> Result<String>;
}

#[derive(Debug, Args)]
#[group(required = false)]
pub struct UseFilter {
    /// Prompt for filter to use as default_query
    #[cfg(feature = "cloud")]
    #[arg(long = "filter")]
    pub value: bool,
}
