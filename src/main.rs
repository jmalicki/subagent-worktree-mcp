use anyhow::{Context, Result};
use rmcp::server::{McpServer, RequestHandler};
use rmcp::types::{
    CallToolRequest, CallToolResult, ListToolsRequest, ListToolsResult, 
    Tool, ToolInputSchema, TextContent
};
use rmcp_macros::mcp_tool;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
    
    /// Agent type to spawn (defaults to "cursor-agent")
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
        let agent_type = config.agent_type.unwrap_or_else(|| "cursor-agent".to_string());
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
        info!("Killing agents in worktree: {}", worktree_path.display());
        
        // TODO: Implement actual agent killing logic
        // This would use the agent monitor to find and kill processes
        // running in the specified worktree directory
        
        if force {
            warn!("Force killing agents in worktree: {}", worktree_path.display());
        }
        
        Ok(())
    }

    /// Remove a git branch
    async fn remove_branch(&self, branch_name: &str) -> Result<()> {
        info!("Removing branch: {}", branch_name);
        
        // Convert to owned String for move into spawn_blocking
        let branch_name = branch_name.to_string();
        
        tokio::task::spawn_blocking(move || {
            let repo = git2::Repository::open(".")?;
            let mut branch = repo.find_branch(&branch_name, git2::BranchType::Local)?;
            branch.delete()?;
            Ok::<(), git2::Error>(())
        }).await??;
        
        Ok(())
    }

    /// List all worktrees
    async fn list_worktrees(&self) -> Result<String> {
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

// MCP Tool definitions using the official SDK macros
#[mcp_tool(
    name = "spawn_subagent",
    description = "Spawn a new subagent with a git worktree for isolated development"
)]
async fn spawn_subagent(config: SubagentConfig) -> Result<String> {
    // This would be called by the MCP server
    // For now, we'll implement this in the RequestHandler
    Err(anyhow::anyhow!("Tool not yet implemented with macro"))
}

#[mcp_tool(
    name = "monitor_agents", 
    description = "Monitor running agent processes and their status"
)]
async fn monitor_agents(
    only_our_agents: Option<bool>,
    only_waiting_agents: Option<bool>,
    agent_types: Option<Vec<String>>,
    worktree_paths: Option<Vec<String>>
) -> Result<String> {
    // TODO: Implement agent monitoring
    Ok("Agent monitoring not yet implemented".to_string())
}

#[mcp_tool(
    name = "cleanup_worktree",
    description = "⚠️ DESTRUCTIVE: Clean up a worktree and optionally kill running agents and remove the branch"
)]
async fn cleanup_worktree(
    worktree_name: String,
    force: Option<bool>,
    remove_branch: Option<bool>,
    kill_agents: Option<bool>
) -> Result<String> {
    let config = CleanupConfig {
        worktree_name,
        force: force.unwrap_or(false),
        remove_branch: remove_branch.unwrap_or(false),
        kill_agents: kill_agents.unwrap_or(false),
    };
    
    // This would be called by the MCP server
    // For now, we'll implement this in the RequestHandler
    Err(anyhow::anyhow!("Tool not yet implemented with macro"))
}

#[mcp_tool(
    name = "list_worktrees",
    description = "List all worktrees and their current status"
)]
async fn list_worktrees() -> Result<String> {
    // This would be called by the MCP server
    // For now, we'll implement this in the RequestHandler
    Err(anyhow::anyhow!("Tool not yet implemented with macro"))
}

impl RequestHandler for SubagentWorktreeServer {
    async fn list_tools(&self, _request: ListToolsRequest) -> Result<ListToolsResult> {
        // The macros should automatically generate this, but for now we'll manually define
        Ok(ListToolsResult {
            tools: vec![
                Tool {
                    name: "spawn_subagent".to_string(),
                    description: Some("Spawn a new subagent with a git worktree for isolated development".to_string()),
                    input_schema: ToolInputSchema::JsonSchema(json!({
                        "type": "object",
                        "properties": {
                            "branch_name": {
                                "type": "string",
                                "description": "Name of the branch to create for the subagent"
                            },
                            "prompt": {
                                "type": "string", 
                                "description": "Initial prompt to give to the new subagent"
                            },
                            "base_branch": {
                                "type": "string",
                                "description": "Base branch to create from (optional, defaults to current branch)"
                            },
                            "worktree_dir": {
                                "type": "string",
                                "description": "Custom worktree directory name (optional, defaults to branch_name)"
                            },
                            "agent_type": {
                                "type": "string",
                                "description": "Type of agent to spawn (optional, defaults to 'cursor-agent')",
                                "enum": ["cursor-agent", "vscode", "vim", "neovim"]
                            },
                            "agent_options": {
                                "type": "object",
                                "description": "Agent-specific options",
                                "properties": {
                                    "new_window": {
                                        "type": "boolean",
                                        "description": "Open in new window"
                                    },
                                    "wait": {
                                        "type": "boolean", 
                                        "description": "Wait for process completion"
                                    },
                                    "detach": {
                                        "type": "boolean",
                                        "description": "Detach process"
                                    },
                                    "custom_options": {
                                        "type": "object",
                                        "description": "Custom options as key-value pairs"
                                    }
                                }
                            }
                        },
                        "required": ["branch_name", "prompt"]
                    }))
                },
                Tool {
                    name: "monitor_agents".to_string(),
                    description: Some("Monitor running agent processes and their status".to_string()),
                    input_schema: ToolInputSchema::JsonSchema(json!({
                        "type": "object",
                        "properties": {
                            "only_our_agents": {
                                "type": "boolean",
                                "description": "Only show agents we spawned (optional)"
                            },
                            "only_waiting_agents": {
                                "type": "boolean",
                                "description": "Only show agents waiting for input (optional)"
                            },
                            "agent_types": {
                                "type": "array",
                                "items": {"type": "string"},
                                "description": "Filter by agent types (optional)"
                            },
                            "worktree_paths": {
                                "type": "array", 
                                "items": {"type": "string"},
                                "description": "Filter by worktree paths (optional)"
                            }
                        }
                    }))
                },
                Tool {
                    name: "cleanup_worktree".to_string(),
                    description: Some("⚠️ DESTRUCTIVE: Clean up a worktree and optionally kill running agents and remove the branch".to_string()),
                    input_schema: ToolInputSchema::JsonSchema(json!({
                        "type": "object",
                        "properties": {
                            "worktree_name": {
                                "type": "string",
                                "description": "Name of the worktree/branch to clean up"
                            },
                            "force": {
                                "type": "boolean",
                                "description": "Force cleanup even if agents are still running (optional)"
                            },
                            "remove_branch": {
                                "type": "boolean",
                                "description": "Remove the git branch after cleanup (optional)"
                            },
                            "kill_agents": {
                                "type": "boolean",
                                "description": "Kill running agents before cleanup (optional)"
                            }
                        },
                        "required": ["worktree_name"]
                    }))
                },
                Tool {
                    name: "list_worktrees".to_string(),
                    description: Some("List all worktrees and their current status".to_string()),
                    input_schema: ToolInputSchema::JsonSchema(json!({
                        "type": "object",
                        "properties": {},
                        "description": "No parameters required"
                    }))
                }
            ],
        })
    }

    async fn call_tool(&self, request: CallToolRequest) -> Result<CallToolResult> {
        match request.name.as_str() {
            "spawn_subagent" => {
                let config: SubagentConfig = serde_json::from_value(request.arguments)?;
                let result = self.handle_spawn_subagent(config).await?;
                Ok(CallToolResult {
                    content: vec![TextContent {
                        text: result,
                        r#type: "text".to_string(),
                    }],
                    is_error: false,
                })
            }
            "monitor_agents" => {
                // TODO: Implement agent monitoring
                Ok(CallToolResult {
                    content: vec![TextContent {
                        text: "Agent monitoring not yet implemented".to_string(),
                        r#type: "text".to_string(),
                    }],
                    is_error: false,
                })
            }
            "cleanup_worktree" => {
                let config: CleanupConfig = serde_json::from_value(request.arguments)?;
                let result = self.handle_cleanup_worktree(config).await?;
                Ok(CallToolResult {
                    content: vec![TextContent {
                        text: result,
                        r#type: "text".to_string(),
                    }],
                    is_error: false,
                })
            }
            "list_worktrees" => {
                let result = self.list_worktrees().await?;
                Ok(CallToolResult {
                    content: vec![TextContent {
                        text: result,
                        r#type: "text".to_string(),
                    }],
                    is_error: false,
                })
            }
            _ => Err(anyhow::anyhow!("Unknown tool: {}", request.name))
        }
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

    // Start the MCP server
    let mut mcp_server = McpServer::new(server);
    
    info!("MCP server started with tools:");
    info!("  - spawn_subagent: Spawn a new subagent with a git worktree");
    info!("  - monitor_agents: Monitor running agent processes");
    info!("  - cleanup_worktree: ⚠️ DESTRUCTIVE: Clean up worktree and agents");
    info!("  - list_worktrees: List all worktrees and their status");

    // Run the MCP server
    mcp_server.run().await?;

    Ok(())
}