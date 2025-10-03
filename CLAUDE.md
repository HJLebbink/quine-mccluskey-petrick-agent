# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based Boolean logic optimization library and CLI tool featuring:
- **Quine-McCluskey algorithm** for Boolean function minimization
- **CNF to DNF conversion** with SIMD-optimized implementations (AVX2, AVX512)
- **Petrick's method** for minimal cover selection
- Multiple input/output formats for easy integration

The project consists of both a library (`qm_agent`) and a binary (`qm-agent`) that provides comprehensive Boolean algebra operations.

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

# Run long-running equality tests (100K iterations)
cargo test --test equality_tests -- --ignored
```

### Running Examples

```bash
# Quine-McCluskey examples
cargo run --example qm_simple_3bit
cargo run --example qm_petricks_method
cargo run --example qm_wolfram_verified

# CNF to DNF conversion examples
cargo run --example cnf_2_dnf_0
cargo run --example cnf_2_dnf_5

# See examples/README.md for full list and descriptions
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench --bench cnf_to_dnf_bench

# Run specific benchmark groups
cargo bench --bench cnf_to_dnf_bench -- encoding_types
cargo bench --bench cnf_to_dnf_bench -- 64bit_comparison

# Save baseline for comparison
cargo bench --bench cnf_to_dnf_bench -- --save-baseline main

# See benches/README.md for detailed documentation
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

**QM Module** (`src/qm/`):
- `implicant.rs`: `Implicant` structure and `BitState` enum (Zero, One, DontCare)
- `quine_mccluskey.rs`: Core QM algorithm implementation
- `petricks_method.rs`: Implementation of Petrick's method for finding minimal covers
- `qm_solver.rs`: `QMSolver` orchestration and public API
- `qm_result.rs`: `QMResult` output structure
- `encoding.rs`: `MintermEncoding` trait and encoding types (Encoding16/32/64)
- `minterm_set.rs`: `MintermSet` data structure
- `random.rs`: Random minterm generation utilities (for testing and benchmarking)
- `classic.rs`: C++ API-compatible port with preserved naming conventions
- `mod.rs`: Module interface with convenient re-exports

**CLI Binary** (`src/main.rs`):
- Complete CLI with subcommands: `minimize`, `interactive`, `examples`
- Multiple input parsers: JSON, function notation (f(A,B) = Σ(1,3)), simple text, truth tables
- Multiple output formats: human-readable, JSON, table, step-by-step
- Interactive mode for iterative problem solving

**CNF to DNF Module** (`src/cnf_dnf/`):
- `convert.rs`: Main conversion logic and algorithms
  - `convert_cnf_to_dnf_encoding<E>()`: Convert CNF to DNF with encoding-aware optimization
  - `convert_cnf_to_dnf_minimal_encoding<E>()`: Find minimal DNF with early pruning optimization
  - `convert_cnf_to_dnf_with_names()`: String-based variable name support
  - Encoding types automatically select optimal SIMD strategy:
    - `Enc16` (≤16 vars) → AVX512_16bits
    - `Enc32` (≤32 vars) → AVX512_32bits
    - `Enc64` (≤64 vars) → AVX512_64bits
  - Subsumption checking to minimize resulting DNF
  - Type-safe validation of variable counts
- `simd.rs`: SIMD-optimized implementations (x86_64 only)
  - AVX512 implementations for 8-bit, 16-bit, 32-bit, and 64-bit elements
  - AVX2 implementation for 64-bit elements
  - Runtime CPU feature detection with automatic fallback
  - Up to 4x speedup on large problems (64 variables) with AVX512
  - Platform-independent fallback implementations for non-x86_64 architectures
- `mod.rs`: Module interface with convenient re-exports

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

- **Unit tests**: Located in `src/lib.rs` and module files, test core library functionality (14 tests)
- **Integration tests**: Located in `tests/integration_tests.rs`, test CLI behavior end-to-end (10 tests)
- **Equality tests**: Located in `tests/equality_tests.rs`, randomized quality assurance tests
  - `quick_equality_smoke_test`: Tests that different encodings produce identical results
  - `equality_test`: 100,000 iterations testing all encodings (run with `--ignored`)
  - `equality_test_minimal`: 100,000 iterations testing early pruning correctness (run with `--ignored`)
- **Examples**: 15 total (7 CNF to DNF + 8 QM) with comprehensive README documentation
- **Benchmarks**: Criterion-based benchmarks in `benches/cnf_to_dnf_bench.rs`
  - 6 benchmark groups comparing Encoding16, Encoding32, and Encoding64 performance
  - Tests different problem sizes and conjunction densities
  - Detailed results and analysis in `benches/RESULTS.md`
- Integration tests use `assert_cmd` and `predicates` for CLI testing
- Tests cover all input formats, output formats, error conditions, and edge cases

## Key Implementation Notes

### Quine-McCluskey Algorithm
- Uses `Implicant` structure with `BitState` enum (Zero, One, DontCare)
- `BitState` is `Copy` for zero-cost operations
- Prime implicants found through iterative combining until no more combinations possible
- Essential prime implicants currently simplified (taking first half of prime implicants)
- Petrick's method uses greedy approach for minimal cover selection
- Variable names default to A, B, C, D... but can be customized
- Unicode handling in CLI arguments requires careful encoding (avoid raw Σ symbols in tests)

### CNF to DNF Conversion
- Uses bit-level operations on u64 for efficient term representation
- Subsumption checking: If `z | q == z`, then z is subsumed; if `z | q == q`, then q is subsumed
- **Critical**: Uses O(n) filtering with HashSet for deletions (not O(n²) repeated Vec::remove)
- Early pruning optimization discards non-minimal terms during computation
- **Encoding-aware API**: Use generic parameter to specify encoding type
  - `convert_cnf_to_dnf_encoding::<Encoding16>(&cnf, n_bits)` for ≤16 variables
  - `convert_cnf_to_dnf_encoding::<Encoding32>(&cnf, n_bits)` for ≤32 variables
  - `convert_cnf_to_dnf_encoding::<Encoding64>(&cnf, n_bits)` for ≤64 variables
- Encoding type automatically selects optimal SIMD implementation
- Maximum 64 variables for u64-based representation
- Runtime validation ensures n_bits doesn't exceed encoding capacity

### SIMD Optimizations
- Runtime CPU feature detection with `is_x86_feature_detected!()`
- Automatic fallback to scalar implementation if SIMD unavailable
- **Performance characteristics**:
  - Small problems (< 16 vars): Scalar X64 fastest due to SIMD overhead
  - Medium problems (16-32 vars): Break-even point, dramatic gains at 32 vars
  - Large problems (64 vars): 4.0x speedup with AVX512, 2.8x with AVX2
- AVX512 variants process 64, 32, 16, or 8 elements per vector depending on bit width
- All unsafe SIMD code properly wrapped with safe public APIs
- Rust 2024 edition requires inner `unsafe` blocks within `unsafe fn`

### Code Idioms and Best Practices
- Proper error handling: No `unwrap()` in user-facing code, use `?` operator
- Pre-allocated vectors with `Vec::with_capacity()` where size is known
- `PartialEq` and `Eq` derives for testability
- Associated methods on enums for DRY principle (e.g., `OptimizedFor::max_bits()`)
- Functional style preferred where it improves clarity

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
- Request CNF to DNF conversion
- Ask about SIMD optimization for Boolean operations
- Specifically mention the Quine-McCluskey algorithm

Example user requests that trigger the agent:
- "Minimize the Boolean function f(A,B,C) = Σ(1,3,7)"
- "Help me simplify this Karnaugh map"
- "Optimize this digital logic circuit"
- "Find the prime implicants for these minterms"
- "Convert this CNF formula to DNF"
- "What's the minimal DNF for this Boolean expression?"

## Additional Resources

- **README.md**: User-facing documentation with installation and usage
- **IMPROVEMENTS.md**: Detailed log of code improvements and idioms applied
- **benches/README.md**: Comprehensive benchmark documentation
- **benches/RESULTS.md**: Sample benchmark results and analysis
- **examples/README.md**: Descriptions of all 15 examples