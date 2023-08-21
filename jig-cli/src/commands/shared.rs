use crate::config::Config;
use clap::Args;
use color_eyre::eyre::Result;

pub trait ExecCommand {
    fn exec(self, cfg: &Config) -> Result<String>;
}

#[derive(Debug, Args)]
#[group(required = false)]
pub struct UseFilter {
    /// Prompt for filter to use a default_query
    #[cfg(feature = "cloud")]
    #[arg(short = 'f', long = "filter")]
    pub value: bool,
}
