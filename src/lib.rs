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
pub use doc_generator::{DocGenerator, run_doc_generator};

/// Main server configuration and implementation
pub mod server {
    pub use crate::main::{SubagentWorktreeServer, SubagentConfig, CleanupConfig};
}

// Include the main module (which contains the server implementation)
mod main;
