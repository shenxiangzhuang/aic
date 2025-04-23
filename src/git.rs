use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

/// Get the diff for staged changes in the git repository
pub fn get_diff() -> Result<String> {
    // Check git installation and is in a repo by `git status`
    let git_status_output = Command::new("git").arg("status").output()?;

    if !git_status_output.status.success() {
        println!(
            "{}",
            "⚠️  Make sure git is installed and you're in a git repository.".yellow()
        );
        return Ok("".to_string());
    }

    // Get the diff of staged changes
    let output = Command::new("git")
        .args(["diff", "--staged"])
        .output()
        .context("Failed to execute git diff command.")?;

    // Parse diff content
    let diff = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok(diff)
}

/// Push committed changes to the remote repository
pub fn push_changes() -> Result<()> {
    println!("{} Running 'git push'...", "▶".green());
    let output = Command::new("git")
        .arg("push")
        .output()
        .context("Failed to execute git push command.")?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr).into_owned();
        eprintln!(
            "{}",
            format!("⚠️  Failed to push changes: {}", error_message).red()
        );
        anyhow::bail!("Git push failed");
    }

    println!("{} Changes pushed successfully.", "✔".green());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use tempfile::Builder;

    #[test]
    fn test_get_diff_with_staged_changes() -> Result<()> {
        // Create a temporary git repository
        let tmp_dir = Builder::new()
            .prefix("test_get_diff_with_staged_changes")
            .tempdir()
            .unwrap();
        let repo_path = tmp_dir.path();

        // Initialize git repository
        Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()?;

        // Configure git user for the test
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()?;
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .output()?;

        // Create and stage a test file
        let test_file = repo_path.join("test.txt");
        let test_content = "Hello, World!";
        File::create(&test_file)?.write_all(test_content.as_bytes())?;

        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(repo_path)
            .output()?;

        // Change to the test directory
        env::set_current_dir(repo_path)?;

        // Get the diff
        let diff = get_diff()?;

        // Verify the diff contains our changes
        let normalized_diff = diff.replace("\r\n", "\n");
        assert!(normalized_diff.contains("test.txt"));
        assert!(normalized_diff.contains(test_content));

        Ok(())
    }
}
