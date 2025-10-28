use anyhow::Result;
use subagent_worktree_mcp::run_server;

#[tokio::main]
async fn main() -> Result<()> {
    run_server().await
}