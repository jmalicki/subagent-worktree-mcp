# Subagent Worktree MCP Server

A Model Context Protocol (MCP) server for spawning subagents with git worktrees. This tool allows you to create isolated development environments for different tasks while maintaining proper git workflow.

## Features

- **Git Worktree Management**: Create and manage git worktrees for isolated development
- **Multi-Agent Support**: Spawn different types of agents (cursor-cli, VS Code, vim, etc.)
- **Process Monitoring**: Track running agents and their status
- **Extensible Architecture**: Easy to add new agent types
- **MCP Protocol**: Standardized interface for AI agent communication

## Quick Start

### Prerequisites

- Rust 1.90.0+ (2024 Edition)
- Git
- One or more supported agents (cursor-cli, VS Code, etc.)
- MCP-compatible AI agent (Claude Desktop, Cursor, etc.)

### Installation

```bash
git clone <repository-url>
cd subagent-worktree-mcp
cargo build --release
```

### MCP Integration

1. **Configure your MCP client** (e.g., Claude Desktop) to use this server:

```json
{
  "mcpServers": {
    "subagent-worktree": {
      "command": "cargo",
      "args": ["run", "--release"],
      "cwd": "/path/to/subagent-worktree-mcp"
    }
  }
}
```

2. **Use the tools** through your MCP client to spawn subagents and manage worktrees.

### Direct Testing (Development)

```bash
# Run the MCP server directly for testing
cargo run

# Or build and run the release version
cargo build --release
./target/release/subagent-worktree-mcp
```

## Architecture

### Core Components

1. **GitWorktreeManager**: Handles git worktree creation and management
2. **SubagentSpawner**: Manages spawning of different agent types
3. **AgentMonitor**: Monitors running agent processes
4. **AgentSpawner Trait**: Extensible interface for different agent types

### Supported Agents

- **cursor-cli**: Cursor AI-powered editor
- **VS Code**: Visual Studio Code
- **Vim/Neovim**: Terminal-based editors
- **Extensible**: Easy to add new agent types

## MCP Protocol Usage

This server implements the Model Context Protocol (MCP) and provides tools for AI agents to manage git worktrees and spawn subagents.

### MCP Server Configuration

```json
{
  "mcpServers": {
    "subagent-worktree": {
      "command": "cargo",
      "args": ["run", "--release"],
      "cwd": "/path/to/subagent-worktree-mcp"
    }
  }
}
```

### Example MCP Tool Calls

#### Spawn a Subagent
```json
{
  "method": "tools/call",
  "params": {
    "name": "spawn_subagent",
    "arguments": {
      "branch_name": "feature/new-feature",
      "prompt": "Implement user authentication system",
      "base_branch": "main",
      "agent_type": "cursor-cli",
      "agent_options": {
        "new_window": true,
        "detach": false
      }
    }
  }
}
```

#### Monitor Running Agents
```json
{
  "method": "tools/call",
  "params": {
    "name": "monitor_agents",
    "arguments": {
      "only_our_agents": true,
      "only_waiting_agents": false
    }
  }
}
```

## MCP Tools

### `spawn_subagent`

Spawn a new subagent with a git worktree.

**Parameters:**
- `branch_name` (required): Name of the branch to create
- `prompt` (required): Initial prompt for the subagent
- `base_branch` (optional): Base branch to create from
- `worktree_dir` (optional): Custom worktree directory name
- `agent_type` (optional): Type of agent to spawn (default: "cursor-cli")
- `agent_options` (optional): Agent-specific options

### `monitor_agents`

Monitor running agent processes.

**Parameters:**
- `only_our_agents` (optional): Only show agents we spawned
- `only_waiting_agents` (optional): Only show agents waiting for input
- `agent_types` (optional): Filter by agent types
- `worktree_paths` (optional): Filter by worktree paths

### `cleanup_worktree` ⚠️ **DESTRUCTIVE**

Clean up a worktree and optionally kill running agents and remove the branch.

**Parameters:**
- `worktree_name` (required): Name of the worktree/branch to clean up
- `force` (optional): Force cleanup even if agents are still running
- `remove_branch` (optional): Remove the git branch after cleanup
- `kill_agents` (optional): Kill running agents before cleanup

**⚠️ Warning:** This tool is destructive and will:
- Kill running agent processes
- Remove the worktree directory
- Optionally delete the git branch
- Cannot be undone

### `list_worktrees`

List all worktrees and their current status.

**Parameters:** None

**Returns:** Information about all worktrees including paths, branches, and commits

## Development

### Project Structure

```
src/
├── main.rs              # Main MCP server implementation
├── git_operations.rs    # Git worktree management
├── subagent_spawner.rs  # Agent spawning and management
└── agent_monitor.rs     # Process monitoring

tests/
└── integration_tests.rs # Comprehensive test suite
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_create_worktree_basic
```

### Code Quality

This project uses:
- **Rust 2024 Edition**: Latest language features
- **Clippy**: Linting and code quality
- **Rustfmt**: Code formatting
- **Pre-commit hooks**: Automated quality checks

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes
4. Run tests: `cargo test`
5. Run clippy: `cargo clippy`
6. Format code: `cargo fmt`
7. Commit with conventional commits: `git commit -m "feat: add new feature"`
8. Push and create a pull request

### Conventional Commits

This project follows [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `style:` Code style changes
- `refactor:` Code refactoring
- `test:` Test additions/changes
- `chore:` Maintenance tasks

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Roadmap

- [ ] Full MCP server implementation
- [ ] Web UI for agent management
- [ ] Agent communication protocols
- [ ] Resource usage monitoring
- [ ] Agent templates and presets
- [ ] Integration with popular IDEs

## Troubleshooting

### Common Issues

1. **Git worktree creation fails**: Ensure you're in a git repository
2. **Agent not found**: Check if the agent is installed and in PATH
3. **Permission denied**: Ensure proper file permissions for worktree directories

### Debug Mode

Run with debug logging:

```bash
RUST_LOG=debug cargo run
```

## Support

- Create an issue for bug reports
- Start a discussion for questions
- Check existing issues for solutions

---

Built with ❤️ using Rust and the MCP protocol.
