# QM Rust Agent

A Quine-McCluskey Boolean minimization agent built in Rust, designed to work seamlessly with Claude Code for Boolean algebra tasks.

## Overview

This tool implements the Quine-McCluskey algorithm to minimize Boolean functions and provides multiple input/output formats for easy integration with various workflows. It's particularly useful for digital logic design, Boolean algebra education, and automated circuit optimization.

## Features

- **Multiple Input Formats**:
  - Function notation: `f(A,B,C) = Σ(1,3,7)`
  - With don't cares: `f(A,B,C) = Σ(1,3,7) + d(2,4)`
  - Simple format: `minimize minterms 1,3,7 with 3 variables`
  - JSON: `{"minterms": [1,3,7], "variables": 3}`
  - Truth table: `truth table: 00110110`
  - File input (JSON)

- **Output Formats**:
  - Human-readable (default)
  - JSON
  - Table format
  - Step-by-step solution

- **Features**:
  - Prime implicant generation
  - Essential prime implicant identification
  - Cost reduction analysis
  - Truth table generation
  - Interactive mode

## Installation

### Prerequisites

- Rust (latest stable version)
- Cargo

### Dependencies

Add these to your `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
regex = "1.5"
anyhow = "1.0"
```

### Build

```bash
git clone <repository-url>
cd qmc-rust-agent
cargo build --release
```

Or run directly with:
```bash
cargo run -- minimize -i "f(A,B) = Σ(1,3)"
```

## Usage

### Command Line Interface

```bash
# Basic minimization
cargo run -- minimize -i "f(A,B,C) = Σ(1,3,7)"

# With don't cares
cargo run -- minimize -i "f(A,B,C) = Σ(1,3,7) + d(2,4)"

# Show step-by-step solution
cargo run -- minimize -i "f(A,B) = Σ(1,3)" --show-steps

# JSON output
cargo run -- minimize -i "minimize minterms 1,3,7 with 3 variables" -f json

# From file
cargo run -- minimize -i input.json

# Interactive mode
cargo run -- interactive

# Show examples
cargo run -- examples
```

### Input Formats

#### 1. Function Notation
```
f(A,B,C) = Σ(1,3,7)
f(A,B,C) = Σ(1,3,7) + d(2,4)  # with don't cares
```

#### 2. Simple Format
```
minimize minterms 1,3,7 with 3 variables
```

#### 3. JSON Format
```json
{
  "minterms": [1, 3, 7],
  "dont_cares": [2, 4],
  "variables": 3,
  "variable_names": ["A", "B", "C"]
}
```

#### 4. Truth Table Format
```
truth table: 00110110
```

### Output Example

```
🔍 Quine-McCluskey Boolean Minimization Result
════════════════════════════════════════════

📊 Input:
   Minterms: [1, 3, 7]
   Don't cares: [2, 4]

✨ Minimized Expression (SOP):
   F = A'B + BC

🎯 Prime Implicants:
   • A'B
   • BC
   • AC

⭐ Essential Prime Implicants:
   • A'B
   • BC

💰 Cost Reduction: 45.2%
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
- Multiple input parsers: JSON, function notation, simple text, truth tables
- Multiple output formats: human-readable, JSON, table, step-by-step
- Interactive mode for iterative problem solving

## Testing

```bash
# Run all tests (unit + integration)
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_tests

# Run a specific test
cargo test test_minimize_simple_json
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

[Add your license here]

## Acknowledgments

- Built for integration with Claude Code
- Implements the classic Quine-McCluskey algorithm for Boolean minimization