# Cross-Repository Dependency Structure

## Overview

This project (`qmc-rust-agent`) depends on an external local workspace (`bitwise-rust-agent`) for AVX-512 SIMD primitives. Meanwhile, `bitwise-rust-agent` depends back on `qmc-rust-agent`. The dependency graph is **not a true cycle** — only through a third crate that depends on both.

## Dependency Graph

```
qmc-rust-agent (this repo)
  └─ depends on: bitwise-simd (workspace sibling, path = C:/Source/Private/rust/bitwise-rust-agent/bitwise-simd)
         │
         └─ ZERO dependencies on qmc-rust-agent (acyclic)

bitwise-rust-agent (workspace)
  ├── bitwise-simd        (SIMD primitives crate)
  │      └── dependencies: rand (optional, test-utils feature only)
  │
  └── bitwise-agent-rust  (code generator for SIMD intrinsics)
         ├── depends on: bitwise-simd (workspace sibling, features = ["test-utils"])
         └── depends on: qm-agent (workspace path → C:/Source/Github/qmc-rust-agent)
```

## Repository Inventory

### 1. `qmc-rust-agent` (this repo, C:\Source\Github\qmc-rust-agent)

**Purpose:** Quine-McCluskey Boolean minimization library and CLI tool with CNF-to-DNF conversion and if-then-else simplification via JSON API for Claude Code integration.

**Package name:** `qm-agent` (lib: `qm_agent`, bin: `qm-agent`)

**Key modules:**
- `qm` — Quine-McCluskey algorithm, Petrick's method, encoding system
- `cnf_dnf` — CNF to DNF conversion with SIMD optimizations
- `simplify` — If-then-else chain simplification with dead code detection
- `agent_api` — JSON API for Claude Code integration

### 2. `bitwise-rust-agent` (C:\Source\Private\rust\bitwise-rust-agent)

**Purpose:** Workspace containing low-level AVX-512 SIMD primitives and a code generator for ABC-synthesized boolean functions.

**Workspace crate `bitwise-simd`:**
- **Package name:** `bitwise-simd` (lib: `bitwise_simd`)
- **Role:** AVX-512 intrinsics for bitwise operations
- **Modules:**
  - `bit_plane` — GFNI bit-plane transposition (`bps_gfni_4to8`, `bps_gfni_8to4`, `bps_gfni_8to8`)
  - `bit_tools` — `bit_ror`, `bit_select` low-level operations
  - `gfni_tools` — GFNI instruction utilities
  - `generated` — ABC-synthesized function code (`_mm512_covers_4_4_4_1`, `_mm512_covers_5_5_5_1`)
- **Dependencies:** None (pure SIMD primitives). Optional `rand` for test-utils only.

**Workspace crate `bitwise-agent-rust`:**
- **Package name:** `bitwise-agent-rust` (lib: `bitwise_agent_rust`)
- **Role:** Code generator for AVX-512 SIMD bitwise operations using ABC synthesis output
- **Dependencies:** `bitwise-simd`, `qm-agent` (via workspace)

### 3. `sneller` (C:\Source\sneller\sneller)

**Role:** CPU-only reference implementation serving as the correctness baseline.

- Go reference code in `go/regexp2/DsTiny.go` and CPU AVX-512 kernels
- Source of truth for data format, algorithm correctness, expected behavior
- GPU extensions in `cuda/parallel-regex` extend only the regex/DFA portions

## qm-agent → bitwise-simd Usage

`qmc-rust-agent` uses `bitwise-simd` exclusively for the SIMD-accelerated coverage matrix in `src/qm/simd_coverage.rs`:

| Function | File | Purpose |
|----------|------|---------|
| `bps_gfni_8to4` | `simd_coverage.rs:295` | Bit-plane separation for 4-bit coverage checks |
| `bps_gfni_8to8` | `simd_coverage.rs:413` | Bit-plane separation for 5-bit coverage checks |
| `_mm512_covers_4_4_4_1` | `simd_coverage.rs:307` | Generated AVX-512 intrinsic: check 512 4-bit coverage pairs simultaneously |
| `_mm512_covers_5_5_5_1` | `simd_coverage.rs:435` | Generated AVX-512 intrinsic: check 512 5-bit coverage pairs simultaneously |

The SIMD coverage matrix processes 512 prime implicant-minterm pairs simultaneously, achieving 5.93x speedup over scalar (requires AVX-512F + GFNI).

## Cargo.toml Configuration

### qmc-rust-agent/Cargo.toml
```toml
[dependencies]
bitwise-simd = { path = "C:/Source/Private/rust/bitwise-rust-agent/bitwise-simd", optional = true }

[features]
default = ["simd"]
simd = ["dep:bitwise-simd"]
```

### bitwise-rust-agent/Cargo.toml (workspace)
```toml
[workspace]
members = ["bitwise-agent", "bitwise-simd"]
resolver = "2"

[workspace.dependencies]
qm-agent = { path = "C:/source/Github/qmc-rust-agent" }
```

### bitwise-rust-agent/bitwise-agent/Cargo.toml
```toml
[dependencies]
bitwise-simd = { path = "../bitwise-simd", features = ["test-utils"] }
qm-agent = { workspace = true }
```

## Known Issues

### 1. Hardcoded Absolute Path

`qmc-rust-agent/Cargo.toml` uses a hardcoded Windows absolute path:
```toml
bitwise-simd = { path = "C:/Source/Private/rust/bitwise-rust-agent/bitwise-simd", optional = true }
```

This prevents cloning/forking to any other machine without manually editing `Cargo.toml`.

### 2. No Workspace Integration

`qmc-rust-agent` is not part of the `bitwise-rust-agent` workspace. This means:
- `qmc-rust-agent` cannot use `qm-agent = { workspace = true }` for the reverse dependency
- Changes to `bitwise-simd` require `cargo clean` in `qmc-rust-agent` (Cargo's out-of-tree build behavior detects out-of-date deps)
- No shared dependency versioning

### 3. Feature Mismatch

`bitwise-agent-rust` enables `features = ["test-utils"]` on `bitwise-simd` (providing `rand` for test harnesses), but `qmc-rust-agent` does not. The `simd` feature is sufficient for production use but prevents running `bitwise-simd`'s own test utilities from `qmc-rust-agent`.

### 4. No Publish Support

With hardcoded path dependencies, neither crate can be published to crates.io. Both are development-only tools.

## Development Workflow

### Editing bitwise-simd (SIMD primitives)
1. Make changes in `C:\Source\Private\rust\bitwise-rust-agent\bitwise-simd\`
2. In `qmc-rust-agent`: `cargo clean` (force recompile due to out-of-tree workspace)
3. In `bitwise-rust-agent`: `cargo build` (workspace build)

### Editing bitwise-agent-rust (code generator)
1. Make changes in `C:\Source\Private\rust\bitwise-rust-agent\bitwise-agent\`
2. Build workspace: `cd C:\Source\Private\rust\bitwise-rust-agent && cargo build`
3. Run generator to produce new intrinsics in `bitwise-simd/generated/`
4. Rebuild `qmc-rust-agent`: `cargo clean && cargo build`

### Adding new boolean function code generation
1. Design boolean function for ABC synthesis
2. Generate using `bitwise-agent-rust` (which uses `qm-agent` for logic validation)
3. New intrinsics appear in `bitwise-simd/generated/`
4. `qmc-rust-agent` picks them up automatically via the re-export in `bitwise-simd/src/lib.rs`
