use anyhow::Result;
use subagent_worktree_mcp::subagent_spawner::{AgentInfo, AgentOptions, CursorCliAgent};

#[test]
fn test_agent_options_default() -> Result<()> {
    // Test: Verify AgentOptions default values are correct
    // This test ensures the default configuration is sensible

    let options = AgentOptions::default();

    assert_eq!(
        options.new_window, true,
        "Default new_window should be true"
    );
    assert_eq!(options.wait, true, "Default wait should be true");
    assert_eq!(options.detach, false, "Default detach should be false");
    assert_eq!(
        options.custom_options.len(),
        0,
        "Default custom_options should be empty"
    );

    Ok(())
}

#[test]
fn test_agent_options_custom() -> Result<()> {
    // Test: Verify AgentOptions can be created with custom values
    // This test ensures custom configuration works correctly

    let mut custom_options = std::collections::HashMap::new();
    custom_options.insert("timeout".to_string(), "30".to_string());

    let options = AgentOptions {
        new_window: false,
        wait: false,
        detach: true,
        custom_options,
    };

    assert_eq!(
        options.new_window, false,
        "Custom new_window should be false"
    );
    assert_eq!(options.wait, false, "Custom wait should be false");
    assert_eq!(options.detach, true, "Custom detach should be true");
    assert_eq!(
        options.custom_options.len(),
        1,
        "Custom options should have one entry"
    );
    assert_eq!(
        options.custom_options.get("timeout"),
        Some(&"30".to_string())
    );

    Ok(())
}

#[test]
fn test_agent_info_creation() -> Result<()> {
    // Test: Verify AgentInfo can be created successfully
    // This test ensures the agent information structure works

    let info = AgentInfo {
        available: true,
        version: "1.0.0".to_string(),
        description: "Test agent for unit testing".to_string(),
    };

    assert_eq!(info.available, true, "Available should be true");
    assert_eq!(info.version, "1.0.0", "Version should be correct");
    assert_eq!(
        info.description, "Test agent for unit testing",
        "Description should be correct"
    );

    Ok(())
}

#[test]
fn test_agent_info_unavailable() -> Result<()> {
    // Test: Verify AgentInfo can represent unavailable agents
    // This test ensures the structure handles unavailable agents correctly

    let info = AgentInfo {
        available: false,
        version: "".to_string(),
        description: "Agent not found".to_string(),
    };

    assert_eq!(info.available, false, "Available should be false");
    assert_eq!(
        info.version, "",
        "Version should be empty for unavailable agent"
    );
    assert_eq!(
        info.description, "Agent not found",
        "Description should indicate unavailability"
    );

    Ok(())
}

#[test]
fn test_cursor_cli_agent_creation() -> Result<()> {
    // Test: Verify CursorCliAgent can be created successfully
    // This test ensures the agent struct can be instantiated

    let agent = CursorCliAgent;

    // Just verify it can be created - this is a unit test
    assert!(true, "CursorCliAgent should be created successfully");

    Ok(())
}

#[test]
fn test_agent_options_serialization() -> Result<()> {
    // Test: Verify AgentOptions can be serialized and deserialized
    // This test ensures the struct works with serde

    let options = AgentOptions {
        new_window: true,
        wait: false,
        detach: true,
        custom_options: {
            let mut map = std::collections::HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map.insert("key2".to_string(), "value2".to_string());
            map
        },
    };

    // Test JSON serialization
    let json = serde_json::to_string(&options)?;
    let deserialized: AgentOptions = serde_json::from_str(&json)?;

    assert_eq!(deserialized.new_window, options.new_window);
    assert_eq!(deserialized.wait, options.wait);
    assert_eq!(deserialized.detach, options.detach);
    assert_eq!(
        deserialized.custom_options.len(),
        options.custom_options.len()
    );

    Ok(())
}

#[test]
fn test_agent_info_serialization() -> Result<()> {
    // Test: Verify AgentInfo can be serialized and deserialized
    // This test ensures the struct works with serde

    let info = AgentInfo {
        available: true,
        version: "2.1.0".to_string(),
        description: "Serialization test agent".to_string(),
    };

    // Test JSON serialization
    let json = serde_json::to_string(&info)?;
    let deserialized: AgentInfo = serde_json::from_str(&json)?;

    assert_eq!(deserialized.available, info.available);
    assert_eq!(deserialized.version, info.version);
    assert_eq!(deserialized.description, info.description);

    Ok(())
}

#[test]
fn test_agent_options_edge_cases() -> Result<()> {
    // Test: Verify AgentOptions handles edge cases correctly
    // This test ensures the struct handles unusual but valid inputs

    // Test with empty custom options
    let options_empty = AgentOptions {
        new_window: false,
        wait: false,
        detach: false,
        custom_options: std::collections::HashMap::new(),
    };

    assert_eq!(options_empty.custom_options.len(), 0);

    // Test with large custom options
    let mut large_options = std::collections::HashMap::new();
    for i in 0..100 {
        large_options.insert(format!("key_{}", i), format!("value_{}", i));
    }

    let options_large = AgentOptions {
        new_window: true,
        wait: true,
        detach: false,
        custom_options: large_options,
    };

    assert_eq!(options_large.custom_options.len(), 100);

    Ok(())
}

#[test]
fn test_agent_info_edge_cases() -> Result<()> {
    // Test: Verify AgentInfo handles edge cases correctly
    // This test ensures the struct handles unusual but valid inputs

    // Test with empty strings
    let info_empty = AgentInfo {
        available: false,
        version: "".to_string(),
        description: "".to_string(),
    };

    assert_eq!(info_empty.version.len(), 0);
    assert_eq!(info_empty.description.len(), 0);

    // Test with long strings
    let long_string = "a".repeat(1000);
    let info_long = AgentInfo {
        available: true,
        version: long_string.clone(),
        description: long_string.clone(),
    };

    assert_eq!(info_long.version.len(), 1000);
    assert_eq!(info_long.description.len(), 1000);

    Ok(())
}
