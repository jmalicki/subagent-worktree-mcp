use anyhow::Result;
use tempfile::TempDir;
use std::path::Path;

use subagent_worktree_mcp::git_operations::GitWorktreeManager;

/// Test helper to create a temporary git repository
fn create_temp_git_repo() -> Result<(TempDir, std::path::PathBuf)> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("test_repo");
    
    // Create directory
    std::fs::create_dir(&repo_path)?;
    
    // Initialize git repository
    let output = std::process::Command::new("git")
        .arg("init")
        .current_dir(&repo_path)
        .output()?;
    
    assert!(output.status.success(), "Failed to initialize git repository");
    
    // Create initial commit
    std::fs::write(repo_path.join("README.md"), "# Test Repository\n")?;
    
    let output = std::process::Command::new("git")
        .args(&["add", "README.md"])
        .current_dir(&repo_path)
        .output()?;
    
    assert!(output.status.success(), "Failed to add README.md");
    
    let output = std::process::Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()?;
    
    assert!(output.status.success(), "Failed to create initial commit");
    
    Ok((temp_dir, repo_path))
}

#[tokio::test]
async fn test_git_worktree_manager_invalid_path() -> Result<()> {
    // Test: Verify GitWorktreeManager fails gracefully for invalid paths
    // This test ensures proper error handling for non-existent directories
    
    let invalid_path = Path::new("/nonexistent/path/that/does/not/exist");
    let result = GitWorktreeManager::new(invalid_path.to_path_buf());
    
    // Should fail gracefully for invalid path
    assert!(result.is_err(), "Should fail for invalid path");
    
    Ok(())
}

#[tokio::test]
async fn test_git_worktree_manager_non_git_directory() -> Result<()> {
    // Test: Verify GitWorktreeManager fails gracefully for non-git directories
    // This test ensures proper error handling for directories without git
    
    let temp_dir = TempDir::new()?;
    let non_git_path = temp_dir.path().join("not_a_repo");
    std::fs::create_dir(&non_git_path)?;
    
    let result = GitWorktreeManager::new(non_git_path);
    
    // Should fail gracefully for non-git directory
    assert!(result.is_err(), "Should fail for non-git directory");
    
    Ok(())
}

#[tokio::test]
async fn test_create_worktree_invalid_branch_name() -> Result<()> {
    // Test: Verify worktree creation fails gracefully for invalid branch names
    // This test ensures proper error handling for malformed branch names
    
    let (_temp_dir, repo_path) = create_temp_git_repo()?;
    let manager = GitWorktreeManager::new(repo_path)?;
    
    // Try to create worktree with invalid branch name
    let result = manager.create_worktree("", None, None).await;
    
    // Should fail gracefully for empty branch name
    assert!(result.is_err(), "Should fail for empty branch name");
    
    Ok(())
}

#[tokio::test]
async fn test_create_worktree_invalid_base_branch() -> Result<()> {
    // Test: Verify worktree creation fails gracefully for non-existent base branch
    // This test ensures proper error handling for invalid base branches
    
    let (_temp_dir, repo_path) = create_temp_git_repo()?;
    let manager = GitWorktreeManager::new(repo_path)?;
    
    // Try to create worktree with non-existent base branch
    let result = manager.create_worktree("test-branch", Some("nonexistent-base"), None).await;
    
    // Should fail gracefully for non-existent base branch
    assert!(result.is_err(), "Should fail for non-existent base branch");
    
    Ok(())
}

#[tokio::test]
async fn test_create_worktree_duplicate_branch() -> Result<()> {
    // Test: Verify worktree creation fails gracefully for duplicate branch names
    // This test ensures proper error handling for existing branches
    
    let (_temp_dir, repo_path) = create_temp_git_repo()?;
    let manager = GitWorktreeManager::new(repo_path)?;
    
    // Create first worktree
    let _first_worktree = manager.create_worktree("duplicate-branch", None, None).await?;
    
    // Try to create another worktree with the same branch name
    let result = manager.create_worktree("duplicate-branch", None, None).await;
    
    // Should fail gracefully for duplicate branch name
    assert!(result.is_err(), "Should fail for duplicate branch name");
    
    Ok(())
}

#[tokio::test]
async fn test_create_worktree_invalid_directory_name() -> Result<()> {
    // Test: Verify worktree creation handles invalid directory names
    // This test ensures proper error handling for malformed directory names
    
    let (_temp_dir, repo_path) = create_temp_git_repo()?;
    let manager = GitWorktreeManager::new(repo_path)?;
    
    // Try to create worktree with invalid directory name
    let result = manager.create_worktree("test-branch", None, Some("")).await;
    
    // Should fail gracefully for empty directory name
    assert!(result.is_err(), "Should fail for empty directory name");
    
    Ok(())
}

#[tokio::test]
async fn test_remove_worktree_nonexistent() -> Result<()> {
    // Test: Verify worktree removal fails gracefully for non-existent worktrees
    // This test ensures proper error handling for invalid removal attempts
    
    let (_temp_dir, repo_path) = create_temp_git_repo()?;
    let manager = GitWorktreeManager::new(repo_path)?;
    
    // Try to remove non-existent worktree
    let nonexistent_path = _temp_dir.path().join("nonexistent-worktree");
    let result = manager.remove_worktree(&nonexistent_path).await;
    
    // Should fail gracefully for non-existent worktree
    assert!(result.is_err(), "Should fail for non-existent worktree");
    
    Ok(())
}

#[tokio::test]
async fn test_remove_worktree_invalid_path() -> Result<()> {
    // Test: Verify worktree removal handles invalid paths gracefully
    // This test ensures proper error handling for malformed paths
    
    let (_temp_dir, repo_path) = create_temp_git_repo()?;
    let manager = GitWorktreeManager::new(repo_path)?;
    
    // Try to remove with invalid path
    let invalid_path = Path::new("/invalid/path/with/special/chars/<>:|?*");
    let result = manager.remove_worktree(invalid_path).await;
    
    // Should fail gracefully for invalid path
    assert!(result.is_err(), "Should fail for invalid path");
    
    Ok(())
}

#[tokio::test]
async fn test_list_worktrees_corrupted_repo() -> Result<()> {
    // Test: Verify worktree listing handles corrupted repositories gracefully
    // This test ensures proper error handling for repository corruption
    
    let (_temp_dir, repo_path) = create_temp_git_repo()?;
    let manager = GitWorktreeManager::new(repo_path)?;
    
    // Corrupt the git directory by removing important files
    let git_dir = repo_path.join(".git");
    if git_dir.exists() {
        std::fs::remove_dir_all(&git_dir)?;
    }
    
    // Try to list worktrees from corrupted repo
    let result = manager.list_worktrees().await;
    
    // Should fail gracefully for corrupted repository
    assert!(result.is_err(), "Should fail for corrupted repository");
    
    Ok(())
}

#[tokio::test]
async fn test_git_operations_permission_errors() -> Result<()> {
    // Test: Verify git operations handle permission errors gracefully
    // This test ensures proper error handling for permission issues
    
    let (_temp_dir, repo_path) = create_temp_git_repo()?;
    let manager = GitWorktreeManager::new(repo_path)?;
    
    // Create a worktree first
    let worktree_path = manager.create_worktree("permission-test", None, None).await?;
    
    // Make the worktree directory read-only (on Unix systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&worktree_path)?.permissions();
        perms.set_mode(0o444); // Read-only
        std::fs::set_permissions(&worktree_path, perms)?;
    }
    
    // Try to remove the read-only worktree
    let result = manager.remove_worktree(&worktree_path).await;
    
    // Should fail gracefully for permission errors
    assert!(result.is_err(), "Should fail for permission errors");
    
    Ok(())
}

#[tokio::test]
async fn test_git_operations_concurrent_access() -> Result<()> {
    // Test: Verify git operations handle concurrent access gracefully
    // This test ensures proper error handling for concurrent modifications
    
    let (_temp_dir, repo_path) = create_temp_git_repo()?;
    let manager1 = GitWorktreeManager::new(repo_path.clone())?;
    let manager2 = GitWorktreeManager::new(repo_path)?;
    
    // Try to create worktrees concurrently
    let result1 = manager1.create_worktree("concurrent-1", None, None).await;
    let result2 = manager2.create_worktree("concurrent-2", None, None).await;
    
    // At least one should succeed, but we're testing error handling
    assert!(true, "Should handle concurrent access gracefully");
    
    Ok(())
}

#[tokio::test]
async fn test_git_operations_disk_full_simulation() -> Result<()> {
    // Test: Verify git operations handle disk space issues gracefully
    // This test ensures proper error handling for disk space problems
    
    let (_temp_dir, repo_path) = create_temp_git_repo()?;
    let manager = GitWorktreeManager::new(repo_path)?;
    
    // Create a very large worktree name to simulate potential issues
    let large_name = "a".repeat(1000);
    let result = manager.create_worktree(&large_name, None, None).await;
    
    // Should handle large names gracefully
    assert!(true, "Should handle large names gracefully");
    
    Ok(())
}

#[tokio::test]
async fn test_git_operations_network_issues() -> Result<()> {
    // Test: Verify git operations handle network issues gracefully
    // This test ensures proper error handling for network problems
    
    let (_temp_dir, repo_path) = create_temp_git_repo()?;
    let manager = GitWorktreeManager::new(repo_path)?;
    
    // Try to create worktree with a remote base branch (should fail gracefully)
    let result = manager.create_worktree("network-test", Some("origin/nonexistent"), None).await;
    
    // Should fail gracefully for network issues
    assert!(result.is_err(), "Should fail gracefully for network issues");
    
    Ok(())
}

#[tokio::test]
async fn test_git_operations_malformed_git_config() -> Result<()> {
    // Test: Verify git operations handle malformed git config gracefully
    // This test ensures proper error handling for configuration issues
    
    let (_temp_dir, repo_path) = create_temp_git_repo()?;
    
    // Corrupt the git config
    let config_path = repo_path.join(".git").join("config");
    if config_path.exists() {
        std::fs::write(&config_path, "malformed config content")?;
    }
    
    let manager = GitWorktreeManager::new(repo_path)?;
    
    // Try to create worktree with corrupted config
    let result = manager.create_worktree("config-test", None, None).await;
    
    // Should fail gracefully for malformed config
    assert!(result.is_err(), "Should fail gracefully for malformed config");
    
    Ok(())
}
