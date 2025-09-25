# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based Quine-McCluskey Boolean minimization agent designed as a CLI tool for Claude to use for Boolean function minimization tasks. The project consists of both a library (`qm_agent`) and a binary (`qm-agent`) that provides multiple input formats and output modes.

## Essential Commands

### Building and Testing
```bash
# Build the project
cargo build

# Run all tests (unit + integration)
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_tests

# Run a specific test
cargo test test_minimize_simple_json
```

### Running the CLI
```bash
# Basic minimization
cargo run -- minimize -i "minimize minterms 1,3,7 with 3 variables"

# JSON input
cargo run -- minimize -i '{"minterms": [1,3,7], "variables": 3}' -f json

# Show step-by-step solution
cargo run -- minimize -i "f(A,B) = Σ(1,3)" --show-steps

# Interactive mode
cargo run -- interactive

# Show examples
cargo run -- examples
```

## Architecture

### Core Components

**Library Structure** (`src/lib.rs`):
- `QMSolver`: Main solver interface that orchestrates the QM algorithm
- `QMResult`: Result structure containing minimized expressions, prime implicants, and solution steps
- Convenience functions for common operations (parsing, variable name generation)

**QM Solver Module** (`src/qm_solver/`):
- `quine_mccluskey.rs`: Core QM algorithm implementation with `DummyImplicant` and `BitState` types
- `petricks_method.rs`: Implementation of Petrick's method for finding minimal covers
- `utils.rs`: Utility functions for the QM algorithm
- `mod.rs`: Module interface and `QMSolver` orchestration

**CLI Binary** (`src/main.rs`):
- Complete CLI with subcommands: `minimize`, `interactive`, `examples`
- Multiple input parsers: JSON, function notation (f(A,B) = Σ(1,3)), simple text, truth tables
- Multiple output formats: human-readable, JSON, table, step-by-step
- Interactive mode for iterative problem solving

### Integration Pattern

The CLI integrates with the library through the `integrate_your_qm_solver()` function in `main.rs`. This function:
1. Creates a `QMSolver` instance
2. Sets minterms and don't-care terms
3. Calls `solver.solve()` to get the `QMResult`
4. Extracts and formats the results for CLI output

### Input Format Support

The CLI supports multiple input formats through `parse_input()` and `parse_natural_input()`:
- **JSON**: `{"minterms": [1,3,7], "variables": 3, "dont_cares": [2,4]}`
- **Function notation**: `f(A,B,C) = Σ(1,3,7) + d(2,4)`
- **Simple text**: `minimize minterms 1,3,7 with 3 variables`
- **Truth table**: `truth table: 00110110`
- **File input**: JSON files can be passed as file paths

### Output Formats

The CLI provides multiple output formats:
- **Human-readable**: Formatted with emojis and sections (default)
- **JSON**: Structured data for programmatic use
- **Table**: Truth table format
- **Steps**: Step-by-step solution process

## Testing Structure

- **Unit tests**: Located in `src/lib.rs`, test core library functionality
- **Integration tests**: Located in `tests/integration_tests.rs`, test CLI behavior end-to-end
- Integration tests use `assert_cmd` and `predicates` for CLI testing
- Tests cover all input formats, output formats, error conditions, and edge cases

## Key Implementation Notes

- The QM algorithm uses a `DummyImplicant` structure with `BitState` enum (Zero, One, DontCare)
- Prime implicants are found through iterative combining until no more combinations are possible
- Essential prime implicants are currently simplified (taking first half of prime implicants)
- The Petrick's method implementation uses a greedy approach for minimal cover selection
- Variable names default to A, B, C, D... but can be customized
- Unicode handling in CLI arguments requires careful encoding (avoid raw Σ symbols in tests)

## Agent Registration and Distribution

### For Claude Code Users

This project includes a configured subagent for Claude Code. The subagent is automatically available when working in this project directory.

**Subagent Configuration**: `.claude/agents/qm-agent.md`
- **Name**: `qm-agent`
- **Purpose**: Boolean function minimization and Quine-McCluskey algorithm tasks
- **Tools**: Bash (for running the CLI)

### Installation for New Projects

To use this QM agent in other projects:

1. **Copy the subagent file**:
   ```bash
   mkdir -p .claude/agents
   cp .claude/agents/qm-agent.md /path/to/your/project/.claude/agents/
   ```

2. **Or install globally** (available in all projects):
   ```bash
   mkdir -p ~/.claude/agents
   cp .claude/agents/qm-agent.md ~/.claude/agents/
   ```

3. **Build the CLI tool** in your project:
   ```bash
   # Copy source files or add as dependency
   cargo build --release
   ```

### Using the Agent

Claude Code will automatically detect and use the `qm-agent` subagent when users:
- Ask for Boolean function minimization
- Mention Karnaugh maps or K-maps
- Request digital logic optimization
- Need help with Boolean algebra problems
- Specifically mention the Quine-McCluskey algorithm

Example user requests that trigger the agent:
- "Minimize the Boolean function f(A,B,C) = Σ(1,3,7)"
- "Help me simplify this Karnaugh map"
- "Optimize this digital logic circuit"
- "Find the prime implicants for these minterms"