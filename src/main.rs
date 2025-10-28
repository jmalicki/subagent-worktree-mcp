use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

use crate::git_operations::GitWorktreeManager;
use crate::subagent_spawner::{SubagentSpawner, CursorCliAgent, AgentOptions};

/// Configuration for spawning a subagent
#[derive(Debug, Serialize, Deserialize)]
pub struct SubagentConfig {
    /// Name of the branch to create for the subagent
    pub branch_name: String,
    /// Optional base branch to create from (defaults to current branch)
    pub base_branch: Option<String>,
    /// Prompt to give to the new subagent
    pub prompt: String,
    /// Optional directory name for the worktree (defaults to branch_name)
    pub worktree_dir: Option<String>,
    
    /// Agent type to spawn (defaults to "cursor-cli")
    pub agent_type: Option<String>,
    
    /// Agent-specific options
    pub agent_options: Option<AgentOptions>,
}

/// Configuration for cleaning up a worktree and its agents
#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Name of the worktree/branch to clean up
    pub worktree_name: String,
    /// Whether to force cleanup even if agents are still running
    pub force: bool,
    /// Whether to remove the git branch after cleanup
    pub remove_branch: bool,
    /// Whether to kill running agents before cleanup
    pub kill_agents: bool,
}

/// Main MCP server implementation
pub struct SubagentWorktreeServer {
    git_manager: GitWorktreeManager,
    spawner: SubagentSpawner,
}

impl SubagentWorktreeServer {
    /// Create a new server instance
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        let git_manager = GitWorktreeManager::new(repo_path)?;
        let mut spawner = SubagentSpawner::new()?;
        
        // Register default agents
        spawner.register_agent(Box::new(CursorCliAgent));
        
        Ok(Self {
            git_manager,
            spawner,
        })
    }

    /// Handle the spawn_subagent tool call
    async fn handle_spawn_subagent(&self, config: SubagentConfig) -> Result<String> {
        info!("Spawning subagent with config: {:?}", config);
        
        // Validate that we're in a git repository
        if !self.git_manager.is_git_repo() {
            return Err(anyhow::anyhow!("Not in a git repository"));
        }

        // Create the worktree
        let worktree_path = self.git_manager.create_worktree(
            &config.branch_name,
            config.base_branch.as_deref(),
            config.worktree_dir.as_deref(),
        ).await?;

        info!("Created worktree at: {}", worktree_path.display());

        // Determine agent type and options
        let agent_type = config.agent_type.unwrap_or_else(|| "cursor-cli".to_string());
        let agent_options = config.agent_options.unwrap_or_default();
        
        // Spawn the specified agent in the new worktree directory
        self.spawner.spawn_agent(&agent_type, &worktree_path, &config.prompt, &agent_options).await?;

        Ok(format!(
            "Successfully spawned subagent in worktree '{}' at {}",
            config.branch_name,
            worktree_path.display()
        ))
    }

    /// Handle the cleanup_worktree tool call
    async fn handle_cleanup_worktree(&self, config: CleanupConfig) -> Result<String> {
        info!("Cleaning up worktree with config: {:?}", config);
        
        // Validate that we're in a git repository
        if !self.git_manager.is_git_repo() {
            return Err(anyhow::anyhow!("Not in a git repository"));
        }

        // Find the worktree path
        let repo_path = std::env::current_dir()?;
        let worktree_path = repo_path.parent()
            .context("Repository has no parent directory")?
            .join(&config.worktree_name);

        if !worktree_path.exists() {
            return Err(anyhow::anyhow!("Worktree '{}' does not exist", config.worktree_name));
        }

        // Kill running agents if requested
        if config.kill_agents {
            self.kill_agents_in_worktree(&worktree_path, config.force).await?;
        }

        // Remove the worktree
        self.git_manager.remove_worktree(&worktree_path).await?;

        // Remove the branch if requested
        if config.remove_branch {
            self.remove_branch(&config.worktree_name).await?;
        }

        Ok(format!(
            "Successfully cleaned up worktree '{}'{}",
            config.worktree_name,
            if config.remove_branch { " and removed branch" } else { "" }
        ))
    }

    /// Kill agents running in a specific worktree
    async fn kill_agents_in_worktree(&self, worktree_path: &std::path::Path, force: bool) -> Result<()> {
        use crate::agent_monitor::{AgentMonitor, AgentMonitorConfig};
        
        let mut monitor = AgentMonitor::new(std::env::current_dir()?);
        let config = AgentMonitorConfig {
            only_our_agents: true,
            only_waiting_agents: false,
            agent_types: None,
            worktree_paths: Some(vec![worktree_path.to_string_lossy().to_string()]),
        };

        let agents = monitor.get_running_agents(&config).await?;
        
        for agent in agents {
            info!("Killing agent {} (PID: {}) in worktree {}", 
                  agent.name, agent.pid, worktree_path.display());
            
            let success = monitor.kill_agent(agent.pid, force).await?;
            if !success {
                warn!("Failed to kill agent {} (PID: {})", agent.name, agent.pid);
            }
        }

        Ok(())
    }

    /// Remove a git branch
    async fn remove_branch(&self, branch_name: &str) -> Result<()> {
        use tokio::task;
        use git2::{Repository, BranchType};
        
        let repo_path = std::env::current_dir()?;
        let branch_name = branch_name.to_string(); // Convert to owned string
        
        task::spawn_blocking(move || {
            let repo = Repository::open(&repo_path)
                .context("Failed to open git repository")?;
            
            // Find the branch
            let mut branch = repo.find_branch(&branch_name, BranchType::Local)
                .context(format!("Branch '{}' not found", branch_name))?;
            
            // Delete the branch
            branch.delete()
                .context(format!("Failed to delete branch '{}'", branch_name))?;
            
            info!("Successfully deleted branch '{}'", branch_name);
            Ok(())
        }).await.context("Failed to spawn blocking task")?
    }

    /// List all worktrees and their status
    async fn list_worktrees(&self) -> Result<String> {
        info!("Listing all worktrees");
        
        // Validate that we're in a git repository
        if !self.git_manager.is_git_repo() {
            return Err(anyhow::anyhow!("Not in a git repository"));
        }

        let worktrees = self.git_manager.list_worktrees().await?;
        
        if worktrees.is_empty() {
            return Ok("No worktrees found".to_string());
        }

        let mut result = String::from("Worktrees:\n");
        for worktree in worktrees {
            result.push_str(&format!(
                "  - Path: {}\n    Branch: {}\n    Commit: {}\n\n",
                worktree.path.display(),
                worktree.branch.as_deref().unwrap_or("unknown"),
                worktree.commit.as_deref().unwrap_or("unknown")
            ));
        }

        Ok(result)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("subagent_worktree_mcp=info")
        .init();

    // Get the current working directory as the repository path
    let repo_path = std::env::current_dir()?;
    info!("Starting MCP server for repository: {}", repo_path.display());

    // Create the server
    let server = SubagentWorktreeServer::new(repo_path)?;

    // For now, just demonstrate the functionality
    let config = SubagentConfig {
        branch_name: "demo-branch".to_string(),
        base_branch: None,
        prompt: "Hello, this is a demo subagent!".to_string(),
        worktree_dir: None,
        agent_type: Some("cursor-cli".to_string()),
        agent_options: Some(AgentOptions::default()),
    };

    let result = server.handle_spawn_subagent(config).await?;
    info!("Result: {}", result);

    // Demonstrate cleanup tool (commented out for safety)
    /*
    let cleanup_config = CleanupConfig {
        worktree_name: "demo-branch".to_string(),
        force: false,
        remove_branch: true,
        kill_agents: true,
    };
    
    let cleanup_result = server.handle_cleanup_worktree(cleanup_config).await?;
    info!("Cleanup result: {}", cleanup_result);
    */

    // TODO: When implementing full MCP server, register these tools:
    // 1. spawn_subagent - Creates worktree and spawns agent
    // 2. monitor_agents - Lists running agents in worktrees  
    // 3. cleanup_worktree - DESTRUCTIVE: Kills agents and removes worktree
    // 4. list_worktrees - Shows all worktrees and their status
    
    info!("MCP server ready with tools: spawn_subagent, monitor_agents, cleanup_worktree, list_worktrees");

    Ok(())
}
