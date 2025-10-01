# CNF to DNF Benchmarks

Comprehensive benchmarks comparing different SIMD optimization levels for CNF to DNF conversion.

## Running Benchmarks

### Quick Benchmark (recommended)
Run a quick benchmark to see relative performance:
```bash
cargo bench --bench cnf_to_dnf_bench
```

### Specific Benchmark Groups
Run only specific benchmark groups:

```bash
# Compare all optimization levels (X64, AVX2, AVX512 variants)
cargo bench --bench cnf_to_dnf_bench -- optimization_levels

# Test different problem sizes
cargo bench --bench cnf_to_dnf_bench -- problem_sizes

# Compare AVX512 variants with matching bit widths
cargo bench --bench cnf_to_dnf_bench -- avx512_variants

# Compare minimal DNF with/without early pruning
cargo bench --bench cnf_to_dnf_bench -- minimal_dnf

# Compare X64 vs AVX2 vs AVX512 for 64-bit problems
cargo bench --bench cnf_to_dnf_bench -- 64bit_comparison

# Test sparse vs dense conjunction patterns
cargo bench --bench cnf_to_dnf_bench -- conjunction_density
```

### Save Baseline for Comparison
```bash
# Save current results as baseline
cargo bench --bench cnf_to_dnf_bench -- --save-baseline main

# After changes, compare against baseline
cargo bench --bench cnf_to_dnf_bench -- --baseline main
```

## Benchmark Groups

### 1. `optimization_levels`
Compares all optimization levels on a small problem (8 variables, 5 conjunctions):
- **X64**: Scalar baseline implementation
- **AVX2_64bits**: AVX2 SIMD (4 elements per vector)
- **AVX512_8bits**: AVX512 with 8-bit elements (64 elements per vector)
- **AVX512_16bits**: AVX512 with 16-bit elements (32 elements per vector)
- **AVX512_32bits**: AVX512 with 32-bit elements (16 elements per vector)
- **AVX512_64bits**: AVX512 with 64-bit elements (8 elements per vector)

**Purpose**: Understand relative performance of each optimization level.

### 2. `problem_sizes`
Tests scaling behavior with different problem sizes:
- **8var_5conj**: 8 variables, 5 conjunctions (small)
- **16var_8conj**: 16 variables, 8 conjunctions (medium)
- **32var_10conj**: 32 variables, 10 conjunctions (large)

Compares X64 baseline vs optimal AVX512 variant for each size.

**Purpose**: Show how SIMD benefits scale with problem size.

### 3. `avx512_variants`
Tests each AVX512 variant with problem sizes matching their bit width:
- **8bit_8vars**: 8-bit SIMD with 8-variable problems
- **16bit_16vars**: 16-bit SIMD with 16-variable problems
- **32bit_32vars**: 32-bit SIMD with 32-variable problems
- **64bit_64vars**: 64-bit SIMD with 64-variable problems

**Purpose**: Show each variant performing optimally on appropriately-sized problems.

### 4. `minimal_dnf`
Compares minimal DNF conversion with and without early pruning optimization:
- **without_pruning**: Standard minimal DNF conversion
- **with_pruning**: Uses early pruning to discard non-minimal terms during computation

**Purpose**: Measure the effectiveness of the early pruning optimization.

### 5. `64bit_comparison`
Direct comparison for 64-bit problems:
- **X64_scalar**: Baseline scalar implementation
- **AVX2_64bits**: AVX2 with 4 elements per vector
- **AVX512_64bits**: AVX512 with 8 elements per vector

**Purpose**: Show SIMD speedup on largest supported problem size.

### 6. `conjunction_density`
Tests performance with different literal densities:
- **sparse_2lit**: 2 literals per conjunction
- **medium_4lit**: 4 literals per conjunction
- **dense_8lit**: 8 literals per conjunction

**Purpose**: Understand how conjunction complexity affects performance.

## Interpreting Results

### Expected Performance Characteristics

1. **SIMD Speedup**:
   - AVX512 should show 2-8x speedup over scalar for matching bit widths
   - AVX2 should show 1.5-4x speedup over scalar
   - Speedup increases with problem size (more data to vectorize)

2. **Bit Width Selection**:
   - Smaller bit widths (8-bit, 16-bit) process more elements per vector
   - Should be fastest when problem fits within bit width limit
   - Using wider bits than needed reduces SIMD advantage

3. **Early Pruning**:
   - Should show significant speedup when finding minimal DNF
   - More effective with larger problems and denser conjunctions

4. **CPU Support**:
   - If your CPU lacks AVX512, those benchmarks fall back to scalar
   - Check CPU features to understand your results:
   ```bash
   # Windows
   cargo run --example cnf_2_dnf_0  # Shows detected SIMD features

   # Linux
   cat /proc/cpuinfo | grep flags
   ```

### Reading Criterion Output

Criterion provides detailed statistics for each benchmark:
- **time**: Mean execution time with confidence interval
- **thrpt**: Throughput (operations per second)
- **change**: Percentage change vs previous run

Example output:
```
optimization_levels/small/AVX512_64bits
                        time:   [45.123 µs 45.456 µs 45.789 µs]
                        thrpt:  [219.52 elem/s 221.14 elem/s 222.76 elem/s]
                        change: [-15.234% -14.567% -13.901%] (p = 0.00 < 0.05)
                        Performance has improved.
```

## Hardware Requirements

- **AVX2 benchmarks**: Requires Intel Haswell (2013+) or AMD Excavator (2015+)
- **AVX512 benchmarks**: Requires Intel Skylake-X (2017+) or AMD Zen 4 (2022+)

If your CPU doesn't support these instructions, the benchmarks will automatically fall back to scalar implementation, showing no performance difference.

## Benchmark Configuration

All benchmarks use:
- **Fixed seed (42)**: Ensures reproducible results
- **Random CNF generation**: Realistic problem patterns
- **Black box calls**: Prevents compiler over-optimization
- **Throughput measurement**: Tracks elements/operations processed

Default Criterion settings:
- Warm-up time: 3 seconds
- Measurement time: 5 seconds
- Sample size: 100 iterations (minimum)

## Customizing Benchmarks

To add your own benchmark:

```rust
fn bench_my_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_test");

    let cnf = vec![0b0011, 0b1100]; // Your CNF

    group.bench_function("test_name", |b| {
        b.iter(|| {
            convert_cnf_to_dnf(
                black_box(&cnf),
                black_box(4),
                black_box(OptimizedFor::Avx512_64bits)
            )
        });
    });

    group.finish();
}

// Add to criterion_group!
criterion_group!(benches, bench_my_test, ...);
```

## Troubleshooting

**Benchmarks are slow**:
- This is expected for larger problem sizes
- Use `--bench ... -- --quick` for faster approximate results
- Or run specific smaller benchmark groups

**No SIMD speedup observed**:
- Check if your CPU supports AVX2/AVX512
- Verify SIMD features are detected (run unit test `test_simd_availability`)
- Very small problems may not benefit from SIMD overhead

**Results vary between runs**:
- Normal due to system load and thermal throttling
- Save baseline and compare for more stable results
- Close background applications during benchmarking
- Check CPU frequency scaling settings

## Further Analysis

Generate detailed reports:
```bash
# HTML report with graphs (opens in browser)
cargo bench --bench cnf_to_dnf_bench

# Reports are saved to: target/criterion/
```

Compare against saved baseline:
```bash
# Save baseline
cargo bench --bench cnf_to_dnf_bench -- --save-baseline before

# Make changes to code...

# Compare
cargo bench --bench cnf_to_dnf_bench -- --baseline before
```
