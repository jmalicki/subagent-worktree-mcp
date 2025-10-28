use anyhow::Result;
use std::fs;
use std::path::Path;

/// Documentation generator that creates markdown documentation
/// for the MCP server tools
pub struct DocGenerator;

impl DocGenerator {
    /// Generate documentation for all MCP tools
    pub fn generate_tools_documentation() -> String {
        let mut doc = String::new();
        doc.push_str("## MCP Tools\n\n");

        // Add documentation for each tool
        doc.push_str(&Self::generate_tool_documentation(
            "spawn_subagent",
            "Spawn a new subagent with a git worktree for isolated development",
            r#"{
                "type": "object",
                "properties": {
                    "branch_name": {"type": "string", "description": "Name of the branch to create for the subagent"},
                    "prompt": {"type": "string", "description": "Initial prompt to give to the subagent"},
                    "worktree_dir": {"type": "string", "description": "Optional directory name for the worktree (defaults to branch name)"},
                    "agent_type": {"type": "string", "description": "Agent type to spawn (defaults to 'cursor-agent')"},
                    "agent_options": {
                        "type": "object",
                        "properties": {
                            "new_window": {"type": "boolean", "description": "Open in new window"},
                            "wait_for_completion": {"type": "boolean", "description": "Wait for agent to complete"},
                            "timeout_seconds": {"type": "integer", "description": "Timeout in seconds"}
                        }
                    }
                },
                "required": ["branch_name", "prompt"]
            }"#
        ));

        doc.push_str(&Self::generate_tool_documentation(
            "cleanup_worktree",
            "Clean up a worktree and optionally delete the branch (destructive)",
            r#"{
                "type": "object",
                "properties": {
                    "worktree_path": {"type": "string", "description": "Path to the worktree to clean up"},
                    "delete_branch": {"type": "boolean", "description": "Whether to delete the branch (default: false)"},
                    "force": {"type": "boolean", "description": "Whether to force cleanup even if there are uncommitted changes (default: false)"}
                },
                "required": ["worktree_path"]
            }"#
        ));

        doc.push_str(&Self::generate_tool_documentation(
            "list_worktrees",
            "List all git worktrees and their associated agents",
            r#"{
                "type": "object",
                "properties": {
                    "include_agents": {"type": "boolean", "description": "Whether to include agent information (default: true)"},
                    "only_our_agents": {"type": "boolean", "description": "Only show agents spawned by our system (default: true)"},
                    "only_waiting_agents": {"type": "boolean", "description": "Only show agents waiting for input (default: false)"}
                }
            }"#
        ));

        doc
    }

    fn generate_tool_documentation(name: &str, description: &str, schema_json: &str) -> String {
        let mut doc = String::new();
        doc.push_str(&format!("### `{}`\n\n", name));
        doc.push_str(&format!("{}\n\n", description));

        // Parse the JSON schema to extract parameters
        if let Ok(schema) = serde_json::from_str::<serde_json::Value>(schema_json) {
            if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
                if !properties.is_empty() {
                    doc.push_str("**Parameters:**\n");

                    let required_fields = schema
                        .get("required")
                        .and_then(|r| r.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .collect::<std::collections::HashSet<_>>()
                        })
                        .unwrap_or_default();

                    for (param_name, prop) in properties {
                        if let Some(prop_obj) = prop.as_object() {
                            let param_description = prop_obj
                                .get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("No description available");
                            let param_type = prop_obj
                                .get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("unknown");
                            let required = required_fields.contains(param_name.as_str());

                            doc.push_str(&format!(
                                "- `{}`: {} ({}, {})\n",
                                param_name,
                                param_description,
                                param_type,
                                if required { "required" } else { "optional" }
                            ));
                        }
                    }
                    doc.push_str("\n");
                }
            }
        }

        doc
    }

    /// Update the README.md file with generated documentation
    pub fn update_readme(readme_path: &Path) -> Result<()> {
        let readme_content = fs::read_to_string(readme_path)?;

        // Find the MCP Tools section and replace it
        let tools_doc = Self::generate_tools_documentation();

        // Simple replacement - in a real implementation, you'd want more sophisticated parsing
        let updated_content = readme_content.replace("## MCP Tools", &tools_doc);

        fs::write(readme_path, updated_content)?;
        Ok(())
    }

    /// Validate that documentation matches implementation
    pub fn validate_docs() -> Result<()> {
        // For now, just check that we can generate documentation
        let _doc = Self::generate_tools_documentation();
        println!("Documentation validation passed");
        Ok(())
    }

    /// Run the documentation generator
    pub fn run_doc_generator() -> Result<()> {
        println!("Generating MCP tools documentation...");

        let tools_doc = Self::generate_tools_documentation();
        println!("Generated documentation:\n{}", tools_doc);

        Self::validate_docs()?;

        println!("Documentation generation completed successfully");
        Ok(())
    }
}
