//! Subagent Worktree MCP Server
//! 
//! A Model Context Protocol (MCP) server for spawning subagents with git worktrees.
//! This library provides functionality for creating isolated development environments
//! for AI agents using git worktrees and managing their lifecycle.

pub mod agent_monitor;
pub mod git_operations;
pub mod subagent_spawner;
pub mod doc_generator;

// Re-export main types for easier use
pub use agent_monitor::{AgentMonitor, AgentMonitorConfig, AgentProcessInfo, AgentSummary};
pub use git_operations::{GitWorktreeManager, WorktreeInfo};
pub use subagent_spawner::{AgentSpawner, AgentOptions, AgentInfo, SubagentSpawner, CursorCliAgent};
pub use doc_generator::DocGenerator;

// MCP Server implementation
use anyhow::Result;
use rmcp::tool_router;
use rmcp::tool;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::tool::ToolRouter;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

/// Configuration for spawning a subagent
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SubagentConfig {
    /// Name of the branch to create for the subagent
    pub branch_name: String,
    /// Initial prompt to give to the subagent
    pub prompt: String,
    /// Optional directory name for the worktree (defaults to branch name)
    pub worktree_dir: Option<String>,
    /// Agent type to spawn (defaults to "cursor-agent")
    pub agent_type: Option<String>,
    /// Additional options for the agent
    pub agent_options: Option<AgentOptions>,
}

/// Configuration for cleaning up a worktree
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CleanupConfig {
    /// Path to the worktree to clean up
    pub worktree_path: String,
    /// Whether to delete the branch (default: false)
    pub delete_branch: Option<bool>,
    /// Whether to force cleanup even if there are uncommitted changes (default: false)
    pub force: Option<bool>,
}

/// Configuration for listing worktrees
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListWorktreesConfig {
    /// Whether to include agent information (default: true)
    pub include_agents: Option<bool>,
    /// Only show agents spawned by our system (default: true)
    pub only_our_agents: Option<bool>,
    /// Only show agents waiting for input (default: false)
    pub only_waiting_agents: Option<bool>,
}

/// Main MCP server for subagent worktree management
pub struct SubagentWorktreeServer {
    git_manager: GitWorktreeManager,
    spawner: SubagentSpawner,
    tool_router: ToolRouter<Self>,
}

impl SubagentWorktreeServer {
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        let git_manager = GitWorktreeManager::new(repo_path)?;
        let spawner = SubagentSpawner::new()?;
        
        Ok(Self {
            git_manager,
            spawner,
            tool_router: Self::tool_router(),
        })
    }

    async fn handle_spawn_subagent(&self, config: SubagentConfig) -> Result<String> {
        let worktree_dir = config.worktree_dir.unwrap_or_else(|| config.branch_name.clone());
        let agent_type = config.agent_type.unwrap_or_else(|| "cursor-agent".to_string());
        
        info!("Spawning subagent: branch={}, worktree={}, agent={}", 
              config.branch_name, worktree_dir, agent_type);

        // Create the worktree
        let worktree_path = self.git_manager.create_worktree(
            &config.branch_name,
            None, // No base branch specified
            Some(&worktree_dir),
        ).await?;

        // Spawn the agent
        let options = config.agent_options.unwrap_or_default();
        self.spawner.spawn_agent(
            &agent_type,
            &worktree_path,
            &config.prompt,
            &options,
        ).await?;

        Ok(format!("Successfully spawned {} subagent in worktree: {}", 
                   agent_type, worktree_path.display()))
    }

    async fn handle_cleanup_worktree(&self, config: CleanupConfig) -> Result<String> {
        let worktree_path = PathBuf::from(&config.worktree_path);
        let delete_branch = config.delete_branch.unwrap_or(false);
        let force = config.force.unwrap_or(false);

        info!("Cleaning up worktree: {}, delete_branch={}, force={}", 
              worktree_path.display(), delete_branch, force);

        // Kill any agents running in this worktree
        self.kill_agents_in_worktree(&worktree_path).await?;

        // Remove the worktree
        self.git_manager.remove_worktree(&worktree_path).await?;

        let mut result = format!("Successfully cleaned up worktree: {}", worktree_path.display());

        // Optionally delete the branch
        if delete_branch
            && let Some(branch_name) = worktree_path.file_name()
                .and_then(|name| name.to_str()) {
            self.remove_branch(branch_name).await?;
            result.push_str(&format!(" and deleted branch: {}", branch_name));
        }

        Ok(result)
    }

    async fn handle_list_worktrees(&self, config: ListWorktreesConfig) -> Result<String> {
        let include_agents = config.include_agents.unwrap_or(true);
        let only_our_agents = config.only_our_agents.unwrap_or(true);
        let only_waiting_agents = config.only_waiting_agents.unwrap_or(false);

        info!("Listing worktrees: include_agents={}, only_our_agents={}, only_waiting_agents={}", 
              include_agents, only_our_agents, only_waiting_agents);

        let worktrees = self.git_manager.list_worktrees().await?;
        
        if !include_agents {
            let worktree_info: Vec<String> = worktrees.iter()
                .map(|wt| format!("- {} (branch: {})", wt.path.display(), wt.branch.as_deref().unwrap_or("unknown")))
                .collect();
            return Ok(worktree_info.join("\n"));
        }

        // TODO: Implement agent monitoring integration
        // For now, just return worktree information
        let worktree_info: Vec<String> = worktrees.iter()
            .map(|wt| format!("- {} (branch: {}) - No agent info available", wt.path.display(), wt.branch.as_deref().unwrap_or("unknown")))
            .collect();
        
        Ok(worktree_info.join("\n"))
    }

    async fn kill_agents_in_worktree(&self, worktree_path: &std::path::Path) -> Result<()> {
        // TODO: Implement agent process killing
        // This would use the agent monitor to find and kill processes
        info!("Killing agents in worktree: {}", worktree_path.display());
        Ok(())
    }

    async fn remove_branch(&self, branch_name: &str) -> Result<()> {
        // TODO: Implement branch deletion
        info!("Removing branch: {}", branch_name);
        Ok(())
    }
}

#[tool_router]
impl SubagentWorktreeServer {
    /// Spawn a new subagent with a git worktree for isolated development
    #[tool(description = "Spawn a new subagent with a git worktree for isolated development")]
    async fn spawn_subagent(&self, params: Parameters<SubagentConfig>) -> Result<String, String> {
        match self.handle_spawn_subagent(params.0).await {
            Ok(result) => Ok(result),
            Err(e) => Err(format!("Failed to spawn subagent: {}", e)),
        }
    }

    /// Clean up a worktree and optionally delete the branch (destructive)
    #[tool(description = "Clean up a worktree and optionally delete the branch (destructive)")]
    async fn cleanup_worktree(&self, params: Parameters<CleanupConfig>) -> Result<String, String> {
        match self.handle_cleanup_worktree(params.0).await {
            Ok(result) => Ok(result),
            Err(e) => Err(format!("Failed to cleanup worktree: {}", e)),
        }
    }

    /// List all git worktrees and their associated agents
    #[tool(description = "List all git worktrees and their associated agents")]
    async fn list_worktrees(&self, params: Parameters<ListWorktreesConfig>) -> Result<String, String> {
        match self.handle_list_worktrees(params.0).await {
            Ok(result) => Ok(result),
            Err(e) => Err(format!("Failed to list worktrees: {}", e)),
        }
    }
}

/// Run the MCP server
pub async fn run_server() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let repo_path = std::env::current_dir()?;
    info!("Starting MCP server for repository: {}", repo_path.display());
    
    let _server = SubagentWorktreeServer::new(repo_path)?;
    
    info!("MCP server started with tools:");
    info!("  - spawn_subagent: Spawn a new subagent with a git worktree");
    info!("  - cleanup_worktree: Clean up a worktree and optionally delete the branch");
    info!("  - list_worktrees: List all git worktrees and their associated agents");
    
    // TODO: Implement proper MCP server serving
    // For now, just keep the server running
    tokio::signal::ctrl_c().await?;
    info!("MCP server shutting down");
    
    Ok(())
}