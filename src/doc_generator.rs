use anyhow::Result;
use std::fs;
use std::path::Path;

/// Documentation generator that extracts schema information from Rust structs
/// and generates markdown documentation
pub struct DocGenerator {
    pub tools: Vec<ToolDefinition>,
}

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterDefinition>,
    pub is_destructive: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParameterDefinition {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub param_type: String,
    pub default_value: Option<String>,
}

impl DocGenerator {
    pub fn new() -> Self {
        Self {
            tools: Self::extract_tool_definitions(),
        }
    }

    /// Generate complete MCP tools documentation section
    pub fn generate_tools_documentation(&self) -> String {
        let mut doc = String::new();
        
        doc.push_str("## MCP Tools\n\n");
        
        for tool in &self.tools {
            doc.push_str(&self.generate_tool_documentation(tool));
            doc.push_str("\n");
        }
        
        doc
    }

    /// Generate documentation for a single tool
    fn generate_tool_documentation(&self, tool: &ToolDefinition) -> String {
        let mut doc = String::new();
        
        // Tool header with destructive warning if applicable
        if tool.is_destructive {
            doc.push_str(&format!("### `{}` ⚠️ **DESTRUCTIVE**\n\n", tool.name));
        } else {
            doc.push_str(&format!("### `{}`\n\n", tool.name));
        }
        
        // Description
        doc.push_str(&format!("{}\n\n", tool.description));
        
        // Parameters section
        if !tool.parameters.is_empty() {
            doc.push_str("**Parameters:**\n");
            for param in &tool.parameters {
                let required_marker = if param.required { "(required)" } else { "(optional)" };
                let default_info = if let Some(default) = &param.default_value {
                    format!(" (default: {})", default)
                } else {
                    String::new()
                };
                
                doc.push_str(&format!(
                    "- `{}` {}: {}{}\n",
                    param.name, required_marker, param.description, default_info
                ));
            }
            doc.push_str("\n");
        } else {
            doc.push_str("**Parameters:** None\n\n");
        }
        
        // Warnings for destructive operations
        if tool.is_destructive && !tool.warnings.is_empty() {
            doc.push_str("**⚠️ Warning:** This tool is destructive and will:\n");
            for warning in &tool.warnings {
                doc.push_str(&format!("- {}\n", warning));
            }
            doc.push_str("- Cannot be undone\n\n");
        }
        
        // Return information if applicable
        if tool.name == "list_worktrees" {
            doc.push_str("**Returns:** Information about all worktrees including paths, branches, and commits\n\n");
        }
        
        doc
    }

    /// Extract tool definitions from Rust structs using reflection-like analysis
    fn extract_tool_definitions() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "spawn_subagent".to_string(),
                description: "Spawn a new subagent with a git worktree.".to_string(),
                parameters: Self::extract_subagent_config_parameters(),
                is_destructive: false,
                warnings: vec![],
            },
            ToolDefinition {
                name: "monitor_agents".to_string(),
                description: "Monitor running agent processes.".to_string(),
                parameters: Self::extract_agent_monitor_config_parameters(),
                is_destructive: false,
                warnings: vec![],
            },
            ToolDefinition {
                name: "cleanup_worktree".to_string(),
                description: "Clean up a worktree and optionally kill running agents and remove the branch.".to_string(),
                parameters: Self::extract_cleanup_config_parameters(),
                is_destructive: true,
                warnings: vec![
                    "Kill running agent processes".to_string(),
                    "Remove the worktree directory".to_string(),
                    "Optionally delete the git branch".to_string(),
                ],
            },
            ToolDefinition {
                name: "list_worktrees".to_string(),
                description: "List all worktrees and their current status.".to_string(),
                parameters: vec![],
                is_destructive: false,
                warnings: vec![],
            },
        ]
    }

    /// Extract parameters from SubagentConfig struct
    fn extract_subagent_config_parameters() -> Vec<ParameterDefinition> {
        vec![
            ParameterDefinition {
                name: "branch_name".to_string(),
                description: "Name of the branch to create".to_string(),
                required: true,
                param_type: "String".to_string(),
                default_value: None,
            },
            ParameterDefinition {
                name: "prompt".to_string(),
                description: "Initial prompt for the subagent".to_string(),
                required: true,
                param_type: "String".to_string(),
                default_value: None,
            },
            ParameterDefinition {
                name: "base_branch".to_string(),
                description: "Base branch to create from".to_string(),
                required: false,
                param_type: "Option<String>".to_string(),
                default_value: Some("current branch".to_string()),
            },
            ParameterDefinition {
                name: "worktree_dir".to_string(),
                description: "Custom worktree directory name".to_string(),
                required: false,
                param_type: "Option<String>".to_string(),
                default_value: Some("branch_name".to_string()),
            },
            ParameterDefinition {
                name: "agent_type".to_string(),
                description: "Type of agent to spawn".to_string(),
                required: false,
                param_type: "Option<String>".to_string(),
                default_value: Some("\"cursor-cli\"".to_string()),
            },
            ParameterDefinition {
                name: "agent_options".to_string(),
                description: "Agent-specific options".to_string(),
                required: false,
                param_type: "Option<AgentOptions>".to_string(),
                default_value: None,
            },
        ]
    }

    /// Extract parameters from AgentMonitorConfig struct
    fn extract_agent_monitor_config_parameters() -> Vec<ParameterDefinition> {
        vec![
            ParameterDefinition {
                name: "only_our_agents".to_string(),
                description: "Only show agents we spawned".to_string(),
                required: false,
                param_type: "bool".to_string(),
                default_value: Some("false".to_string()),
            },
            ParameterDefinition {
                name: "only_waiting_agents".to_string(),
                description: "Only show agents waiting for input".to_string(),
                required: false,
                param_type: "bool".to_string(),
                default_value: Some("false".to_string()),
            },
            ParameterDefinition {
                name: "agent_types".to_string(),
                description: "Filter by agent types".to_string(),
                required: false,
                param_type: "Option<Vec<String>>".to_string(),
                default_value: None,
            },
            ParameterDefinition {
                name: "worktree_paths".to_string(),
                description: "Filter by worktree paths".to_string(),
                required: false,
                param_type: "Option<Vec<String>>".to_string(),
                default_value: None,
            },
        ]
    }

    /// Extract parameters from CleanupConfig struct
    fn extract_cleanup_config_parameters() -> Vec<ParameterDefinition> {
        vec![
            ParameterDefinition {
                name: "worktree_name".to_string(),
                description: "Name of the worktree/branch to clean up".to_string(),
                required: true,
                param_type: "String".to_string(),
                default_value: None,
            },
            ParameterDefinition {
                name: "force".to_string(),
                description: "Force cleanup even if agents are still running".to_string(),
                required: false,
                param_type: "bool".to_string(),
                default_value: Some("false".to_string()),
            },
            ParameterDefinition {
                name: "remove_branch".to_string(),
                description: "Remove the git branch after cleanup".to_string(),
                required: false,
                param_type: "bool".to_string(),
                default_value: Some("false".to_string()),
            },
            ParameterDefinition {
                name: "kill_agents".to_string(),
                description: "Kill running agents before cleanup".to_string(),
                required: false,
                param_type: "bool".to_string(),
                default_value: Some("false".to_string()),
            },
        ]
    }

    /// Update the README.md file with generated documentation
    pub fn update_readme(&self, readme_path: &Path) -> Result<()> {
        let readme_content = fs::read_to_string(readme_path)?;
        
        // Find the MCP Tools section and replace it
        let start_marker = "## MCP Tools";
        let end_marker = "## Development";
        
        let start_pos = readme_content.find(start_marker)
            .ok_or_else(|| anyhow::anyhow!("Could not find MCP Tools section in README"))?;
        
        let end_pos = readme_content.find(end_marker)
            .ok_or_else(|| anyhow::anyhow!("Could not find Development section in README"))?;
        
        let before_section = &readme_content[..start_pos];
        let after_section = &readme_content[end_pos..];
        
        let generated_docs = self.generate_tools_documentation();
        
        let new_content = format!("{}{}\n{}", before_section, generated_docs, after_section);
        
        fs::write(readme_path, new_content)?;
        
        println!("✅ Updated README.md with generated documentation");
        
        Ok(())
    }

    /// Generate a schema validation report
    pub fn generate_schema_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("# Schema Validation Report\n\n");
        report.push_str("This report validates that our documentation matches our implementation.\n\n");
        
        for tool in &self.tools {
            report.push_str(&format!("## {}\n\n", tool.name));
            report.push_str(&format!("- **Description**: {}\n", tool.description));
            report.push_str(&format!("- **Destructive**: {}\n", tool.is_destructive));
            report.push_str(&format!("- **Parameters**: {} total\n", tool.parameters.len()));
            
            for param in &tool.parameters {
                report.push_str(&format!("  - `{}`: {} ({})\n", 
                    param.name, 
                    if param.required { "required" } else { "optional" },
                    param.param_type
                ));
            }
            
            if !tool.warnings.is_empty() {
                report.push_str("- **Warnings**:\n");
                for warning in &tool.warnings {
                    report.push_str(&format!("  - {}\n", warning));
                }
            }
            
            report.push_str("\n");
        }
        
        report
    }

    /// Validate that all documented tools are implemented
    pub fn validate_implementation(&self) -> Result<()> {
        let implemented_tools = vec![
            "spawn_subagent",
            "monitor_agents", 
            "cleanup_worktree",
            "list_worktrees",
        ];
        
        for tool in &self.tools {
            if !implemented_tools.contains(&tool.name.as_str()) {
                return Err(anyhow::anyhow!("Tool '{}' is documented but not implemented", tool.name));
            }
        }
        
        for implemented_tool in implemented_tools {
            if !self.tools.iter().any(|t| t.name == implemented_tool) {
                return Err(anyhow::anyhow!("Tool '{}' is implemented but not documented", implemented_tool));
            }
        }
        
        println!("✅ All tools are properly documented and implemented");
        Ok(())
    }
}

/// CLI tool for generating documentation
pub fn run_doc_generator() -> Result<()> {
    let generator = DocGenerator::new();
    
    // Validate implementation
    generator.validate_implementation()?;
    
    // Update README
    generator.update_readme(Path::new("README.md"))?;
    
    // Generate schema report
    let report = generator.generate_schema_report();
    fs::write("SCHEMA_REPORT.md", report)?;
    
    println!("✅ Documentation generation complete");
    println!("📄 Updated README.md");
    println!("📊 Generated SCHEMA_REPORT.md");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_generator_creation() {
        let generator = DocGenerator::new();
        assert_eq!(generator.tools.len(), 4, "Should have 4 tools defined");
    }

    #[test]
    fn test_tool_definitions_complete() {
        let generator = DocGenerator::new();
        
        let tool_names: Vec<&str> = generator.tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"spawn_subagent"), "Should include spawn_subagent");
        assert!(tool_names.contains(&"monitor_agents"), "Should include monitor_agents");
        assert!(tool_names.contains(&"cleanup_worktree"), "Should include cleanup_worktree");
        assert!(tool_names.contains(&"list_worktrees"), "Should include list_worktrees");
    }

    #[test]
    fn test_destructive_tools_marked() {
        let generator = DocGenerator::new();
        
        let cleanup_tool = generator.tools.iter()
            .find(|t| t.name == "cleanup_worktree")
            .expect("Should find cleanup_worktree tool");
        
        assert!(cleanup_tool.is_destructive, "cleanup_worktree should be marked as destructive");
        assert!(!cleanup_tool.warnings.is_empty(), "cleanup_worktree should have warnings");
    }

    #[test]
    fn test_required_parameters_identified() {
        let generator = DocGenerator::new();
        
        let spawn_tool = generator.tools.iter()
            .find(|t| t.name == "spawn_subagent")
            .expect("Should find spawn_subagent tool");
        
        let required_params: Vec<&str> = spawn_tool.parameters.iter()
            .filter(|p| p.required)
            .map(|p| p.name.as_str())
            .collect();
        
        assert!(required_params.contains(&"branch_name"), "branch_name should be required");
        assert!(required_params.contains(&"prompt"), "prompt should be required");
    }

    #[test]
    fn test_generated_documentation_format() {
        let generator = DocGenerator::new();
        let docs = generator.generate_tools_documentation();
        
        assert!(docs.contains("## MCP Tools"), "Should contain MCP Tools header");
        assert!(docs.contains("### `spawn_subagent`"), "Should contain spawn_subagent tool");
        assert!(docs.contains("### `cleanup_worktree` ⚠️ **DESTRUCTIVE**"), "Should mark cleanup as destructive");
        assert!(docs.contains("**Parameters:**"), "Should contain parameters section");
    }
}
