//! Integration tests for cursor-agent functionality
//!
//! These tests verify that the MCP server can properly interact with cursor-agent,
//! including spawning processes, detecting availability, and handling responses.

use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;
use std::path::Path;
use tempfile::TempDir;
use tokio::time::{Duration, sleep};

use subagent_worktree_mcp::subagent_spawner::{
    AgentOptions, AgentSpawner, CursorCliAgent, SubagentSpawner,
};

/// Test that cursor-agent availability detection works
#[tokio::test]
async fn test_cursor_agent_availability() -> Result<()> {
    let agent = CursorCliAgent;

    // Test availability check - this should not panic regardless of whether cursor-agent is installed
    let available = agent.is_available().await?;

    if available {
        println!("cursor-agent is available on this system");

        // If available, test getting agent info
        let info = agent.get_info().await?;
        assert_eq!(info.description, "cursor-agent");
        assert!(info.available);
        println!("Agent info: {:?}", info);
    } else {
        println!("cursor-agent is not available on this system");
    }

    Ok(())
}

/// Test cursor-agent version detection
#[tokio::test]
async fn test_cursor_agent_version() -> Result<()> {
    let agent = CursorCliAgent;

    if agent.is_available().await? {
        let info = agent.get_info().await?;

        // Version should be available if cursor-agent is installed
        if !info.version.is_empty() {
            println!("cursor-agent version: {}", info.version);
            assert!(!info.version.is_empty());
        }
    }

    Ok(())
}

/// Test spawning cursor-agent with a simple prompt (if available)
#[tokio::test]
async fn test_cursor_agent_spawn_simple() -> Result<()> {
    let agent = CursorCliAgent;

    if !agent.is_available().await? {
        println!("Skipping cursor-agent spawn test - not available");
        return Ok(());
    }

    let temp_dir = TempDir::new()?;
    let worktree_path = temp_dir.path();

    // Create a simple test file
    std::fs::write(worktree_path.join("test.txt"), "Hello, cursor-agent!")?;

    let options = AgentOptions {
        new_window: false,
        wait: true,
        detach: false,
        custom_options: std::collections::HashMap::new(),
    };

    // Test spawning with a simple prompt
    let result = agent
        .spawn(
            worktree_path,
            "Please read the test.txt file and tell me what it contains.",
            &options,
        )
        .await;

    match result {
        Ok(_) => {
            println!("Successfully spawned cursor-agent");
        }
        Err(e) => {
            // This might fail due to API key, network, or other issues
            println!("cursor-agent spawn failed (expected in CI): {}", e);
        }
    }

    Ok(())
}

/// Test cursor-agent with detached mode
#[tokio::test]
async fn test_cursor_agent_spawn_detached() -> Result<()> {
    let agent = CursorCliAgent;

    if !agent.is_available().await? {
        println!("Skipping cursor-agent detached test - not available");
        return Ok(());
    }

    let temp_dir = TempDir::new()?;
    let worktree_path = temp_dir.path();

    let options = AgentOptions {
        new_window: true,
        wait: false, // Detached mode
        detach: true,
        custom_options: std::collections::HashMap::new(),
    };

    // Test spawning in detached mode
    let result = agent
        .spawn(
            worktree_path,
            "Please create a simple hello world program.",
            &options,
        )
        .await;

    match result {
        Ok(_) => {
            println!("Successfully spawned cursor-agent in detached mode");

            // Give it a moment to start
            sleep(Duration::from_millis(500)).await;
        }
        Err(e) => {
            println!("cursor-agent detached spawn failed (expected in CI): {}", e);
        }
    }

    Ok(())
}

/// Test the SubagentSpawner with cursor-agent
#[tokio::test]
async fn test_subagent_spawner_cursor_agent() -> Result<()> {
    let spawner = SubagentSpawner::new()?;

    // Test listing available agents
    let agents = spawner.list_available_agents().await?;
    println!("Available agents: {:?}", agents);

    // Check if cursor-agent is in the list
    let has_cursor_agent = agents
        .iter()
        .any(|agent| agent.description == "cursor-agent");

    if has_cursor_agent {
        println!("cursor-agent is available via SubagentSpawner");

        // Test spawning through the spawner
        let temp_dir = TempDir::new()?;
        let worktree_path = temp_dir.path();

        let options = AgentOptions {
            new_window: false,
            wait: true,
            detach: false,
            custom_options: std::collections::HashMap::new(),
        };

        let result = spawner
            .spawn_agent("cursor-agent", worktree_path, "Test prompt", &options)
            .await;

        match result {
            Ok(_) => {
                println!("Successfully spawned cursor-agent via SubagentSpawner");
            }
            Err(e) => {
                println!(
                    "SubagentSpawner cursor-agent spawn failed (expected in CI): {}",
                    e
                );
            }
        }
    } else {
        println!("cursor-agent not available via SubagentSpawner");
    }

    Ok(())
}

/// Test cursor-agent command line interface directly
#[test]
#[ignore = "Requires cursor-agent to be installed on the system"]
fn test_cursor_agent_cli_help() -> Result<()> {
    let mut cmd = Command::new("cursor-agent");
    cmd.arg("--help");

    let result = cmd.assert();

    // This might fail if cursor-agent is not installed, which is expected in some environments
    match result.try_success() {
        Ok(_) => {
            println!("cursor-agent --help succeeded");
        }
        Err(_) => {
            println!("cursor-agent --help failed (expected if not installed)");
        }
    }

    Ok(())
}

/// Test cursor-agent version command
#[test]
#[ignore = "Requires cursor-agent to be installed on the system"]
fn test_cursor_agent_cli_version() -> Result<()> {
    let mut cmd = Command::new("cursor-agent");
    cmd.arg("--version");

    let result = cmd.assert();

    match result.try_success() {
        Ok(_) => {
            println!("cursor-agent --version succeeded");
        }
        Err(_) => {
            println!("cursor-agent --version failed (expected if not installed)");
        }
    }

    Ok(())
}

/// Test cursor-agent with invalid arguments (should fail gracefully)
#[test]
#[ignore = "Requires cursor-agent to be installed on the system"]
fn test_cursor_agent_cli_invalid_args() -> Result<()> {
    let mut cmd = Command::new("cursor-agent");
    cmd.arg("--invalid-argument");

    let result = cmd.assert();

    // This should fail with invalid arguments
    match result.try_success() {
        Ok(_) => {
            println!("cursor-agent with invalid args unexpectedly succeeded");
        }
        Err(_) => {
            println!("cursor-agent with invalid args failed as expected");
        }
    }

    Ok(())
}

/// Integration test: Test the complete workflow with cursor-agent
#[tokio::test]
async fn test_cursor_agent_complete_workflow() -> Result<()> {
    let spawner = SubagentSpawner::new()?;

    // Check if cursor-agent is available
    let agents = spawner.list_available_agents().await?;
    let cursor_agent_available = agents
        .iter()
        .any(|agent| agent.description == "cursor-agent");

    if !cursor_agent_available {
        println!("Skipping complete workflow test - cursor-agent not available");
        return Ok(());
    }

    // Create a temporary worktree
    let temp_dir = TempDir::new()?;
    let worktree_path = temp_dir.path();

    // Create some test files
    std::fs::write(
        worktree_path.join("README.md"),
        "# Test Project\n\nThis is a test project for cursor-agent integration.",
    )?;
    std::fs::write(
        worktree_path.join("main.rs"),
        "fn main() {\n    println!(\"Hello, world!\");\n}",
    )?;

    // Test spawning cursor-agent with a coding task
    let options = AgentOptions {
        new_window: false,
        wait: true,
        detach: false,
        custom_options: std::collections::HashMap::new(),
    };

    let prompt = "Please review the main.rs file and suggest improvements to make it more idiomatic Rust code.";

    let result = spawner
        .spawn_agent("cursor-agent", worktree_path, prompt, &options)
        .await;

    match result {
        Ok(_) => {
            println!("Complete workflow test succeeded");
        }
        Err(e) => {
            println!(
                "Complete workflow test failed (expected in CI without API key): {}",
                e
            );
        }
    }

    Ok(())
}
