use crate::config::Config;
use color_eyre::eyre::Result;

pub trait ExecCommand {
    async fn exec(self, cfg: &Config) -> Result<String>;
}
