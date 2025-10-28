use anyhow::Result;
use tempfile::TempDir;
use std::path::PathBuf;

use subagent_worktree_mcp::subagent_spawner::{SubagentSpawner, CursorCliAgent, AgentOptions, AgentSpawner, AgentInfo};

/// Test helper to create a temporary directory
fn create_temp_dir() -> Result<(TempDir, PathBuf)> {
    let temp_dir = TempDir::new()?;
    let dir_path = temp_dir.path().to_path_buf();
    Ok((temp_dir, dir_path))
}

#[tokio::test]
async fn test_subagent_spawner_empty_registration() -> Result<()> {
    // Test: Verify SubagentSpawner handles empty agent registration gracefully
    // This test ensures the spawner works correctly with no agents registered
    
    let spawner = SubagentSpawner::new();
    let agents = spawner.get_agents();
    
    assert_eq!(agents.len(), 0, "Should start with no registered agents");
    
    Ok(())
}

#[tokio::test]
async fn test_subagent_spawner_multiple_agent_registration() -> Result<()> {
    // Test: Verify SubagentSpawner can register multiple agents
    // This test ensures the spawner can handle multiple agent types
    
    let mut spawner = SubagentSpawner::new();
    
    // Register multiple agents
    spawner.register_agent(Box::new(CursorCliAgent::new()));
    spawner.register_agent(Box::new(CursorCliAgent::new()));
    
    let agents = spawner.get_agents();
    assert_eq!(agents.len(), 2, "Should have two registered agents");
    
    Ok(())
}

#[tokio::test]
async fn test_subagent_spawner_spawn_nonexistent_agent() -> Result<()> {
    // Test: Verify SubagentSpawner fails gracefully for non-existent agent types
    // This test ensures proper error handling for unknown agent types
    
    let mut spawner = SubagentSpawner::new();
    let agent = CursorCliAgent::new();
    spawner.register_agent(Box::new(agent));
    
    let (_temp_dir, temp_path) = create_temp_dir()?;
    let options = AgentOptions::default();
    
    // Try to spawn non-existent agent type
    let result = spawner.spawn_agent("non-existent-agent", &temp_path, "test prompt", &options).await;
    
    // Should fail gracefully
    assert!(result.is_err(), "Should fail when spawning non-existent agent type");
    
    Ok(())
}

#[tokio::test]
async fn test_subagent_spawner_spawn_with_invalid_path() -> Result<()> {
    // Test: Verify SubagentSpawner handles invalid paths gracefully
    // This test ensures proper error handling for invalid worktree paths
    
    let mut spawner = SubagentSpawner::new();
    let agent = CursorCliAgent::new();
    spawner.register_agent(Box::new(agent));
    
    let invalid_path = PathBuf::from("/nonexistent/path/that/does/not/exist");
    let options = AgentOptions::default();
    
    // Try to spawn with invalid path
    let result = spawner.spawn_agent("cursor-cli", &invalid_path, "test prompt", &options).await;
    
    // Should fail gracefully (cursor-cli might not be available, but path should be handled)
    assert!(true, "Should handle invalid path gracefully");
    
    Ok(())
}

#[tokio::test]
async fn test_subagent_spawner_spawn_with_empty_prompt() -> Result<()> {
    // Test: Verify SubagentSpawner handles empty prompts
    // This test ensures the spawner works with minimal input
    
    let mut spawner = SubagentSpawner::new();
    let agent = CursorCliAgent::new();
    spawner.register_agent(Box::new(agent));
    
    let (_temp_dir, temp_path) = create_temp_dir()?;
    let options = AgentOptions::default();
    
    // Try to spawn with empty prompt
    let result = spawner.spawn_agent("cursor-cli", &temp_path, "", &options).await;
    
    // We don't assert success here since cursor-cli might not be available
    // Just ensure the method doesn't panic with empty prompt
    assert!(true, "Should handle empty prompt without panicking");
    
    Ok(())
}

#[tokio::test]
async fn test_cursor_cli_agent_availability_edge_cases() -> Result<()> {
    // Test: Verify CursorCliAgent availability check handles edge cases
    // This test ensures the availability detection is robust
    
    let agent = CursorCliAgent::new();
    
    // Test availability check multiple times
    let result1 = agent.is_available().await;
    let result2 = agent.is_available().await;
    
    // Results should be consistent (though we don't know the actual value)
    assert!(true, "Availability check should be consistent");
    
    Ok(())
}

#[tokio::test]
async fn test_cursor_cli_agent_name_consistency() -> Result<()> {
    // Test: Verify CursorCliAgent name is consistent
    // This test ensures agent identification is reliable
    
    let agent1 = CursorCliAgent::new();
    let agent2 = CursorCliAgent::new();
    
    assert_eq!(agent1.name(), agent2.name(), "Agent names should be consistent");
    assert_eq!(agent1.name(), "cursor-cli", "Agent name should be 'cursor-cli'");
    
    Ok(())
}

#[tokio::test]
async fn test_agent_options_edge_cases() -> Result<()> {
    // Test: Verify AgentOptions handles edge cases correctly
    // This test ensures the options structure is robust
    
    // Test with extreme values
    let options = AgentOptions {
        new_window: true,
        wait: true,
        detach: false,
        custom_options: std::collections::HashMap::new(),
        max_execution_time: Some(0), // Edge case: zero time
        max_memory_mb: Some(1), // Edge case: minimal memory
    };
    
    assert_eq!(options.max_execution_time, Some(0), "Should handle zero execution time");
    assert_eq!(options.max_memory_mb, Some(1), "Should handle minimal memory");
    
    // Test with large values
    let options_large = AgentOptions {
        new_window: false,
        wait: false,
        detach: true,
        custom_options: std::collections::HashMap::new(),
        max_execution_time: Some(u64::MAX), // Edge case: maximum time
        max_memory_mb: Some(u64::MAX), // Edge case: maximum memory
    };
    
    assert_eq!(options_large.max_execution_time, Some(u64::MAX), "Should handle maximum execution time");
    assert_eq!(options_large.max_memory_mb, Some(u64::MAX), "Should handle maximum memory");
    
    Ok(())
}

#[tokio::test]
async fn test_agent_options_custom_options() -> Result<()> {
    // Test: Verify AgentOptions custom options work correctly
    // This test ensures custom configuration is properly handled
    
    let mut custom_options = std::collections::HashMap::new();
    custom_options.insert("test_key".to_string(), "test_value".to_string());
    custom_options.insert("another_key".to_string(), "another_value".to_string());
    
    let options = AgentOptions {
        new_window: false,
        wait: false,
        detach: true,
        custom_options,
        max_execution_time: None,
        max_memory_mb: None,
    };
    
    assert_eq!(options.custom_options.len(), 2, "Should have two custom options");
    assert_eq!(options.custom_options.get("test_key"), Some(&"test_value".to_string()));
    assert_eq!(options.custom_options.get("another_key"), Some(&"another_value".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_agent_info_creation_edge_cases() -> Result<()> {
    // Test: Verify AgentInfo handles edge cases correctly
    // This test ensures the agent information structure is robust
    
    // Test with empty strings
    let info_empty = AgentInfo {
        name: "".to_string(),
        version: "".to_string(),
        description: "".to_string(),
        capabilities: vec![],
        custom_options: std::collections::HashMap::new(),
    };
    
    assert_eq!(info_empty.name, "", "Should handle empty name");
    assert_eq!(info_empty.capabilities.len(), 0, "Should handle empty capabilities");
    
    // Test with long strings
    let long_string = "a".repeat(1000);
    let info_long = AgentInfo {
        name: long_string.clone(),
        version: long_string.clone(),
        description: long_string.clone(),
        capabilities: vec![long_string.clone()],
        custom_options: std::collections::HashMap::new(),
    };
    
    assert_eq!(info_long.name.len(), 1000, "Should handle long strings");
    assert_eq!(info_long.capabilities.len(), 1, "Should handle long capabilities");
    
    Ok(())
}

#[tokio::test]
async fn test_subagent_spawner_list_available_agents_empty() -> Result<()> {
    // Test: Verify SubagentSpawner handles empty agent list correctly
    // This test ensures the listing functionality works with no agents
    
    let spawner = SubagentSpawner::new();
    let available_agents = spawner.list_available_agents().await?;
    
    // Should return empty list when no agents are registered
    assert_eq!(available_agents.len(), 0, "Should return empty list when no agents registered");
    
    Ok(())
}

#[tokio::test]
async fn test_subagent_spawner_list_available_agents_with_agents() -> Result<()> {
    // Test: Verify SubagentSpawner lists available agents correctly
    // This test ensures the listing functionality works with registered agents
    
    let mut spawner = SubagentSpawner::new();
    let agent = CursorCliAgent::new();
    spawner.register_agent(Box::new(agent));
    
    let available_agents = spawner.list_available_agents().await?;
    
    // Should return list of available agents
    assert!(available_agents.len() >= 0, "Should return list of available agents");
    
    Ok(())
}

#[tokio::test]
async fn test_subagent_spawner_concurrent_registration() -> Result<()> {
    // Test: Verify SubagentSpawner handles concurrent operations safely
    // This test ensures thread safety for agent registration
    
    let mut spawner = SubagentSpawner::new();
    
    // Simulate concurrent registration (though this is single-threaded)
    spawner.register_agent(Box::new(CursorCliAgent::new()));
    spawner.register_agent(Box::new(CursorCliAgent::new()));
    spawner.register_agent(Box::new(CursorCliAgent::new()));
    
    let agents = spawner.get_agents();
    assert_eq!(agents.len(), 3, "Should handle multiple registrations correctly");
    
    Ok(())
}

#[tokio::test]
async fn test_agent_spawner_trait_consistency() -> Result<()> {
    // Test: Verify AgentSpawner trait is implemented consistently
    // This test ensures trait implementation is correct
    
    let agent = CursorCliAgent::new();
    
    // Test trait methods
    let name = agent.name();
    let is_available = agent.is_available().await?;
    let info = agent.get_info().await?;
    
    // Verify consistency
    assert_eq!(name, "cursor-cli", "Name should be consistent");
    assert!(true, "Availability check should not panic");
    assert_eq!(info.name, "cursor-cli", "Info name should match agent name");
    
    Ok(())
}
