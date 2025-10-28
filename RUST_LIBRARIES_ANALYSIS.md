# Rust Libraries for MCP Server and Agent Abstraction

## Overview

This document summarizes existing Rust libraries that could be useful for building an extensible MCP (Model Context Protocol) server that spawns subagents with git worktrees. The analysis covers agent frameworks, process management, git operations, and MCP protocol implementations.

## MCP Server Libraries

### 1. agentic (Highly Recommended)
- **Purpose**: Dedicated Rust library for agentic applications with MCP and A2A support
- **Key Features**:
  - **MCP Client and Server Support** - Native MCP protocol implementation
  - **Integration with `rpc-router`** - Efficient JSON-RPC routing
  - **Planned A2A Support** - Future Agent-to-Agent communication
  - **Planned `genai` integration** - Multi-AI provider support
- **Current Version**: 0.0.3 (under active development)
- **Use Case**: **Perfect for our MCP server implementation**
- **Maturity**: Early stage but focused on MCP
- **Repository**: [docs.rs/agentic](https://docs.rs/agentic)
- **Recommendation**: **Highly Recommended** - This is exactly what we need!

### 2. Current `mcp` crate
- **Purpose**: Basic MCP protocol implementation
- **Status**: Currently using this
- **Recommendation**: Consider migrating to `agentic` for better MCP support

## Agent Framework Libraries

### 1. Radkit (Rust Agent Development Kit)
- **Purpose**: Full-featured agent framework with A2A Protocol support
- **Key Features**: 
  - Native A2A protocol support for agent interoperability
  - Multi-provider LLM support (Anthropic, Google Gemini)
  - Advanced tool system with built-in task management
  - Zero conversion overhead for agent communication
- **Use Case**: Could replace our custom agent abstraction entirely
- **Maturity**: Production-ready framework
- **Repository**: [microagents.github.io/radkit](https://microagents.github.io/radkit/)
- **Recommendation**: Consider for future LLM-powered agent features

### 2. AgentAI
- **Purpose**: Simplified AI agent creation with LLM integration
- **Key Features**:
  - Support for major LLM providers (OpenAI, Anthropic, Gemini, Ollama)
  - Custom tool creation via `ToolBox`
  - Flexible model selection for different tasks
- **Use Case**: Good for LLM-powered agents, but may be overkill for simple process spawning
- **Maturity**: Under heavy development (API may change)
- **Repository**: [docs.rs/agentai](https://docs.rs/agentai)
- **Recommendation**: Monitor for future LLM integration needs

### 3. AutoAgents
- **Purpose**: Multi-agent framework with Ractor actor system
- **Key Features**:
  - ReAct and Basic executors with streaming support
  - Structured outputs with JSON schema validation
  - WebAssembly support for browser-based agents
  - Multi-platform deployment capabilities
- **Use Case**: Good for complex multi-agent coordination
- **Maturity**: Production-ready
- **Repository**: [github.com/liquidos-ai/AutoAgents](https://github.com/liquidos-ai/AutoAgents)
- **Recommendation**: Consider for complex multi-agent scenarios

### 4. Rusty Agent
- **Purpose**: Simple framework for multi-agent systems
- **Key Features**:
  - ZeroMQ for messaging between agents
  - Dynamic agent discovery and communication
  - Two-thread architecture per agent
- **Use Case**: Lightweight multi-agent coordination
- **Maturity**: Stable
- **Repository**: [docs.rs/rusty_agent](https://docs.rs/rusty_agent)
- **Recommendation**: Good for simple multi-agent scenarios

## Process Management Libraries

### 5. Sopht
- **Purpose**: Sophisticated long-running process management
- **Key Features**:
  - IPC protocol for command/response exchanges
  - Process lifecycle management
  - Superior to basic tools like `tmux`
  - Client-server architecture for process control
- **Use Case**: Perfect for managing subagent processes
- **Maturity**: Stable
- **Repository**: [docs.rs/sopht](https://docs.rs/sopht)
- **Recommendation**: **Highly Recommended** - Excellent for our subagent process management

### 6. PMDaemon
- **Purpose**: High-performance cross-platform process manager
- **Key Features**:
  - Process lifecycle management (start/stop/restart/reload/delete)
  - Clustering support with automatic load balancing
  - Auto-restart on crashes with configurable limits
  - Real-time monitoring (CPU, memory, uptime tracking)
  - System metrics collection
- **Use Case**: Excellent for production-grade subagent management
- **Maturity**: Production-ready
- **Repository**: [docs.rs/pmdaemon](https://docs.rs/pmdaemon)
- **Recommendation**: **Highly Recommended** - Best for production environments

### 7. Process Control
- **Purpose**: Process execution with resource limits
- **Key Features**:
  - Execution time limits
  - Automatic termination capabilities
  - Resource consumption control
  - Child process monitoring
- **Use Case**: Good for preventing runaway subagents
- **Maturity**: Stable
- **Repository**: [docs.rs/process_control](https://docs.rs/process_control)
- **Recommendation**: **Recommended** - Essential for resource management

### 8. Systemg
- **Purpose**: Lightweight process management for Unix-like systems
- **Key Features**:
  - Simple CLI interface for service management
  - Logging capabilities
  - Lifecycle hooks for service management
  - Background service management
- **Use Case**: Good for Unix-specific process management
- **Maturity**: Stable
- **Repository**: [docs.rs/systemg](https://docs.rs/systemg)
- **Recommendation**: Consider for Unix-specific deployments

## Git Operations Libraries

### 9. git2-rs (Currently Used)
- **Purpose**: Git operations via libgit2
- **Key Features**:
  - Comprehensive git functionality
  - Mature and well-tested
  - C library bindings
- **Status**: Currently implemented in our project
- **Repository**: [github.com/rust-lang/git2-rs](https://github.com/rust-lang/git2-rs)
- **Recommendation**: Continue using - it's the standard choice

### 10. gitoxide (Alternative)
- **Purpose**: Pure Rust git implementation
- **Key Features**:
  - Faster than git2-rs
  - Pure Rust (no C dependencies)
  - Modern async/await support
  - Better error handling
- **Use Case**: Could replace git2-rs for better performance
- **Maturity**: Stable
- **Repository**: [github.com/Byron/gitoxide](https://github.com/Byron/gitoxide)
- **Recommendation**: Consider for performance improvements

## Workflow Orchestration Libraries

### 11. Mahler
- **Purpose**: Automated job orchestration with dynamic workflows
- **Key Features**:
  - Hierarchical Task Networks (HTNs) based planning
  - Dynamic workflow composition
  - Directed Acyclic Graph (DAG) representation
  - Intelligent planning for state transitions
- **Use Case**: Complex workflow management for agent tasks
- **Maturity**: Stable
- **Repository**: [docs.rs/mahler](https://docs.rs/mahler)
- **Recommendation**: Consider for complex agent workflows

## Implementation Recommendations

### Immediate Implementation (Phase 1)
1. **Migrate to `agentic` crate** - Perfect MCP server implementation
2. **Keep current trait-based agent abstraction** - It's well-designed and flexible
3. **Integrate Sopht** for sophisticated process management
4. **Add Process Control** for resource limits and safety
5. **Continue with git2-rs** - It's working well

### Future Enhancements (Phase 2)
1. **Consider gitoxide** for performance improvements
2. **Evaluate PMDaemon** for production-grade process management
3. **Monitor Radkit** for potential LLM-powered agent features

### Long-term Considerations (Phase 3)
1. **AgentAI integration** if LLM-powered agents are needed
2. **AutoAgents** for complex multi-agent coordination
3. **Mahler** for sophisticated workflow orchestration

## Architecture Decision

### Current Approach Strengths
- **Trait-based design** provides excellent extensibility
- **Simple and focused** on core requirements
- **No unnecessary complexity** from full agent frameworks
- **Easy to test and maintain**

### Recommended Enhancements
```rust
// MCP server with agentic crate
use agentic::{McpServer, McpClient};

// Enhanced process management with Sopht
use sopht::ProcessManager;

// Resource limits with Process Control
use process_control::{Command, Timeout};

// Future: Consider gitoxide for better performance
// use gitoxide::Repository;
```

### Integration Strategy
1. **Phase 1**: Migrate to `agentic` crate + enhance with Sopht + Process Control
2. **Phase 2**: Evaluate performance improvements (gitoxide, PMDaemon)
3. **Phase 3**: Consider full agent frameworks only if LLM integration is needed

## Conclusion

The discovery of the `agentic` crate changes our implementation strategy significantly. This dedicated MCP library is **perfect** for our use case and should be our primary choice for MCP server implementation.

### Updated Recommendation
1. **Migrate to `agentic` crate** - This provides native MCP server/client support
2. **Keep our trait-based agent abstraction** - It's well-designed and flexible
3. **Enhance with Sopht + Process Control** - For robust process management
4. **Future-proof with planned A2A support** - The `agentic` crate is planning A2A integration

### Key Benefits of `agentic`
- **Native MCP support** - No need to implement MCP protocol ourselves
- **Active development** - Focused specifically on agentic applications
- **Future A2A support** - Built-in path to Agent-to-Agent communication
- **`rpc-router` integration** - Efficient JSON-RPC handling
- **Planned `genai` integration** - Future LLM provider support

The `agentic` crate is exactly what we need - a focused, MCP-native library that handles the protocol complexity while allowing us to focus on our core functionality of spawning subagents with git worktrees.
