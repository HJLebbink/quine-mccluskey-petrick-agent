# Quality Assurance Tests

## Equality Tests

The `equality_tests.rs` file contains randomized quality assurance tests ported from the C++ implementation.

### Running Tests

```bash
# Quick smoke test (100 iterations, runs automatically with cargo test)
cargo test quick_equality_smoke_test

# Full equality test (100,000 iterations - takes significant time!)
cargo test equality_test -- --ignored --nocapture

# Full minimal equality test (100,000 iterations)
cargo test equality_test_minimal -- --ignored --nocapture
```

### Test Descriptions

#### `quick_equality_smoke_test`
- **Runs by default** with `cargo test`
- 100 random iterations
- Tests variables: 1-16 bits (small for speed)
- Verifies X64 and Avx512_64bits produce identical results
- Completes in ~0.01 seconds

#### `equality_test` (ignored by default)
- **100,000 random experiments**
- Tests all optimization levels: X64, Avx512_64bits, Avx2_64bits, Avx512_32bits, Avx512_16bits, Avx512_8bits
- Random parameters:
  - Variables: 1-64 bits
  - Conjunctions: 1-10
  - Disjunctions: 1 to n_variables
- Verifies all implementations produce identical DNF results
- Reports progress every 1000 experiments
- **Runtime**: Can take hours depending on hardware

#### `equality_test_minimal` (ignored by default)
- **100,000 random experiments**
- Tests minimal DNF conversion
- Verifies `EARLY_PRUNE=true` and `EARLY_PRUNE=false` produce identical minimal results
- Random parameters same as `equality_test`
- Reports progress every 1000 experiments
- **Runtime**: Can take hours depending on hardware

### Purpose

These tests provide confidence that:

1. **Correctness**: All optimization levels produce mathematically equivalent results
2. **Consistency**: The early pruning optimization doesn't miss any minimal terms
3. **Robustness**: Algorithm works correctly across wide range of input sizes and complexities

### Original C++ Implementation

These tests are direct ports from `cnf_to_dnf.h`:
- `cnf::tests::equality_test()` → `equality_test()`
- `cnf::tests::equality_test_minimal()` → `equality_test_minimal()`

### Notes

- Tests use random seeds from entropy for maximum coverage
- Progress is printed every 1000 iterations
- Any mismatch causes immediate test failure with diagnostic output
- The `--nocapture` flag is recommended to see progress output
