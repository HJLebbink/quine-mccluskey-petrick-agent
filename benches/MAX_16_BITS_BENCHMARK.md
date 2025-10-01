# MAX_16_BITS Performance Benchmark

This benchmark measures the performance difference between `MAX_16_BITS = true` (16-bit encoding) and `MAX_16_BITS = false` (32-bit encoding) in `src/qm/classic.rs`.

## How to Run

The benchmark must be run twice - once for each configuration:

### Step 1: Benchmark with 32-bit encoding (current default)
```bash
# Ensure MAX_16_BITS = false in src/qm/classic.rs (line 7)
cargo bench --bench max_16_bits_bench -- --save-baseline 32bit
```

### Step 2: Change to 16-bit encoding
Edit `src/qm/classic.rs` line 7:
```rust
// Change from:
pub const MAX_16_BITS: bool = false;

// To:
pub const MAX_16_BITS: bool = true;
```

### Step 3: Benchmark with 16-bit encoding
```bash
cargo bench --bench max_16_bits_bench -- --save-baseline 16bit
```

### Step 4: Compare results
```bash
cargo install critcmp  # If not already installed
critcmp 32bit 16bit
```

## What Gets Measured

### 1. `reduce_minterms` - Core Algorithm
- Tests the optimized O(n×m) reduction algorithm
- Variable counts: 4, 8, 10, 12, 14, 16
- Measures throughput in minterms/second
- **Key metric**: This is the bottleneck in QM algorithm

### 2. `reduce_minterms_classic` - Classic O(n²) Algorithm
- Tests the classic comparison-based algorithm
- Variable counts: 4, 6, 8, 10 (smaller due to O(n²))
- Provides baseline comparison

### 3. `minterm_to_string` - Bit Operations
- Tests single minterm to string conversion
- Variable counts: 4, 8, 12, 16, 20, 24, 28, 32
- Measures impact of bit position calculations
- **Note**: For >16 vars, both modes use 32-bit encoding

### 4. `minterms_to_string` - Batch Conversion
- Tests converting multiple minterms to strings
- Measures formatting overhead
- Tests loop efficiency

### 5. `minterm_set` - Cache Efficiency
- Tests MintermSet creation and population
- Variable counts: 4, 8, 12, 16
- Measures cache locality impact of array size:
  - 16-bit: 33 buckets
  - 32-bit: 65 buckets

### 6. `minterm_set_get` - Access Patterns
- Tests retrieval from MintermSet buckets
- Measures memory access patterns
- Tests cache line utilization

### 7. `full_reduction` - End-to-End
- Reduces minterms until fixed point
- Most realistic workload
- Variable counts: 4, 6, 8, 10, 12
- **Key metric**: Total time for complete minimization

## Expected Results

### Scenarios where 16-bit mode WINS:
- **Variable count ≤ 16**: All operations benefit
- **MintermSet operations**: Better cache locality (33 vs 65 buckets)
- **Small problems**: Cache effects dominate

### Scenarios where both modes are EQUAL:
- **Variable count > 16**: Both use 32-bit encoding
- **Very large problems**: Algorithm complexity dominates

### Predicted Speedup (16-bit vs 32-bit):
- `minterm_set`: **10-20%** (smaller array, better cache)
- `minterm_to_string`: **5-10%** (fewer bit checks)
- `reduce_minterms`: **5-15%** (cache + bit operations)
- `full_reduction`: **8-18%** (compound effect)

## Interpreting Results

Look for these key indicators:

1. **Time per element**: Lower is better
2. **Throughput**: Higher is better (elements/second)
3. **Consistency**: Lower variance indicates stable performance

Example output:
```
reduce_minterms/optimized/12_vars_1638_terms
                        time:   [1.2345 ms 1.2456 ms 1.2567 ms]
                        thrpt:  [1.3034 Melem/s 1.3154 Melem/s 1.3274 Melem/s]
```

## Hardware Dependencies

Performance will vary based on:
- **CPU cache sizes**: Larger L1/L2 cache reduces benefit
- **Memory bandwidth**: Faster RAM reduces cache benefit
- **CPU architecture**: Modern CPUs with larger caches may see smaller gains

## Recommendation

After running benchmarks:
- If 16-bit mode shows **>10% speedup** on typical workloads → Consider auto-selection based on variable count
- If 16-bit mode shows **5-10% speedup** → Document as optimization for small problems
- If 16-bit mode shows **<5% speedup** → Overhead of maintaining two code paths may not be worth it

## Related Files

- `src/qm/classic.rs` - Contains MAX_16_BITS constant and affected functions
- `benches/max_16_bits_bench.rs` - This benchmark suite
- `benches/RESULTS.md` - Benchmark results (update after running)
