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
    use anyhow::Result;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    // Modified version of get_diff that accepts a path parameter
    // for testing purposes only
    fn get_diff_in_dir(dir: &PathBuf) -> Result<String> {
        // Run git diff --staged command in the specified directory
        let output = Command::new("git")
            .args(["diff", "--staged"])
            .current_dir(dir)
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

    // Helper function to set up a test git repository
    fn setup_git_repo() -> Result<TempDir> {
        // Create a temporary directory
        let temp_dir = tempfile::tempdir()?;

        // Initialize git repository
        let output = Command::new("git")
            .args(["init"])
            .current_dir(&temp_dir.path())
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            println!("Git init error: {}", error);
            return Err(anyhow::anyhow!("Failed to initialize git repository"));
        }

        // Configure git user for the repository
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&temp_dir.path())
            .output()?;

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&temp_dir.path())
            .output()?;

        Ok(temp_dir)
    }

    // Helper function to create a file and stage it
    fn create_and_stage_file(repo_dir: &PathBuf, filename: &str, content: &str) -> Result<()> {
        // Print the absolute path for debugging
        let file_path = repo_dir.join(filename);
        println!("Creating file at: {}", file_path.display());

        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create the file with content
        let mut file = File::create(&file_path)?;
        writeln!(file, "{}", content)?;

        // Stage the file
        let output = Command::new("git")
            .args(["add", filename])
            .current_dir(repo_dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            println!("Git add error: {}", error);
            return Err(anyhow::anyhow!("Failed to stage file"));
        }

        Ok(())
    }

    // Helper function to modify a file and stage the changes
    fn modify_and_stage_file(repo_dir: &PathBuf, filename: &str, content: &str) -> Result<()> {
        // Print the absolute path for debugging
        let file_path = repo_dir.join(filename);
        println!("Modifying file at: {}", file_path.display());

        // Check if file exists
        if !file_path.exists() {
            return Err(anyhow::anyhow!(
                "File does not exist: {}",
                file_path.display()
            ));
        }

        // Modify the file
        let mut file = File::create(&file_path)?;
        writeln!(file, "{}", content)?;

        // Stage the file
        let output = Command::new("git")
            .args(["add", filename])
            .current_dir(repo_dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            println!("Git add error: {}", error);
            return Err(anyhow::anyhow!("Failed to stage modified file"));
        }

        Ok(())
    }

    // Helper function to commit changes
    fn commit_changes(repo_dir: &PathBuf, message: &str) -> Result<()> {
        println!("Committing changes with message: {}", message);

        let output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(repo_dir)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            println!("Git commit error: {}", error);
            return Err(anyhow::anyhow!("Failed to commit changes"));
        }

        Ok(())
    }

    // The actual tests

    #[test]
    fn test_empty_diff_when_no_changes() -> Result<()> {
        // Create a temporary git repository
        let temp_dir = setup_git_repo()?;
        let repo_path = temp_dir.path().to_path_buf();

        println!("Temp directory: {}", repo_path.display());

        // Get the diff - should be empty since no changes
        let diff = get_diff_in_dir(&repo_path)?;

        // Verify the diff is empty
        assert!(diff.is_empty());
        Ok(())
    }

    #[test]
    fn test_diff_with_new_file() -> Result<()> {
        // Current Date and Time for test context
        println!("📅 Test date: 2025-03-28 09:49:37");
        println!("👤 Test user: shenxiangzhuang");

        // Create a temporary git repository
        let temp_dir = setup_git_repo()?;
        let repo_path = temp_dir.path().to_path_buf();

        println!("Temp directory: {}", repo_path.display());

        // Create and stage a new file with specific content
        let test_content = "Hello, World!";
        create_and_stage_file(&repo_path, "test.txt", test_content)?;

        // Get the diff
        let diff = get_diff_in_dir(&repo_path)?;

        // For debugging, print the actual diff
        println!("Actual diff output:\n{}", diff);

        // More flexible checking for diff content
        assert!(!diff.is_empty(), "Diff should not be empty");

        // Check for the file name (this should be present regardless of Git version)
        assert!(
            diff.contains("test.txt"),
            "Diff should mention the filename 'test.txt'"
        );

        // Check for the content (should be present in all Git diff formats)
        assert!(
            diff.contains(test_content),
            "Diff should contain the file content"
        );

        Ok(())
    }

    #[test]
    fn test_diff_with_modified_file() -> Result<()> {
        // Current Date and Time for test context
        println!("📅 Test date: 2025-03-28 09:49:37");
        println!("👤 Test user: shenxiangzhuang");

        // Create a temporary git repository
        let temp_dir = setup_git_repo()?;
        let repo_path = temp_dir.path().to_path_buf();

        println!("Temp directory: {}", repo_path.display());

        // Create and commit a file
        let filename = "test.txt";
        println!("Creating initial file: {}", filename);
        create_and_stage_file(&repo_path, filename, "Original content")?;

        // Commit the initial file
        commit_changes(&repo_path, "Initial commit")?;

        // Modify and stage the file
        println!("Modifying file: {}", filename);
        modify_and_stage_file(&repo_path, filename, "Modified content")?;

        // Get the diff
        let diff = get_diff_in_dir(&repo_path)?;

        // For debugging, print the actual diff
        println!("Modified file diff output:\n{}", diff);

        // Verify the diff shows the modification
        assert!(!diff.is_empty(), "Diff should not be empty");

        // More robust checking that accounts for different Git versions
        // and line ending variations
        assert!(
            diff.contains("Original") || diff.contains("-Original"),
            "Diff should show removed content"
        );
        assert!(
            diff.contains("Modified") || diff.contains("+Modified"),
            "Diff should show added content"
        );

        Ok(())
    }

    #[test]
    fn test_diff_with_multiple_files() -> Result<()> {
        // Current Date and Time for test context
        println!("📅 Test date: 2025-03-28 09:49:37");
        println!("👤 Test user: shenxiangzhuang");

        // Create a temporary git repository
        let temp_dir = setup_git_repo()?;
        let repo_path = temp_dir.path().to_path_buf();

        println!("Temp directory: {}", repo_path.display());

        // Create and stage multiple files
        println!("Creating file1.txt");
        create_and_stage_file(&repo_path, "file1.txt", "Content of file 1")?;

        println!("Creating file2.txt");
        create_and_stage_file(&repo_path, "file2.txt", "Content of file 2")?;

        // List the repo contents for debugging
        println!("Repository contents:");
        let entries = fs::read_dir(&repo_path)?;
        for entry in entries {
            if let Ok(entry) = entry {
                println!("  {}", entry.path().display());
            }
        }

        // Get the diff
        let diff = get_diff_in_dir(&repo_path)?;

        // For debugging, print the actual diff
        println!("Multiple files diff output:\n{}", diff);

        // Verify the diff contains both files
        assert!(!diff.is_empty(), "Diff should not be empty");
        assert!(diff.contains("file1.txt"), "Diff should mention file1.txt");
        assert!(diff.contains("file2.txt"), "Diff should mention file2.txt");
        assert!(
            diff.contains("Content of file 1"),
            "Diff should contain content of file1"
        );
        assert!(
            diff.contains("Content of file 2"),
            "Diff should contain content of file2"
        );

        Ok(())
    }

    // Test that the function returns an error in a non-git directory
    #[test]
    fn test_error_in_non_git_directory() -> Result<()> {
        // Create a temporary directory (not a git repo)
        let temp_dir = tempfile::tempdir()?;
        println!("Non-git directory test path: {}", temp_dir.path().display());

        // Get the diff - should error
        let result = get_diff_in_dir(&temp_dir.path().to_path_buf());

        // Verify we got an error
        assert!(result.is_err(), "Should get an error in non-git directory");
        let error = result.unwrap_err().to_string();
        println!("Error message: {}", error);

        Ok(())
    }

    // Test with the current date and username
    #[test]
    fn test_diff_with_user_context() -> Result<()> {
        // Create a temporary git repository with specific file content
        let temp_dir = setup_git_repo()?;
        let repo_path = temp_dir.path().to_path_buf();

        println!("Temp directory: {}", repo_path.display());

        // Create a file with user context information
        let content = "Test file with user context";
        create_and_stage_file(&repo_path, "user_context_file.txt", content)?;

        // Get the diff
        let diff = get_diff_in_dir(&repo_path)?;

        // Verify the diff contains our content
        assert!(!diff.is_empty(), "Diff should not be empty");
        assert!(
            diff.contains(content),
            "Diff should contain the test content"
        );

        Ok(())
    }
}
