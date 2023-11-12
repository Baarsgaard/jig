use std::io;

use clap::{Args, Command};
use clap_complete::{generate, Shell};
use color_eyre::eyre::Result;

#[derive(Args, Debug)]
pub struct Completion {
    #[arg(value_name = "SHELL", value_enum)]
    generator: Shell,
}

impl Completion {
    pub fn exec(self, cli: &mut Command) -> Result<String> {
        generate(
            self.generator,
            cli,
            cli.get_name().to_string(),
            &mut io::stdout(),
        );
        // ValueHint::
        Ok(String::default())
    }
}
