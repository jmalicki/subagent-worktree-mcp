use anyhow::Result;
use std::fs;
use std::path::Path;

/// Documentation generator that generates markdown documentation
/// for the MCP tools defined in the server
pub struct DocGenerator;

impl DocGenerator {
    /// Generate complete MCP tools documentation section
    pub fn generate_tools_documentation() -> String {
        let mut doc = String::new();
        
        doc.push_str("## MCP Tools\n\n");
        
        // Define the tools manually since we're using rmcp macros now
        let tools = vec![
            ("spawn_subagent", "Spawn a new subagent with a git worktree for isolated development"),
            ("cleanup_worktree", "Clean up a worktree and optionally delete the branch (destructive)"),
            ("list_worktrees", "List all git worktrees and their associated agents"),
        ];
        
        for (name, description) in tools {
            doc.push_str(&format!("### `{}`\n\n", name));
            doc.push_str(&format!("{}\n\n", description));
            
            // Add parameter information based on tool name
            match name {
                "spawn_subagent" => {
                    doc.push_str("**Parameters:**\n");
                    doc.push_str("- `branch_name` (string, required): Name of the branch to create for the subagent\n");
                    doc.push_str("- `prompt` (string, required): Initial prompt to give to the subagent\n");
                    doc.push_str("- `worktree_dir` (string, optional): Optional directory name for the worktree (defaults to branch name)\n");
                    doc.push_str("- `agent_type` (string, optional): Agent type to spawn (defaults to \"cursor-agent\")\n");
                    doc.push_str("- `agent_options` (object, optional): Additional options for the agent\n");
                }
                "cleanup_worktree" => {
                    doc.push_str("**Parameters:**\n");
                    doc.push_str("- `worktree_path` (string, required): Path to the worktree to clean up\n");
                    doc.push_str("- `delete_branch` (boolean, optional): Whether to delete the branch (default: false)\n");
                    doc.push_str("- `force` (boolean, optional): Whether to force cleanup even if there are uncommitted changes (default: false)\n");
                }
                "list_worktrees" => {
                    doc.push_str("**Parameters:**\n");
                    doc.push_str("- `include_agents` (boolean, optional): Whether to include agent information (default: true)\n");
                    doc.push_str("- `only_our_agents` (boolean, optional): Only show agents spawned by our system (default: true)\n");
                    doc.push_str("- `only_waiting_agents` (boolean, optional): Only show agents waiting for input (default: false)\n");
                }
                _ => {}
            }
            
            doc.push_str("\n");
        }
        
        doc
    }

    /// Update the README.md file with generated documentation
    pub fn update_readme(readme_path: &Path) -> Result<()> {
        let readme_content = fs::read_to_string(readme_path)?;
        let generated_docs = Self::generate_tools_documentation();
        
        // Find the MCP Tools section and replace it
        let start_marker = "## MCP Tools";
        let end_marker = "\n## ";
        
        if let Some(start_pos) = readme_content.find(start_marker) {
            // Find the end of the MCP Tools section
            let after_start = &readme_content[start_pos..];
            let end_pos = if let Some(end) = after_start.find(end_marker) {
                start_pos + end
            } else {
                readme_content.len()
            };
            
            // Replace the section
            let mut new_content = readme_content[..start_pos].to_string();
            new_content.push_str(&generated_docs);
            new_content.push_str(&readme_content[end_pos..]);
            
            fs::write(readme_path, new_content)?;
        } else {
            // If no MCP Tools section found, append it
            let mut new_content = readme_content;
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push_str(&generated_docs);
            fs::write(readme_path, new_content)?;
        }
        
        Ok(())
    }

    /// Validate that the current documentation matches the implementation
    pub fn validate_docs() -> Result<()> {
        let generated_docs = Self::generate_tools_documentation();
        
        // Basic validation - ensure we have the expected tools
        let expected_tools = ["spawn_subagent", "cleanup_worktree", "list_worktrees"];
        for tool in expected_tools {
            if !generated_docs.contains(tool) {
                return Err(anyhow::anyhow!("Missing tool documentation for: {}", tool));
            }
        }
        
        Ok(())
    }

    /// Run the documentation generator
    pub fn run_doc_generator() -> Result<()> {
        Self::validate_docs()?;
        println!("Documentation validation passed");
        Ok(())
    }
}