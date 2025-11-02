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

- **Performance Optimizations**:
  - AVX512 SIMD vectorization for Quine-McCluskey hot loop
  - **NEW!** SIMD-accelerated coverage matrix (5.93× speedup)
    - Bit-plane transposition for 512-way parallelism
    - Automatic activation for large problems (≥1K checks)
    - Requires AVX-512F and GFNI CPU features
  - 4-16x speedup on compatible CPUs for large problems
  - Runtime CPU feature detection with automatic scalar fallback
  - Handles up to 16 variables efficiently (~10s vs ~120s previously)
  - All correctness guarantees preserved

- **CNF to DNF Conversion**:
  - Conjunctive Normal Form to Disjunctive Normal Form conversion
  - Minimal DNF computation with early pruning optimization
  - SIMD-optimized implementations (AVX2, AVX512)
  - Multiple bit-width variants (8, 16, 32, 64-bit)
  - Runtime CPU feature detection with automatic fallback
  - Up to 4x speedup on large problems with AVX512

- **Claude Integration (NEW!)**:
  - JSON API for simplifying if-then-else conditions
  - Works with any programming language (Go, Rust, C++, Python, etc.)
  - Claude handles language parsing, agent handles boolean algebra
  - Dead code detection across branches
  - Coverage analysis for untested conditions
  - Multi-language code generation
  - Perfect for refactoring complex conditional logic

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

## Claude Integration - Simplify If-Then-Else Logic

The agent provides a JSON API for simplifying conditional logic in any programming language. Claude handles language understanding, and the agent provides boolean algebra optimization.

### How It Works

```
┌─────────────────┐
│  Your Code      │  Go, Rust, C++, Python, etc.
│  (any language) │
└────────┬────────┘
         │
         v
┌─────────────────┐
│  Claude         │  • Parses code
│                 │  • Extracts conditions
│                 │  • Detects side effects
└────────┬────────┘
         │ JSON
         v
┌─────────────────┐
│  QM Agent       │  • Boolean optimization
│  (this tool)    │  • Dead code detection
│                 │  • Coverage analysis
└────────┬────────┘
         │ JSON
         v
┌─────────────────┐
│  Claude         │  • Generates suggestions
│                 │  • Produces idiomatic code
└─────────────────┘
```

### Quick Example

**Input code (Go):**
```go
func check(a, b bool) int {
    if a && b {
        return 1
    }
    if a && !b {
        return 1
    }
    return 0
}
```

**Agent JSON request:**
```json
{
  "variables": {
    "a": "boolean",
    "b": "boolean"
  },
  "branches": [
    {"condition": "a && b", "output": "return 1"},
    {"condition": "a && !b", "output": "return 1"}
  ],
  "default": "return 0",
  "context": {"language": "go"}
}
```

**Run simplification:**
```bash
qm-agent simplify -i input.json
# or
cat input.json | qm-agent simplify
```

**Agent response shows:**
- Condition simplifies to just `a`
- 50% complexity reduction
- Generated Go code suggestion

**Suggested code:**
```go
func check(a, b bool) int {
    if a {
        return 1
    }
    return 0
}
```

### JSON API Reference

See `examples/agent/README.md` for complete API documentation and more examples.

**Quick links:**
- [Simple boolean example](examples/agent/simple.json)
- [Go access control](examples/agent/go_example.json)
- [Dead code detection](examples/agent/dead_code.json)
- [Integer comparisons](examples/agent/integer_comparison.json)

### Supported Features

✅ Boolean variables
✅ Integer variables (bounded domains)
✅ Comparison operators (==, !=, <, >, <=, >=)
✅ Boolean operators (&&, ||, !)
✅ Dead code detection
✅ Coverage analysis
✅ Multi-language code generation (Go, Rust, C++, Python)
✅ Complexity metrics

### Use Cases

1. **Refactoring complex conditions**
   ```
   if (a && b) || (a && !b)  →  if a
   ```

2. **Finding unreachable code**
   ```
   if a || b { ... }
   elif a && b { ... }  // Dead code! Already covered above
   ```

3. **Detecting coverage gaps**
   ```
   if a && b { ... }
   // Missing: !a && !b, !a && b, a && !b
   ```

4. **Optimizing access control**
   ```
   3 overlapping permission checks → 2 simplified checks
   ```

### Integration with Claude

When Claude encounters complex conditional logic, it can:

1. **Extract** conditions and outcomes
2. **Call** `qm-agent simplify` with JSON
3. **Receive** optimization suggestions
4. **Generate** refactored code in original language style

Example Claude workflow:
```
User: "Can you simplify this if-else mess?"
Claude: [analyzes code]
Claude: [calls agent with conditions]
Claude: "I found 2 issues:
         1. Branch 3 is unreachable (covered by branch 1)
         2. The logic simplifies to just checking flag1
         Here's the optimized version..."
```

## Architecture

### Core Components

**Library Structure** (`src/lib.rs`):
- `QMSolver`: Main solver interface that orchestrates the QM algorithm
- `QMResult`: Result structure containing minimized expressions, prime implicants, and solution steps
- Convenience functions for common operations (parsing, variable name generation)

**QM Solver Module** (`src/qm_solver/`):
- `quine_mccluskey.rs`: Core QM algorithm implementation with `Implicant` and `BitState` types
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
   - Scalar implementation often **fastest** due to SIMD overhead
   - SIMD overhead (setup, data movement) can exceed benefits
   - Typical time: 600-900 nanoseconds
   - **Recommendation**: Use `Encoding16` (auto-selects optimal strategy)

2. **Medium Problems (16-32 variables)**:
   - Break-even point around 16 variables
   - SIMD shows dramatic gains at 32 variables (301x speedup)
   - Performance depends heavily on problem structure
   - **Recommendation**: Use `Encoding32` (auto-selects AVX512_32bits)

3. **Large Problems (64 variables)**:
   - Clear SIMD advantages
   - AVX512: 4.0x speedup (395ms → 98ms)
   - AVX2: 2.8x speedup (395ms → 143ms)
   - Speedup scales with elements per vector (4 for AVX2, 8 for AVX512)
   - **Recommendation**: Use `Encoding64` (auto-selects AVX512_64bits)

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

#### Encoding Selection Guide

The new encoding-aware API automatically selects the optimal SIMD strategy:

```rust
use qm_agent::cnf_dnf::convert_cnf_to_dnf_encoding;
use qm_agent::qm::{Encoding16, Encoding32, Encoding64};

// Select encoding based on variable count
// Each encoding validates capacity and auto-selects SIMD optimization
match n_variables {
    0..=16  => convert_cnf_to_dnf_encoding::<Encoding16>(&cnf, n_variables),
    17..=32 => convert_cnf_to_dnf_encoding::<Encoding32>(&cnf, n_variables),
    33..=64 => convert_cnf_to_dnf_encoding::<Encoding64>(&cnf, n_variables),
    _       => panic!("Maximum 64 variables supported"),
}
```

**Benefits**:
- Type-safe: Encoding validates variable count at runtime
- Automatic: No manual SIMD optimization selection needed
- Simple: One parameter instead of two

See [`benches/README.md`](benches/README.md) for detailed documentation and [`benches/RESULTS.md`](benches/RESULTS.md) for complete benchmark results.

### SIMD Coverage Matrix Benchmark

The Quine-McCluskey algorithm includes AVX-512 accelerated coverage matrix computation for checking which minterms are covered by each prime implicant:

```bash
# Run SIMD coverage benchmark (requires AVX-512F and GFNI)
cargo run --release --example benchmark_simd_coverage
```

**Results** (100 prime implicants × 10,000 minterms = 1 million checks):

| Implementation | Time | Throughput | Speedup |
|----------------|------|------------|---------|
| Scalar | 7.69 ms | 130M checks/sec | 1.0× |
| AVX-512 SIMD | 1.30 ms | 770M checks/sec | **5.93×** |

The SIMD implementation processes 512 minterm-implicant pairs simultaneously using bit-plane transposition, achieving nearly 6× speedup despite transposition overhead.

**Documentation:**
- Implementation details: [`SIMD_COVERAGE.md`](SIMD_COVERAGE.md)
- Benchmark code: [`examples/benchmark_simd_coverage.rs`](examples/benchmark_simd_coverage.rs)
- Bug fix history: See `SIMD_COVERAGE_FIX.md` in the bitwise-rust-agent repository

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