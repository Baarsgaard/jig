use crate::{
    config::{self, Config},
    ExecCommand,
};
use anyhow::{anyhow, Result};
use clap::Args;

#[derive(Args, Debug)]
pub struct PrintConfigs {}

impl ExecCommand for PrintConfigs {
    fn exec(self, _cfg: &Config) -> Result<String> {
        if config::config_file().exists() {
            println!("Global: {:?}", config::config_file());
        }
        if config::workspace_config_file().exists() {
            println!("workspace: {:?}", config::workspace_config_file());
        }

        if !config::config_file().exists() && !config::workspace_config_file().exists() {
            Err(anyhow!(
                "Config files missing, expected one or both:\n{:?}\n{:?}",
                config::config_file(),
                config::workspace_config_file()
            ))?
        }
        Ok(String::default())
    }
}
