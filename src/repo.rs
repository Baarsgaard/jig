use std::process::{exit, Command};

use crate::config::find_workspace;
use anyhow::{Context, Result};
use gix::{Repository as Gix_Repository, ThreadSafeRepository};

#[derive(Debug, Clone)]
pub struct Repository {
    repo: Gix_Repository,
}

impl Repository {
    pub fn open() -> Result<Self> {
        let repo = ThreadSafeRepository::open(find_workspace())
            .context("Repository load error")?
            .to_thread_local();

        Ok(Repository { repo })
    }

    pub fn get_branch_name(&self) -> String {
        let head_ref = self.repo.head_ref().unwrap();
        let head_commit = self.repo.head_commit().unwrap();

        match head_ref {
            Some(reference) => reference.name().shorten().to_string(),
            None => head_commit.id.to_hex_with_len(8).to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn create_branch(&self, branch_name: String) {
        let result = Command::new("git")
            .args(["checkout", "-b", branch_name.as_str()])
            .spawn();
        match result {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Failed to create branch:\n{}", e);
                exit(1);
            }
        }
    }
}
