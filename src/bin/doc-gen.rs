use anyhow::Result;
use clap::{Parser, Subcommand};

use subagent_worktree_mcp::doc_generator::{DocGenerator, run_doc_generator};

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
            
            let generator = DocGenerator::new();
            
            // Validate implementation first
            generator.validate_implementation()?;
            
            // Update README
            generator.update_readme(std::path::Path::new(&readme))?;
            
            if report {
                let report_content = generator.generate_schema_report();
                std::fs::write("SCHEMA_REPORT.md", report_content)?;
                println!("📊 Generated SCHEMA_REPORT.md");
            }
            
            println!("✅ Documentation update complete!");
        }
        
        Commands::Validate => {
            println!("🔍 Validating documentation against implementation...");
            
            let generator = DocGenerator::new();
            generator.validate_implementation()?;
            
            println!("✅ All documentation is valid!");
        }
        
        Commands::Report { output } => {
            println!("📊 Generating schema report...");
            
            let generator = DocGenerator::new();
            let report_content = generator.generate_schema_report();
            
            std::fs::write(&output, report_content)?;
            println!("✅ Schema report generated: {}", output);
        }
        
        Commands::List => {
            println!("📋 Current tool definitions:");
            
            let generator = DocGenerator::new();
            
            for tool in &generator.tools {
                println!("\n🔧 {}", tool.name);
                println!("   Description: {}", tool.description);
                println!("   Destructive: {}", tool.is_destructive);
                println!("   Parameters: {} total", tool.parameters.len());
                
                for param in &tool.parameters {
                    let required = if param.required { "required" } else { "optional" };
                    println!("     - {}: {} ({})", param.name, required, param.param_type);
                }
                
                if !tool.warnings.is_empty() {
                    println!("   Warnings:");
                    for warning in &tool.warnings {
                        println!("     - {}", warning);
                    }
                }
            }
        }
    }
    
    Ok(())
}
