use anyhow::Result;
use mcp::types::Tool;
use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::server::SubagentWorktreeServer;

/// Documentation generator that uses the MCP server's actual tool definitions
/// to generate markdown documentation
pub struct DocGenerator;

impl DocGenerator {
    /// Generate complete MCP tools documentation section
    pub fn generate_tools_documentation() -> String {
        let mut doc = String::new();
        
        doc.push_str("## MCP Tools\n\n");
        
        let tools = SubagentWorktreeServer::get_tools();
        for tool in &tools {
            doc.push_str(&Self::generate_tool_documentation(tool));
            doc.push_str("\n");
        }
        
        doc
    }

    /// Generate documentation for a single MCP tool
    fn generate_tool_documentation(tool: &Tool) -> String {
        let mut doc = String::new();
        
        // Tool header
        doc.push_str(&format!("### `{}`\n\n", tool.name));

        // Description
        if let Some(description) = &tool.description {
            doc.push_str(&format!("{}\n\n", description));
        }

        // Parameters from JSON schema
        if let mcp::types::ToolInputSchema::JsonSchema(schema) = &tool.input_schema {
            if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
                if !properties.is_empty() {
                    doc.push_str("**Parameters:**\n");
                    
                    let required_fields = schema.get("required")
                        .and_then(|r| r.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<std::collections::HashSet<_>>())
                        .unwrap_or_default();

                    for (name, prop) in properties {
                        if let Some(prop_obj) = prop.as_object() {
                            let description = prop_obj.get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("No description available");
                            
                            let param_type = prop_obj.get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("unknown");
                            
                            let required = required_fields.contains(name.as_str());
                            
                            doc.push_str(&format!(
                                "- `{}`: {} ({}, {})\n",
                                name,
                                description,
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

    /// Update the README.md with generated documentation
    pub fn update_readme(readme_path: &Path) -> Result<()> {
        let mut readme_content = fs::read_to_string(readme_path)?;
        let new_docs = Self::generate_tools_documentation();

        let start_tag = "<!-- MCP_TOOLS_START -->";
        let end_tag = "<!-- MCP_TOOLS_END -->";

        if let Some(start_idx) = readme_content.find(start_tag) {
            if let Some(end_idx) = readme_content.find(end_tag) {
                let before = &readme_content[..start_idx + start_tag.len()];
                let after = &readme_content[end_idx..];
                readme_content = format!("{}\n{}\n{}", before, new_docs, after);
            }
        }

        fs::write(readme_path, readme_content)?;
        Ok(())
    }

    /// Validate that the README documentation matches the MCP server implementation
    pub fn validate_docs(readme_path: &Path) -> Result<bool> {
        let readme_content = fs::read_to_string(readme_path)?;
        let generated_docs = Self::generate_tools_documentation();

        let start_tag = "<!-- MCP_TOOLS_START -->";
        let end_tag = "<!-- MCP_TOOLS_END -->";

        if let Some(start_idx) = readme_content.find(start_tag) {
            if let Some(end_idx) = readme_content.find(end_tag) {
                let existing_docs_start = start_idx + start_tag.len();
                let existing_docs_end = end_idx;
                let existing_docs = &readme_content[existing_docs_start..existing_docs_end].trim();

                if existing_docs == generated_docs.trim() {
                    println!("✅ Documentation matches implementation.");
                    return Ok(true);
                } else {
                    println!("❌ Documentation mismatch!");
                    println!("--- Expected (Generated) ---\n{}\n----------------------------", generated_docs.trim());
                    println!("--- Actual (README) ---\n{}\n-------------------------", existing_docs);
                    return Ok(false);
                }
            }
        }
        println!("⚠️ Documentation tags not found in README.md. Please add `<!-- MCP_TOOLS_START -->` and `<!-- MCP_TOOLS_END -->`.");
        Ok(false)
    }

    /// List all tools and their schemas for debugging
    pub fn list_tools() {
        let tools = SubagentWorktreeServer::get_tools();
        println!("📋 Current MCP tool definitions:\n");
        
        for tool in &tools {
            println!("🔧 {}", tool.name);
            if let Some(description) = &tool.description {
                println!("   Description: {}", description);
            }
            
            if let mcp::types::ToolInputSchema::JsonSchema(schema) = &tool.input_schema {
                if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
                    println!("   Parameters: {} total", properties.len());
                    for (name, prop) in properties {
                        if let Some(prop_obj) = prop.as_object() {
                            let description = prop_obj.get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("No description");
                            let param_type = prop_obj.get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("unknown");
                            println!("     - {}: {} ({})", name, description, param_type);
                        }
                    }
                }
            }
            
            println!("");
        }
    }
}

/// CLI function for the doc-gen binary
pub fn run_doc_generator(command: &str, readme_path: &Path) -> Result<()> {
    match command {
        "update" => {
            DocGenerator::update_readme(readme_path)?;
            println!("README.md updated successfully.");
        }
        "validate" => {
            if DocGenerator::validate_docs(readme_path)? {
                println!("Documentation is valid.");
            } else {
                println!("Documentation is invalid.");
                std::process::exit(1);
            }
        }
        "list" => {
            DocGenerator::list_tools();
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }
    Ok(())
}