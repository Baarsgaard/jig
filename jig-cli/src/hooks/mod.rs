use crate::config::Config;
use color_eyre::Result;

mod commit_msg;
mod lib;

pub use commit_msg::CommitMsg;
pub use lib::*;
