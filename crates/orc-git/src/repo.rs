use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("not a git repository")]
    NotARepo,

    #[error("git command failed: {0}")]
    CommandFailed(String),
}

pub struct GitRepo {
    root: PathBuf,
}

impl GitRepo {
    pub fn discover(start: &Path) -> Result<Self, GitError> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(start)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(GitError::NotARepo);
        }

        let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Self { root: PathBuf::from(root) })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn current_branch(&self) -> Result<String, GitError> {
        run_git(&self.root, &["rev-parse", "--abbrev-ref", "HEAD"])
    }

    pub fn status_short(&self) -> Result<String, GitError> {
        run_git(&self.root, &["status", "--short"])
    }

    pub fn diff_staged(&self) -> Result<String, GitError> {
        run_git(&self.root, &["diff", "--cached"])
    }

    pub fn diff_unstaged(&self) -> Result<String, GitError> {
        run_git(&self.root, &["diff"])
    }

    pub fn log_oneline(&self, count: usize) -> Result<String, GitError> {
        let n = format!("-{count}");
        run_git(&self.root, &["log", "--oneline", &n])
    }

    pub fn add(&self, paths: &[&str]) -> Result<String, GitError> {
        let mut args = vec!["add"];
        args.extend(paths);
        run_git(&self.root, &args)
    }

    pub fn commit(&self, message: &str) -> Result<String, GitError> {
        run_git(&self.root, &["commit", "-m", message])
    }
}

fn run_git(dir: &Path, args: &[&str]) -> Result<String, GitError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| GitError::CommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::CommandFailed(stderr.to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
