use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use sysinfo::System;
use tracing::{debug, info, warn};

/// Information about a running agent process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name (e.g., "cursor-cli", "code", "vim")
    pub name: String,
    /// Command line arguments
    pub cmd: Vec<String>,
    /// Working directory
    pub cwd: String,
    /// Whether the process is waiting for input (stdin)
    pub waiting_for_input: bool,
    /// CPU usage percentage
    pub cpu_usage: f32,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Process start time
    pub start_time: u64,
    /// Whether this agent was spawned by our system
    pub spawned_by_us: bool,
    /// Associated worktree path if known
    pub worktree_path: Option<String>,
}

/// Configuration for monitoring agents
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentMonitorConfig {
    /// Only show agents spawned by our system
    pub only_our_agents: bool,
    /// Only show agents waiting for input
    pub only_waiting_agents: bool,
    /// Filter by specific agent types (e.g., ["cursor-cli", "code"])
    pub agent_types: Option<Vec<String>>,
    /// Filter by worktree paths
    pub worktree_paths: Option<Vec<String>>,
}

/// Monitors running agent processes and their status
pub struct AgentMonitor {
    /// System information for process monitoring
    system: System,
    /// Tracked agent processes (PID -> info)
    tracked_agents: HashMap<u32, AgentProcessInfo>,
    /// Repository path to identify worktrees
    repo_path: std::path::PathBuf,
}

impl AgentMonitor {
    /// Create a new AgentMonitor
    pub fn new(repo_path: std::path::PathBuf) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system,
            tracked_agents: HashMap::new(),
            repo_path,
        }
    }

    /// Refresh system information and update tracked agents
    pub async fn refresh(&mut self) -> Result<()> {
        self.system.refresh_all();
        self.update_tracked_agents().await?;
        Ok(())
    }

    /// Get all running agent processes matching the configuration
    pub async fn get_running_agents(
        &mut self,
        config: &AgentMonitorConfig,
    ) -> Result<Vec<AgentProcessInfo>> {
        self.refresh().await?;

        let mut agents = Vec::new();

        for (pid, process) in self.system.processes() {
            if self.is_agent_process(process) {
                let agent_info = self.create_agent_info(pid.as_u32(), process)?;

                // Apply filters
                if config.only_our_agents && !agent_info.spawned_by_us {
                    continue;
                }

                if config.only_waiting_agents && !agent_info.waiting_for_input {
                    continue;
                }

                if let Some(ref agent_types) = config.agent_types
                    && !agent_types.contains(&agent_info.name)
                {
                    continue;
                }

                if let Some(ref worktree_paths) = config.worktree_paths {
                    if let Some(ref agent_worktree) = agent_info.worktree_path {
                        if !worktree_paths.iter().any(|wp| agent_worktree.contains(wp)) {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }

                agents.push(agent_info);
            }
        }

        // Sort by PID for consistent ordering
        agents.sort_by_key(|a| a.pid);

        info!(
            "Found {} running agent processes matching criteria",
            agents.len()
        );
        Ok(agents)
    }

    /// Check if a process is likely an agent (editor/IDE)
    fn is_agent_process(&self, process: &sysinfo::Process) -> bool {
        let name = process.name().to_lowercase();

        // Common agent/editor process names
        let agent_names = [
            "cursor",
            "cursor-cli",
            "code",
            "code-cli",
            "vim",
            "nvim",
            "emacs",
            "sublime",
            "atom",
            "brackets",
            "webstorm",
            "intellij",
            "pycharm",
            "clion",
            "rider",
            "goland",
            "phpstorm",
            "rubymine",
            "datagrip",
            "android-studio",
            "fleet",
            "zed",
            "lapce",
            "helix",
            "kakoune",
        ];

        agent_names
            .iter()
            .any(|&agent_name| name.contains(agent_name))
    }

    /// Create AgentProcessInfo from a system process
    fn create_agent_info(&self, pid: u32, process: &sysinfo::Process) -> Result<AgentProcessInfo> {
        let cmd = process.cmd().to_vec();
        let cwd = process
            .cwd()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        // Determine if this is waiting for input by checking if it's reading from stdin
        let waiting_for_input = self.is_process_waiting_for_input(pid)?;

        // Determine if this was spawned by our system
        let spawned_by_us = self.is_spawned_by_us(&cmd, &cwd);

        // Determine associated worktree path
        let worktree_path = self.find_associated_worktree(&cwd);

        Ok(AgentProcessInfo {
            pid,
            name: process.name().to_string(),
            cmd,
            cwd,
            waiting_for_input,
            cpu_usage: process.cpu_usage(),
            memory_usage: process.memory(),
            start_time: process.start_time(),
            spawned_by_us,
            worktree_path,
        })
    }

    /// Check if a process is waiting for input from stdin
    fn is_process_waiting_for_input(&self, pid: u32) -> Result<bool> {
        // On Unix systems, we can check if the process has stdin open and is in a waiting state
        #[cfg(unix)]
        {
            use std::fs;

            // Check if stdin is a terminal (TTY)
            let stdin_path = format!("/proc/{}/fd/0", pid);
            if let Ok(link) = fs::read_link(&stdin_path) {
                let link_str = link.to_string_lossy();
                // If stdin is a terminal, the process might be waiting for input
                if link_str.contains("pts") || link_str.contains("tty") {
                    return Ok(true);
                }
            }
        }

        // Fallback: assume processes with terminal stdin might be waiting
        Ok(false)
    }

    /// Determine if a process was spawned by our system
    fn is_spawned_by_us(&self, cmd: &[String], cwd: &str) -> bool {
        // Check if the working directory is a worktree of our repository
        if let Some(_worktree_path) = self.find_associated_worktree(cwd) {
            // Check if the command contains our typical agent spawning patterns
            let cmd_str = cmd.join(" ");

            // Look for common patterns that indicate we spawned this process
            let our_patterns = ["--new-window", "--wait", "cursor-cli", "code --new-window"];

            return our_patterns.iter().any(|pattern| cmd_str.contains(pattern));
        }

        false
    }

    /// Find the associated worktree path for a given directory
    fn find_associated_worktree(&self, dir: &str) -> Option<String> {
        let dir_path = std::path::Path::new(dir);

        // Check if this directory is a worktree of our repository
        if let Some(parent) = dir_path.parent()
            && parent.parent() == Some(self.repo_path.parent().unwrap_or(&self.repo_path))
        {
            // This looks like a worktree directory
            return Some(dir.to_string());
        }

        None
    }

    /// Update our tracked agents with current system state
    async fn update_tracked_agents(&mut self) -> Result<()> {
        // For now, we'll refresh the system info and rebuild the tracked agents
        // In a more sophisticated implementation, we could track process lifecycle events

        self.tracked_agents.clear();

        for (pid, process) in self.system.processes() {
            if self.is_agent_process(process)
                && let Ok(agent_info) = self.create_agent_info(pid.as_u32(), process)
            {
                self.tracked_agents.insert(pid.as_u32(), agent_info);
            }
        }

        debug!(
            "Updated tracked agents: {} processes",
            self.tracked_agents.len()
        );
        Ok(())
    }

    /// Get detailed information about a specific agent process
    pub async fn get_agent_details(&mut self, pid: u32) -> Result<Option<AgentProcessInfo>> {
        self.refresh().await?;

        if let Some(process) = self.system.process(sysinfo::Pid::from_u32(pid))
            && self.is_agent_process(process)
        {
            return Ok(Some(self.create_agent_info(pid, process)?));
        }

        Ok(None)
    }

    /// Kill an agent process (use with caution!)
    pub async fn kill_agent(&mut self, pid: u32, force: bool) -> Result<bool> {
        let signal = if force { "KILL" } else { "TERM" };

        let output = Command::new("kill")
            .arg(format!("-{}", signal))
            .arg(pid.to_string())
            .output()
            .context("Failed to execute kill command")?;

        if output.status.success() {
            info!("Successfully sent {} signal to process {}", signal, pid);
            Ok(true)
        } else {
            warn!(
                "Failed to kill process {}: {}",
                pid,
                String::from_utf8_lossy(&output.stderr)
            );
            Ok(false)
        }
    }

    /// Get summary statistics about running agents
    pub async fn get_agent_summary(&mut self) -> Result<AgentSummary> {
        self.refresh().await?;

        let mut summary = AgentSummary::default();

        for agent in self.tracked_agents.values() {
            summary.total_agents += 1;
            summary.total_cpu_usage += agent.cpu_usage;
            summary.total_memory_usage += agent.memory_usage;

            if agent.waiting_for_input {
                summary.waiting_for_input += 1;
            }

            if agent.spawned_by_us {
                summary.spawned_by_us += 1;
            }

            // Count by agent type
            *summary.agent_types.entry(agent.name.clone()).or_insert(0) += 1;
        }

        Ok(summary)
    }
}

/// Summary statistics about running agents
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentSummary {
    /// Total number of agent processes
    pub total_agents: usize,
    /// Number of agents waiting for input
    pub waiting_for_input: usize,
    /// Number of agents spawned by our system
    pub spawned_by_us: usize,
    /// Total CPU usage across all agents
    pub total_cpu_usage: f32,
    /// Total memory usage across all agents
    pub total_memory_usage: u64,
    /// Count of agents by type
    pub agent_types: HashMap<String, usize>,
}
