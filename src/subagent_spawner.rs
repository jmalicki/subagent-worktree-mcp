use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tracing::{debug, error, info, warn};

/// Trait for different types of agents that can be spawned
#[async_trait]
pub trait AgentSpawner: Send + Sync {
    /// Check if this agent type is available on the system
    async fn is_available(&self) -> Result<bool>;
    
    /// Spawn the agent in the specified directory with the given prompt
    async fn spawn(&self, worktree_path: &Path, prompt: &str, options: &AgentOptions) -> Result<()>;
    
    /// Get information about this agent type
    async fn get_info(&self) -> Result<AgentInfo>;
    
    /// Get the name of this agent type
    fn name(&self) -> &'static str;
}

/// Configuration options for agent spawning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOptions {
    /// Whether to open in a new window/instance
    pub new_window: bool,
    /// Whether to wait for the process to complete
    pub wait: bool,
    /// Whether to detach the process (don't wait for completion)
    pub detach: bool,
    /// Additional custom options specific to the agent type
    pub custom_options: indexmap::IndexMap<String, String>,
}

impl Default for AgentOptions {
    fn default() -> Self {
        Self {
            new_window: true,
            wait: true,
            detach: false,
            custom_options: indexmap::IndexMap::new(),
        }
    }
}

/// Information about an agent type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Whether the agent is available
    pub available: bool,
    /// Version string of the agent
    pub version: String,
    /// Description of the agent
    pub description: String,
}

/// Cursor CLI agent implementation
pub struct CursorCliAgent;

#[async_trait]
impl AgentSpawner for CursorCliAgent {
    async fn is_available(&self) -> Result<bool> {
        let result = TokioCommand::new("which")
            .arg("cursor-cli")
            .output()
            .await
            .context("Failed to execute 'which cursor-cli' command")?;

        Ok(result.status.success())
    }

    async fn spawn(&self, worktree_path: &Path, prompt: &str, options: &AgentOptions) -> Result<()> {
        if !self.is_available().await? {
            return Err(anyhow::anyhow!("cursor-cli is not available in PATH"));
        }

        info!("Spawning cursor-cli in directory: {}", worktree_path.display());
        debug!("Initial prompt: {}", prompt);

        // Use standard library process management
        let mut cmd = TokioCommand::new("cursor-cli");
        
        // Add arguments based on options
        if options.new_window {
            cmd.arg("--new-window");
        }
        if options.wait {
            cmd.arg("--wait");
        }
        
        // Add custom options as arguments
        for (key, value) in &options.custom_options {
            cmd.arg(format!("--{}", key));
            cmd.arg(value);
        }
        
        // Add the worktree path
        cmd.arg(worktree_path);
        
        // Set up stdio
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(worktree_path);

        // Spawn the process
        let mut process = cmd.spawn()
            .context("Failed to spawn cursor-cli process")?;

        // Send the initial prompt
        if let Some(mut stdin) = process.stdin.take() {
            let prompt_bytes = format!("{}\n", prompt).into_bytes();
            tokio::spawn(async move {
                use tokio::io::AsyncWriteExt;
                if let Err(e) = stdin.write_all(&prompt_bytes).await {
                    error!("Failed to write prompt to cursor-cli stdin: {}", e);
                }
            });
        }

        // Handle process completion based on options
        if options.detach {
            // Detach the process - don't wait for it
            tokio::spawn(async move {
                match process.wait().await {
                    Ok(status) => {
                        if status.success() {
                            info!("Detached cursor-cli process completed successfully");
                        } else {
                            warn!("Detached cursor-cli process exited with non-zero status: {:?}", status.code());
                        }
                    }
                    Err(e) => {
                        error!("Error waiting for detached cursor-cli process: {}", e);
                    }
                }
            });
        } else {
            // Wait for the process to complete
            match process.wait().await {
                Ok(status) => {
                    if status.success() {
                        info!("cursor-cli process completed successfully");
                    } else {
                        warn!("cursor-cli process exited with non-zero status: {:?}", status.code());
                    }
                }
                Err(e) => {
                    error!("Error waiting for cursor-cli process: {}", e);
                    return Err(anyhow::anyhow!("Failed to wait for cursor-cli process: {}", e));
                }
            }
        }

        info!("Successfully spawned cursor-cli subagent");
        Ok(())
    }

    async fn get_info(&self) -> Result<AgentInfo> {
        let available = self.is_available().await?;
        
        let version = if available {
            // Try to get version information
            let version_output = TokioCommand::new("cursor-cli")
                .arg("--version")
                .output()
                .await
                .ok();

            version_output
                .and_then(|output| {
                    if output.status.success() {
                        String::from_utf8(output.stdout).ok()
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "Unknown version".to_string())
        } else {
            "Not available".to_string()
        };

        Ok(AgentInfo {
            available,
            version,
            description: "Cursor CLI - AI-powered code editor".to_string(),
        })
    }

    fn name(&self) -> &'static str {
        "cursor-cli"
    }
}

/// Handles spawning of subagent processes with support for multiple agent types
pub struct SubagentSpawner {
    agents: Vec<Box<dyn AgentSpawner>>,
}

impl SubagentSpawner {
    /// Create a new SubagentSpawner
    pub fn new() -> Result<Self> {
        Ok(Self {
            agents: Vec::new(),
        })
    }

    /// Register a new agent type
    pub fn register_agent(&mut self, agent: Box<dyn AgentSpawner>) {
        self.agents.push(agent);
    }

    /// Get all registered agent types
    pub fn get_agents(&self) -> &[Box<dyn AgentSpawner>] {
        &self.agents
    }

    /// Spawn an agent by name
    pub async fn spawn_agent(
        &self,
        agent_name: &str,
        worktree_path: &Path,
        prompt: &str,
        options: &AgentOptions,
    ) -> Result<()> {
        let agent = self.agents.iter()
            .find(|a| a.name() == agent_name)
            .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", agent_name))?;

        agent.spawn(worktree_path, prompt, options).await
    }

    /// List all available agents
    pub async fn list_available_agents(&self) -> Result<Vec<AgentInfo>> {
        let mut available_agents = Vec::new();
        
        for agent in &self.agents {
            if let Ok(info) = agent.get_info().await {
                available_agents.push(info);
            }
        }
        
        Ok(available_agents)
    }
}
