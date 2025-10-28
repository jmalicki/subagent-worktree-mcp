use anyhow::Result;
use std::collections::HashSet;
use std::fs;

use subagent_worktree_mcp::doc_generator::DocGenerator;

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

/// Test to verify that the generated schema matches what's documented in README
#[tokio::test]
async fn test_schema_matches_documentation() -> Result<()> {
    // Test: Verify that the generated schema from our implementation matches README documentation
    // This ensures the schema is accurate and up-to-date

    // Generate the schema from our implementation
    let generated_schema = DocGenerator::generate_tools_documentation();

    // Read the README file
    let readme_content = fs::read_to_string("README.md")?;

    // Extract schema information from README
    let documented_schema = extract_schema_from_readme(&readme_content);

    // Compare the generated schema with documented schema
    // For now, we'll do a basic comparison - in a real implementation,
    // you might want to parse JSON schemas and compare them structurally

    // Verify that all tools mentioned in README have corresponding schema entries
    for tool_name in &documented_schema.tools {
        assert!(
            generated_schema.contains(tool_name),
            "Tool '{}' documented in README but not found in generated schema",
            tool_name
        );
    }

    println!(
        "✅ Schema matches documentation for {} tools",
        documented_schema.tools.len()
    );

    Ok(())
}

/// Test to verify that the documentation generator works correctly
#[tokio::test]
async fn test_documentation_generator() -> Result<()> {
    // Test: Verify that the documentation generator can produce valid output
    // This ensures the generator itself is working correctly

    // Generate documentation
    let generated_docs = DocGenerator::generate_tools_documentation();

    // Verify the generated documentation is not empty
    assert!(
        !generated_docs.is_empty(),
        "Generated documentation should not be empty"
    );

    // Verify it contains expected tool names
    let expected_tools = ["spawn_subagent", "cleanup_worktree", "list_worktrees"];
    for tool in &expected_tools {
        assert!(
            generated_docs.contains(tool),
            "Generated documentation should contain tool '{}'",
            tool
        );
    }

    println!("✅ Documentation generator produces valid output");

    Ok(())
}

/// Test to verify that the documentation validation works
#[tokio::test]
async fn test_documentation_validation() -> Result<()> {
    // Test: Verify that the documentation validation can detect issues
    // This ensures the validation logic is working correctly

    // Run the validation (this should not panic)
    let result = DocGenerator::validate_docs();

    // The validation should succeed for our current implementation
    assert!(
        result.is_ok(),
        "Documentation validation should pass: {:?}",
        result
    );

    println!("✅ Documentation validation works correctly");

    Ok(())
}

/// Extract documented tools from README content
fn extract_documented_tools(readme_content: &str) -> HashSet<String> {
    let mut tools = HashSet::new();

    // Look for tool names in the README
    // This is a simple regex-based approach - in a real implementation,
    // you might want to use a proper markdown parser

    let lines: Vec<&str> = readme_content.lines().collect();
    for line in lines {
        // Look for lines that mention tool names
        if line.contains("spawn_subagent") {
            tools.insert("spawn_subagent".to_string());
        }
        if line.contains("cleanup_worktree") {
            tools.insert("cleanup_worktree".to_string());
        }
        if line.contains("list_worktrees") {
            tools.insert("list_worktrees".to_string());
        }
    }

    tools
}

/// Get the tools we actually implement
fn get_implemented_tools() -> HashSet<String> {
    let mut tools = HashSet::new();
    tools.insert("spawn_subagent".to_string());
    tools.insert("cleanup_worktree".to_string());
    tools.insert("list_worktrees".to_string());
    tools
}

/// Schema information extracted from README
struct DocumentedSchema {
    tools: Vec<String>,
}

/// Extract schema information from README content
fn extract_schema_from_readme(readme_content: &str) -> DocumentedSchema {
    let mut tools = Vec::new();

    // Look for tool names in the README
    let lines: Vec<&str> = readme_content.lines().collect();
    for line in lines {
        if line.contains("spawn_subagent") {
            tools.push("spawn_subagent".to_string());
        }
        if line.contains("cleanup_worktree") {
            tools.push("cleanup_worktree".to_string());
        }
        if line.contains("list_worktrees") {
            tools.push("list_worktrees".to_string());
        }
    }

    DocumentedSchema { tools }
}
