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

- **Core Features**:
  - Prime implicant generation
  - Essential prime implicant identification
  - Petrick's method for minimal cover selection
  - Cost reduction analysis
  - Truth table generation
  - Interactive mode

- **CNF to DNF Conversion**:
  - Conjunctive Normal Form to Disjunctive Normal Form conversion
  - Minimal DNF computation with early pruning optimization
  - SIMD-optimized implementations (AVX2, AVX512)
  - Multiple bit-width variants (8, 16, 32, 64-bit)
  - Runtime CPU feature detection with automatic fallback
  - Up to 4x speedup on large problems with AVX512

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

### Option 1: Download Release (Recommended)

Download the latest release from [GitHub Releases](https://github.com/your-username/qmc-rust-agent/releases):

```bash
# Download and extract installer package
wget https://github.com/your-username/qmc-rust-agent/releases/latest/download/qmc-rust-agent-installer.tar.gz
tar -xzf qmc-rust-agent-installer.tar.gz
cd qmc-rust-agent-*-installer

# Run installer
./install.sh --global  # Install globally for all projects
```

### Option 2: Build from Source

```bash
git clone https://github.com/your-username/qmc-rust-agent.git
cd qmc-rust-agent
cargo build --release

# Or run directly
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

# Run long-running quality tests (100K iterations)
cargo test --test equality_tests -- --ignored
```

## Benchmarks

The project includes comprehensive benchmarks comparing different SIMD optimization levels for CNF to DNF conversion:

```bash
# Run all benchmarks
cargo bench --bench cnf_to_dnf_bench

# Run specific benchmark groups
cargo bench --bench cnf_to_dnf_bench -- optimization_levels
cargo bench --bench cnf_to_dnf_bench -- problem_sizes
cargo bench --bench cnf_to_dnf_bench -- 64bit_comparison
```

### Performance Characteristics

The CNF to DNF conversion includes SIMD-optimized implementations using AVX2 and AVX512 intrinsics. Benchmark results on Intel CPUs with AVX512F/BW support reveal:

#### SIMD Speedup by Problem Size

| Problem Size | Best Implementation | Speedup vs Scalar |
|-------------|---------------------|-------------------|
| 8 variables | X64 (scalar) | 1.0x (baseline) |
| 16 variables | X64/AVX512 tie | ~1.0x (break-even) |
| 32 variables | AVX512_32bits | **301x faster** |
| 64 variables | AVX512_64bits | **4.0x faster** |

#### Key Findings

1. **Small Problems (< 16 variables)**:
   - Scalar implementation is **fastest**
   - SIMD overhead (setup, data movement) exceeds benefits
   - Typical time: 600-900 nanoseconds
   - **Recommendation**: Use `OptimizedFor::X64`

2. **Medium Problems (16-32 variables)**:
   - Break-even point around 16 variables
   - SIMD shows dramatic gains at 32 variables (301x speedup)
   - Performance depends heavily on problem structure
   - **Recommendation**: Use AVX512 variant matching bit width

3. **Large Problems (64 variables)**:
   - Clear SIMD advantages
   - AVX512: 4.0x speedup (395ms → 98ms)
   - AVX2: 2.8x speedup (395ms → 143ms)
   - Speedup scales with elements per vector (4 for AVX2, 8 for AVX512)
   - **Recommendation**: Use `OptimizedFor::Avx512_64bits` or `Avx2_64bits`

4. **Early Pruning Optimization**:
   - Provides consistent **30% speedup** for minimal DNF computation
   - More effective on larger, denser problems
   - Discards non-minimal terms during computation
   - **Recommendation**: Always enable `EARLY_PRUNE = true` for minimal DNF

5. **Conjunction Density Impact**:
   - Sparse (2 literals): 1.2 µs (baseline)
   - Medium (4 literals): 5.5 µs (4.6x slower)
   - Dense (8 literals): 10.4 µs (8.6x slower)
   - More literals → exponentially more DNF terms
   - **Recommendation**: Use minimal DNF with pruning for dense problems

#### CPU Requirements

- **AVX2 support**: Intel Haswell (2013+) or AMD Excavator (2015+)
- **AVX512 support**: Intel Skylake-X (2017+) or AMD Zen 4 (2022+)
- Automatic fallback to scalar if SIMD not available
- Runtime CPU feature detection (no recompilation needed)

#### Optimization Selection Guide

```rust
use qm_agent::cnf_to_dnf::OptimizedFor;

let opt_level = match n_variables {
    0..=15  => OptimizedFor::X64,              // Scalar fastest
    16      => OptimizedFor::Avx512_16bits,    // Match bit width
    17..=32 => OptimizedFor::Avx512_32bits,    // Dramatic speedup
    33..=64 => OptimizedFor::Avx512_64bits,    // Maximum performance
    _       => OptimizedFor::X64,              // Fallback
};
```

See [`benches/README.md`](benches/README.md) for detailed documentation and [`benches/RESULTS.md`](benches/RESULTS.md) for complete benchmark results.

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