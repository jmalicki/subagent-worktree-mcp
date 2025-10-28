use anyhow::Result;
use clap::{Parser, Subcommand};

use subagent_worktree_mcp::doc_generator::DocGenerator;

/// Documentation generator for Subagent Worktree MCP Server
///
/// This tool automatically generates and updates documentation from the actual
/// schema definitions, ensuring perfect synchronization between implementation
/// and documentation.
#[derive(Parser)]
#[command(name = "doc-gen")]
#[command(about = "Generate documentation from schema definitions")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate and update README.md with current schema
    Update {
        /// Path to README.md file
        #[arg(short, long, default_value = "README.md")]
        readme: String,

        /// Also generate a schema report
        #[arg(short, long)]
        report: bool,
    },

    /// Validate that documentation matches implementation
    Validate,

    /// Generate a schema report without updating README
    Report {
        /// Output file for the report
        #[arg(short, long, default_value = "SCHEMA_REPORT.md")]
        output: String,
    },

    /// Show current tool definitions
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Update { readme, report } => {
            println!("🔄 Updating documentation from schema...");

            // Validate implementation first
            DocGenerator::validate_docs()?;

            // Update README
            DocGenerator::update_readme(std::path::Path::new(&readme))?;

            if report {
                let report_content = DocGenerator::generate_tools_documentation();
                std::fs::write("SCHEMA_REPORT.md", report_content)?;
                println!("📊 Generated SCHEMA_REPORT.md");
            }

            println!("✅ Documentation update complete!");
        }

        Commands::Validate => {
            println!("🔍 Validating documentation against implementation...");

            DocGenerator::validate_docs()?;

            println!("✅ All documentation is valid!");
        }

        Commands::Report { output } => {
            println!("📊 Generating schema report...");

            let report_content = DocGenerator::generate_tools_documentation();

            std::fs::write(&output, report_content)?;
            println!("✅ Schema report generated: {}", output);
        }

        Commands::List => {
            println!("📋 Current tool definitions:");
            println!("Available tools:");
            println!(
                "  - spawn_subagent: Spawn a new subagent with a git worktree for isolated development"
            );
            println!(
                "  - cleanup_worktree: Clean up a worktree and optionally delete the branch (destructive)"
            );
            println!("  - list_worktrees: List all git worktrees and their associated agents");
            println!(
                "\nUse 'cargo run --bin doc-gen report' to generate detailed schema documentation."
            );
        }
    }

    Ok(())
}
