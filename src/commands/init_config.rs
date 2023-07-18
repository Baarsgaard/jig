use crate::{
    config::Config,
    // repo::Repository,
    ExecCommand,
};
// use anyhow::Context;
use clap::Args;

#[derive(Args, Debug)]
pub struct InitConfig {
    #[arg(short, long)]
    all: bool,
}

impl ExecCommand for InitConfig {
    fn exec(self, _cfg: &Config) -> anyhow::Result<String> {
        todo!()
    }
}
