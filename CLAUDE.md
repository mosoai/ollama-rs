# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Building
- `cargo build` - Build the workspace
- `cargo build --verbose` - Build with verbose output

### Testing
- `cargo test` - Run all tests
- `cargo test -p ollama-rs-macros --verbose` - Run macro tests specifically
- `cargo test --all-features` - Run tests with all features enabled

### Linting and Formatting
- `cargo fmt` - Format code
- `cargo clippy --workspace --no-deps --all-features --all-targets -- -D warnings` - Run clippy with strict checks
- `cargo clippy --fix --allow-dirty` - Auto-fix clippy warnings
- `cargo fmt --all -- --check` - Check formatting without making changes

### CI Checks
The CI runs formatting checks, clippy, and builds. The `rust.yml` workflow shows the exact commands used.

## Architecture

### Workspace Structure
This is a Cargo workspace with two crates:
- `ollama-rs` - Main library crate
- `ollama-rs-macros` - Procedural macros crate

### Core Components

**Ollama Client** (`lib.rs`)
- Main entry point for interacting with the Ollama API
- Default connects to `http://127.0.0.1:11434`
- Supports custom URLs and ports via `Ollama::new()` or `Ollama::from_url()`
- Uses `reqwest` internally for HTTP requests

**Generation Module** (`generation/`)
- `chat` - Chat completion with history support
- `completion` - Single-shot text generation
- `embeddings` - Text embedding generation
- `images` - Image input handling
- `tools` - Function calling/tool system
- `parameters` - Request parameters (format, keep_alive, etc.)

**Models Module** (`models/`)
- Model lifecycle operations: `create`, `copy`, `delete`, `list_local`, `pull`, `push`, `show_info`
- `ModelOptions` - Builder-pattern struct for generation parameters (temperature, top_k, top_p, num_ctx, etc.)
- `LocalModel` and `ModelInfo` structs for model metadata

**Tool/Function Calling System**
- `Tool` trait - Defines tools with parameters, name, description, and async `call()` method
- `Coordinator` - High-level abstraction that manages chat history, tool registration, and automatic tool calling
- Tools can be added via `add_tool()` and will be automatically invoked when the LLM requests them
- The `#[ollama_rs::function]` macro (from `ollama-rs-macros`) converts async functions into tools

**Chat History** (`history.rs`)
- `ChatHistory` trait for managing conversation state
- `Vec<ChatMessage>` implements this trait by default
- Custom types can implement `ChatHistory` for specialized history management

**Feature Flags**
- `default` - `reqwest/default-tls`
- `stream` - Enable streaming responses (requires `tokio`)
- `rustls` - Use rustls instead of native TLS
- `headers` - Support for custom request headers
- `tool-implementations` - Built-in tools (search, scraper, calculator, etc.)
- `macros` - Enable the `#[function]` procedural macro
- `modelfile` - Parse Modelfiles as structured data

### Error Handling
Errors are defined in `error.rs` and use `thiserror` for proper error types. The `ToolCallError` enum handles tool-specific failures.

### Examples
See `ollama-rs/examples/` for usage patterns:
- `basic_chatbot.rs` - Simple chatbot
- `chat_with_history.rs` - Chat with history management
- `function_call.rs` - Function/tool calling
- `coordinator.rs` - Using the Coordinator abstraction
- `structured_output.rs` - JSON schema-based structured output
