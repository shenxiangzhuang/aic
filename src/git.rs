use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::process::Command;
use uuid::Uuid;

/// Get the diff for staged changes in the git repository
pub fn get_diff() -> Result<String> {
    // Create a unique temporary file with UUID
    let temp_dir = env::temp_dir();
    let temp_file_path = temp_dir.join(format!("aic_git_diff_{}.txt", Uuid::new_v4()));

    // Ensure the file is removed even if the function panics
    struct TempFileGuard(std::path::PathBuf);
    impl Drop for TempFileGuard {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }
    let _guard = TempFileGuard(temp_file_path.clone());

    // Get the diff of staged changes
    let output = Command::new("git")
        .args(["diff", "--staged"])
        .output()
        .context("Failed to execute git diff command. Make sure git is installed and you're in a git repository.")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Git diff command failed: {}", error));
    }

    // Write the diff to a temporary file
    fs::write(&temp_file_path, &output.stdout)
        .context("Failed to write git diff to temporary file")?;

    // Read the file content
    let diff = fs::read_to_string(&temp_file_path)
        .context("Failed to read git diff from temporary file")?;

    Ok(diff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_get_diff_with_staged_changes() -> Result<()> {
        // Create a temporary git repository
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();

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
