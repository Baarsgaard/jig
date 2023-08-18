use crate::{config::Config, ExecCommand};
use clap::Args;
use color_eyre::eyre::Result;

#[derive(Args, Debug)]
pub struct Upgrade {}

impl ExecCommand for Upgrade {
    fn exec(self, _cfg: &Config) -> Result<String> {
        todo!()
    }
}
