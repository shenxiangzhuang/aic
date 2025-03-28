use anyhow::{Context, Result};
use std::process::Command;

/// Get the diff for staged changes in the git repository
pub fn get_diff() -> Result<String> {
    // Run git diff --staged command
    let output = Command::new("git")
        .args(["diff", "--staged"])
        .output()
        .context("Failed to execute git diff command. Make sure git is installed and you're in a git repository.")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Git diff command failed: {}", error));
    }

    let diff =
        String::from_utf8(output.stdout).context("Failed to parse git diff output as UTF-8")?;

    Ok(diff)
}
