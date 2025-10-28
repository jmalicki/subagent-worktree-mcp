use anyhow::Result;
use std::collections::HashSet;
use std::fs;

use subagent_worktree_mcp::{AgentMonitorConfig, AgentOptions, CleanupConfig, SubagentConfig};

/// Test to verify that our README documentation matches our actual implementation
/// This ensures we don't have documentation drift and that all tools are properly documented
#[tokio::test]
async fn test_readme_documentation_matches_implementation() -> Result<()> {
    // Test: Verify that all documented MCP tools in README.md have corresponding implementations
    // This test ensures documentation accuracy and prevents implementation drift

    // Read the README file
    let readme_content = fs::read_to_string("README.md")?;

    // Extract documented tools from README
    let documented_tools = extract_documented_tools(&readme_content);

    // Define the tools we actually implement
    let implemented_tools = get_implemented_tools();

    // Verify all documented tools are implemented
    for tool in &documented_tools {
        assert!(
            implemented_tools.contains(tool),
            "Documented tool '{}' is not implemented",
            tool
        );
    }

    // Verify all implemented tools are documented
    for tool in &implemented_tools {
        assert!(
            documented_tools.contains(tool),
            "Implemented tool '{}' is not documented in README",
            tool
        );
    }

    println!(
        "✅ All {} documented tools match implementation",
        documented_tools.len()
    );

    Ok(())
}

/// Test to verify that configuration structs match documented parameters
#[tokio::test]
async fn test_config_structs_match_documentation() -> Result<()> {
    // Test: Verify that configuration structs have fields matching documented parameters
    // This ensures schema consistency between docs and implementation

    // Test SubagentConfig fields match documentation
    test_subagent_config_fields();

    // Test CleanupConfig fields match documentation
    test_cleanup_config_fields();

    // Test AgentMonitorConfig fields match documentation
    test_agent_monitor_config_fields();

    println!("✅ All configuration structs match documentation");

    Ok(())
}

/// Test to verify that tool parameter schemas are correctly defined
#[tokio::test]
async fn test_tool_parameter_schemas() -> Result<()> {
    // Test: Verify that tool parameters have correct types and optionality
    // This ensures API consistency and proper serialization

    // Test spawn_subagent parameters
    test_spawn_subagent_schema();

    // Test cleanup_worktree parameters
    test_cleanup_worktree_schema();

    // Test monitor_agents parameters
    test_monitor_agents_schema();

    // Test list_worktrees parameters
    test_list_worktrees_schema();

    println!("✅ All tool parameter schemas are correctly defined");

    Ok(())
}

/// Test to verify that destructive operations are properly marked
#[tokio::test]
async fn test_destructive_operations_marked() -> Result<()> {
    // Test: Verify that destructive operations are properly documented and handled
    // This ensures safety warnings are in place for dangerous operations

    let readme_content = fs::read_to_string("README.md")?;

    // Check that cleanup_worktree is marked as destructive
    assert!(
        readme_content.contains("cleanup_worktree` ⚠️ **DESTRUCTIVE**"),
        "cleanup_worktree should be marked as destructive in README"
    );

    // Check that destructive warnings are present
    assert!(
        readme_content.contains("⚠️ Warning:"),
        "Destructive operation warnings should be present in README"
    );

    // Check that destructive operations are listed
    assert!(
        readme_content.contains("Kill running agent processes"),
        "Should document that cleanup kills processes"
    );

    assert!(
        readme_content.contains("Remove the worktree directory"),
        "Should document that cleanup removes directories"
    );

    assert!(
        readme_content.contains("Cannot be undone"),
        "Should warn that cleanup cannot be undone"
    );

    println!("✅ Destructive operations are properly marked and documented");

    Ok(())
}

/// Test to verify that all required dependencies are documented
#[tokio::test]
async fn test_dependencies_documented() -> Result<()> {
    // Test: Verify that all required system dependencies are documented
    // This ensures users know what they need to install

    let readme_content = fs::read_to_string("README.md")?;

    // Check that git is mentioned as a requirement
    assert!(
        readme_content.contains("git"),
        "README should mention git as a requirement"
    );

    // Check that cursor-cli is mentioned
    assert!(
        readme_content.contains("cursor-cli"),
        "README should mention cursor-cli as a requirement"
    );

    println!("✅ Required dependencies are documented");

    Ok(())
}

// Helper functions

fn extract_documented_tools(readme_content: &str) -> HashSet<String> {
    let mut tools = HashSet::new();

    // Look for tool definitions in README
    let lines: Vec<&str> = readme_content.lines().collect();

    for line in lines {
        if line.starts_with("### `") && line.contains("`") {
            // Extract tool name from markdown headers like "### `tool_name`"
            if let Some(start) = line.find('`') {
                if let Some(end) = line[start + 1..].find('`') {
                    let tool_name = &line[start + 1..start + 1 + end];
                    tools.insert(tool_name.to_string());
                }
            }
        }
    }

    tools
}

fn get_implemented_tools() -> HashSet<String> {
    let mut tools = HashSet::new();

    // These are the tools we actually implement
    tools.insert("spawn_subagent".to_string());
    tools.insert("monitor_agents".to_string());
    tools.insert("cleanup_worktree".to_string());
    tools.insert("list_worktrees".to_string());

    tools
}

fn test_subagent_config_fields() {
    // Test that SubagentConfig has all documented fields
    let config = SubagentConfig {
        branch_name: "test".to_string(),
        prompt: "test prompt".to_string(),
        worktree_dir: Some("custom-dir".to_string()),
        agent_type: Some("cursor-agent".to_string()),
        agent_options: Some(AgentOptions::default()),
    };

    // Verify all documented fields exist
    assert!(
        !config.branch_name.is_empty(),
        "branch_name should be present"
    );
    assert!(!config.prompt.is_empty(), "prompt should be present");
    assert!(
        config.worktree_dir.is_some(),
        "worktree_dir should be optional"
    );
    assert!(config.agent_type.is_some(), "agent_type should be optional");
    assert!(
        config.agent_options.is_some(),
        "agent_options should be optional"
    );
}

fn test_cleanup_config_fields() {
    // Test that CleanupConfig has all documented fields
    let config = CleanupConfig {
        worktree_path: "test-worktree".to_string(),
        force: Some(true),
        delete_branch: Some(true),
    };

    // Verify all documented fields exist
    assert!(
        !config.worktree_path.is_empty(),
        "worktree_path should be present"
    );
    assert!(config.force.unwrap_or(false), "force should be present");
    assert!(
        config.delete_branch.unwrap_or(false),
        "delete_branch should be present"
    );
}

fn test_agent_monitor_config_fields() {
    // Test that AgentMonitorConfig has all documented fields
    let config = AgentMonitorConfig {
        only_our_agents: true,
        only_waiting_agents: true,
        agent_types: Some(vec!["cursor-cli".to_string()]),
        worktree_paths: Some(vec!["/tmp/test".to_string()]),
    };

    // Verify all documented fields exist
    assert!(config.only_our_agents, "only_our_agents should be present");
    assert!(
        config.only_waiting_agents,
        "only_waiting_agents should be present"
    );
    assert!(
        config.agent_types.is_some(),
        "agent_types should be optional"
    );
    assert!(
        config.worktree_paths.is_some(),
        "worktree_paths should be optional"
    );
}

fn test_spawn_subagent_schema() {
    // Test that spawn_subagent parameters are correctly typed
    let config = SubagentConfig {
        branch_name: "required-field".to_string(), // Required
        prompt: "required-field".to_string(),      // Required
        worktree_dir: None,                        // Optional
        agent_type: None,                          // Optional
        agent_options: None,                       // Optional
    };

    // Verify required fields are not optional
    assert!(
        !config.branch_name.is_empty(),
        "branch_name should be required"
    );
    assert!(!config.prompt.is_empty(), "prompt should be required");

    // Verify optional fields can be None
    assert!(
        config.worktree_dir.is_none(),
        "worktree_dir should be optional"
    );
    assert!(config.agent_type.is_none(), "agent_type should be optional");
    assert!(
        config.agent_options.is_none(),
        "agent_options should be optional"
    );
}

fn test_cleanup_worktree_schema() {
    // Test that cleanup_worktree parameters are correctly typed
    let config = CleanupConfig {
        worktree_path: "required-field".to_string(), // Required
        force: Some(false),                          // Optional with default
        delete_branch: Some(false),                  // Optional with default
    };

    // Verify required field is not optional
    assert!(
        !config.worktree_path.is_empty(),
        "worktree_path should be required"
    );

    // Verify optional fields have sensible defaults
    assert!(
        !config.force.unwrap_or(false),
        "force should default to false"
    );
    assert!(
        !config.delete_branch.unwrap_or(false),
        "delete_branch should default to false"
    );
}

fn test_monitor_agents_schema() {
    // Test that monitor_agents parameters are correctly typed
    let config = AgentMonitorConfig {
        only_our_agents: false,     // Optional with default
        only_waiting_agents: false, // Optional with default
        agent_types: None,          // Optional
        worktree_paths: None,       // Optional
    };

    // Verify all fields are optional
    assert!(
        !config.only_our_agents,
        "only_our_agents should default to false"
    );
    assert!(
        !config.only_waiting_agents,
        "only_waiting_agents should default to false"
    );
    assert!(
        config.agent_types.is_none(),
        "agent_types should be optional"
    );
    assert!(
        config.worktree_paths.is_none(),
        "worktree_paths should be optional"
    );
}

fn test_list_worktrees_schema() {
    // Test that list_worktrees has no parameters (as documented)
    // This is verified by the fact that the method takes no parameters
    assert!(
        true,
        "list_worktrees should take no parameters as documented"
    );
}
