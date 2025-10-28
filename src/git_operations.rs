use anyhow::{Context, Result};
use git2::{BranchType, Repository};
use std::path::{Path, PathBuf};
use tokio::task;
use tracing::{debug, info, warn};

/// Manages git worktree operations for subagent spawning
#[derive(Clone)]
pub struct GitWorktreeManager {
    repo_path: PathBuf,
}

impl GitWorktreeManager {
    /// Create a new GitWorktreeManager for the given repository path
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        // Validate that the path is a git repository
        if !Self::is_git_repo_path(&repo_path) {
            return Err(anyhow::anyhow!(
                "Path is not a git repository: {}",
                repo_path.display()
            ));
        }

        Ok(Self { repo_path })
    }

    /// Check if the current directory is a git repository
    pub fn is_git_repo(&self) -> bool {
        Self::is_git_repo_path(&self.repo_path)
    }

    /// Check if a given path is a git repository
    fn is_git_repo_path(path: &Path) -> bool {
        path.join(".git").exists() || Repository::open(path).is_ok()
    }

    /// Create a new worktree for the subagent
    ///
    /// # Arguments
    /// * `branch_name` - Name of the branch to create
    /// * `base_branch` - Optional base branch to create from (defaults to current branch)
    /// * `worktree_dir` - Optional directory name for the worktree (defaults to branch_name)
    ///
    /// # Returns
    /// Path to the created worktree directory
    pub async fn create_worktree(
        &self,
        branch_name: &str,
        base_branch: Option<&str>,
        worktree_dir: Option<&str>,
    ) -> Result<PathBuf> {
        let repo_path = self.repo_path.clone();
        let branch_name = branch_name.to_string();
        let base_branch = base_branch.map(|s| s.to_string());
        let worktree_dir = worktree_dir.map(|s| s.to_string());

        // Run git operations in a blocking task to avoid blocking the async runtime
        task::spawn_blocking(move || {
            Self::create_worktree_blocking(
                &repo_path,
                &branch_name,
                base_branch.as_deref(),
                worktree_dir.as_deref(),
            )
        })
        .await
        .context("Failed to spawn blocking task")?
    }

    /// Blocking implementation of worktree creation
    fn create_worktree_blocking(
        repo_path: &Path,
        branch_name: &str,
        base_branch: Option<&str>,
        worktree_dir: Option<&str>,
    ) -> Result<PathBuf> {
        // Open the git repository
        let repo = Repository::open(repo_path).context("Failed to open git repository")?;

        debug!("Opened repository at: {}", repo_path.display());

        // Determine the base branch
        let base_branch_name = match base_branch {
            Some(branch) => branch.to_string(),
            None => {
                // Get current branch
                let head = repo.head().context("Failed to get HEAD reference")?;

                if let Some(name) = head.shorthand() {
                    name.to_string()
                } else {
                    return Err(anyhow::anyhow!("Could not determine current branch name"));
                }
            }
        };

        info!(
            "Creating branch '{}' from base branch '{}'",
            branch_name, base_branch_name
        );

        // Check if branch already exists
        if Self::branch_exists(&repo, branch_name)? {
            warn!(
                "Branch '{}' already exists, checking it out instead",
                branch_name
            );

            // If branch exists, just check it out
            let branch_ref = repo
                .find_branch(branch_name, BranchType::Local)
                .context("Failed to find existing branch")?;

            let commit = branch_ref
                .get()
                .peel_to_commit()
                .context("Failed to get commit from branch")?;

            repo.checkout_tree(&commit.into_object(), None)
                .context("Failed to checkout existing branch")?;

            repo.set_head(&format!("refs/heads/{}", branch_name))
                .context("Failed to set HEAD to existing branch")?;
        } else {
            // Create new branch from base branch
            let base_commit = Self::get_branch_commit(&repo, &base_branch_name)?;

            let _branch_ref = repo
                .branch(branch_name, &base_commit, false)
                .context("Failed to create new branch")?;

            // Checkout the new branch
            repo.checkout_tree(&base_commit.into_object(), None)
                .context("Failed to checkout new branch")?;

            repo.set_head(&format!("refs/heads/{}", branch_name))
                .context("Failed to set HEAD to new branch")?;
        }

        // Determine worktree directory name
        let worktree_dir_name = worktree_dir.unwrap_or(branch_name);

        // Create worktree directory path (adjacent to the main repository)
        let worktree_path = repo_path
            .parent()
            .context("Repository has no parent directory")?
            .join(worktree_dir_name);

        // Check if worktree directory already exists
        if worktree_path.exists() {
            warn!(
                "Worktree directory already exists: {}",
                worktree_path.display()
            );
            return Ok(worktree_path);
        }

        // Create the worktree using git command (more reliable than libgit2 for worktrees)
        let output = std::process::Command::new("git")
            .arg("worktree")
            .arg("add")
            .arg(&worktree_path)
            .arg(branch_name)
            .current_dir(repo_path)
            .output()
            .context("Failed to execute git worktree add command")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git worktree add failed: {}", error_msg));
        }

        info!(
            "Successfully created worktree at: {}",
            worktree_path.display()
        );
        Ok(worktree_path)
    }

    /// Check if a branch exists in the repository
    fn branch_exists(repo: &Repository, branch_name: &str) -> Result<bool> {
        match repo.find_branch(branch_name, BranchType::Local) {
            Ok(_) => Ok(true),
            Err(ref e) if e.code() == git2::ErrorCode::NotFound => Ok(false),
            Err(e) => Err(anyhow::anyhow!("Error checking branch existence: {}", e)),
        }
    }

    /// Get the commit for a given branch name
    fn get_branch_commit<'a>(repo: &'a Repository, branch_name: &str) -> Result<git2::Commit<'a>> {
        // Try to find the branch reference
        let branch_ref = repo
            .find_branch(branch_name, BranchType::Local)
            .or_else(|_| repo.find_branch(branch_name, BranchType::Remote))
            .context(format!("Branch '{}' not found", branch_name))?;

        // Get the commit from the branch
        let commit = branch_ref.get().peel_to_commit().context(format!(
            "Failed to get commit from branch '{}'",
            branch_name
        ))?;

        Ok(commit)
    }

    /// List all existing worktrees
    pub async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        let repo_path = self.repo_path.clone();

        task::spawn_blocking(move || Self::list_worktrees_blocking(&repo_path))
            .await
            .context("Failed to spawn blocking task")?
    }

    /// Blocking implementation of listing worktrees
    fn list_worktrees_blocking(repo_path: &Path) -> Result<Vec<WorktreeInfo>> {
        let output = std::process::Command::new("git")
            .arg("worktree")
            .arg("list")
            .arg("--porcelain")
            .current_dir(repo_path)
            .output()
            .context("Failed to execute git worktree list command")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git worktree list failed: {}", error_msg));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut worktrees = Vec::new();
        let mut current_worktree = None;

        for line in output_str.lines() {
            if line.starts_with("worktree ") {
                // Save previous worktree if exists
                if let Some(worktree) = current_worktree.take() {
                    worktrees.push(worktree);
                }

                // Start new worktree
                let path = line.strip_prefix("worktree ").unwrap_or("");
                current_worktree = Some(WorktreeInfo {
                    path: PathBuf::from(path),
                    branch: None,
                    commit: None,
                });
            } else if line.starts_with("HEAD ") {
                if let Some(ref mut worktree) = current_worktree {
                    worktree.commit = Some(line.strip_prefix("HEAD ").unwrap_or("").to_string());
                }
            } else if line.starts_with("branch refs/heads/")
                && let Some(ref mut worktree) = current_worktree {
                    worktree.branch = Some(
                        line.strip_prefix("branch refs/heads/")
                            .unwrap_or("")
                            .to_string(),
                    );
                }
        }

        // Add the last worktree
        if let Some(worktree) = current_worktree {
            worktrees.push(worktree);
        }

        Ok(worktrees)
    }

    /// Remove a worktree
    pub async fn remove_worktree(&self, worktree_path: &Path) -> Result<()> {
        let repo_path = self.repo_path.clone();
        let worktree_path = worktree_path.to_path_buf();

        task::spawn_blocking(move || Self::remove_worktree_blocking(&repo_path, &worktree_path))
            .await
            .context("Failed to spawn blocking task")?
    }

    /// Blocking implementation of removing worktrees
    fn remove_worktree_blocking(repo_path: &Path, worktree_path: &Path) -> Result<()> {
        let output = std::process::Command::new("git")
            .arg("worktree")
            .arg("remove")
            .arg(worktree_path)
            .current_dir(repo_path)
            .output()
            .context("Failed to execute git worktree remove command")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git worktree remove failed: {}", error_msg));
        }

        info!("Successfully removed worktree: {}", worktree_path.display());
        Ok(())
    }
}

/// Information about a git worktree
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub commit: Option<String>,
}
