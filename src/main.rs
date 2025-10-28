use subagent_worktree_mcp::run_server;

#[tokio::main]
async fn main() {
    if let Err(e) = run_server().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
