# SIMD Coverage Matrix Implementation

## Overview

The Quine-McCluskey algorithm requires checking which minterms are covered by each prime implicant, creating a coverage matrix. This is a performance-critical operation that grows with O(n × m) where n = number of prime implicants and m = number of minterms.

This implementation provides an AVX-512 SIMD-accelerated version that processes 512 minterm-implicant pairs simultaneously, achieving **5.93× speedup** over the scalar implementation.

## Architecture

### Components

**Location**: `src/qm/simd_coverage.rs`

1. **`should_use_simd()`** - Heuristic to determine if SIMD is beneficial
   - Checks CPU feature support (AVX-512F, GFNI)
   - Enforces minimum problem size (≥1024 checks) to amortize overhead
   - Currently limited to 4-bit values (extensible)

2. **`build_coverage_matrix_simd_4bit()`** - Main SIMD implementation
   - Processes 512 minterms per batch
   - Uses bit-plane transposition for parallel processing
   - Pads inputs to multiples of 512

3. **`check_coverage_batch_4bit()`** - Core AVX-512 logic
   - Broadcasts implicant/mask to 512 values
   - Performs bit-plane separation using GFNI
   - Executes coverage check via generated function
   - Returns 512-bit result vector

4. **`extract_implicant_representation()`** - Converts implicant to bit pattern
   - Extracts fixed value bits (0s and 1s)
   - Extracts don't-care mask (1 = don't care, 0 = must match)

### Integration with bitwise-simd

Uses the `_mm512_covers_4_4_4_1` generated function from the bitwise-simd crate:

```rust
use bitwise_simd::bit_plane::*;
use bitwise_simd::generated::_mm512_covers_4_4_4_1::_mm512_covers_4_4_4_1;
```

The coverage function implements:
```
covers(implicant, mask, minterm) =
    (minterm & ~mask) == (implicant & ~mask)
```

## Bit-Plane Transposition

### The Challenge

Standard representation stores each value in consecutive bytes:
- Value 0 at byte 0, value 1 at byte 1, ..., value 511 at byte 511

But AVX-512 operates on columns of bits across multiple values simultaneously.

### The Solution

**Bit-plane separation** reorganizes data so each bit position forms a plane:
- Plane 0: bit 0 from all 512 values
- Plane 1: bit 1 from all 512 values
- Plane 2: bit 2 from all 512 values
- Plane 3: bit 3 from all 512 values

This allows processing all 512 values in parallel with single instructions.

### Memory Layout

**Key Insight**: 8 input registers (512 bytes) are interleaved, not processed consecutively.

```
Input bytes:   [0] [1] [2] ... [63] [64] [65] ... [127] ... [448] ... [511]
Register:       R0  R0  R0  ... R0   R1   R1   ... R1   ...  R7   ...  R7
                └─────────────────┘   └──────────────┘        └──────────┘
                   64 bytes each        64 bytes each        64 bytes each
```

After bit-plane separation:
- `plane[0][0]` (first output byte) contains bit 0 from: bytes 0, 64, 128, 192, 256, 320, 384, 448
- `plane[0][1]` (second output byte) contains bit 0 from: bytes 1, 65, 129, 193, 257, 321, 385, 449

**Striped Layout Formula**:
- Input index `i` → Output byte `i % 64`, bit `i / 64`

### Example

For values 0-7 at positions 0, 64, 128, 192, 256, 320, 384, 448:

```
Values:  0    1    2    3    4    5    6    7
Bits:  0000 0001 0010 0011 0100 0101 0110 0111

After bit-plane separation (first byte of each plane):
plane[0][0] = 0b01010101  (bit 0: 0,1,0,1,0,1,0,1)
plane[1][0] = 0b00110011  (bit 1: 0,0,1,1,0,0,1,1)
plane[2][0] = 0b00001111  (bit 2: 0,0,0,0,1,1,1,1)
plane[3][0] = 0b00000000  (bit 3: 0,0,0,0,0,0,0,0)
```

## Algorithm Flow

```rust
for each batch of 512 minterms {
    // 1. Load data into 8 ZMM registers (64 bytes each)
    load_512_bytes_into_8_registers();

    // 2. Bit-plane separate all inputs (4-bit)
    bps_gfni_8to4(&minterm_regs, &mut minterm_planes);
    bps_gfni_8to4(&implicant_regs, &mut implicant_planes);
    bps_gfni_8to4(&mask_regs, &mut mask_planes);

    // 3. Arrange input for generated function
    input[0..4]   = minterm_planes;
    input[4..8]   = mask_planes;
    input[8..12]  = implicant_planes;

    // 4. Execute coverage check for 512 values
    _mm512_covers_4_4_4_1(&input, &mut output);

    // 5. Decode results (striped layout!)
    for i in 0..512 {
        byte_idx = i % 64;
        bit_idx = i / 64;
        covered = (output[byte_idx] >> bit_idx) & 1;
        coverage_matrix[pi_idx][minterm_idx] = covered;
    }
}
```

## Performance Analysis

### Benchmark Results

**Configuration:**
- 100 prime implicants
- 10,000 minterms
- 1 million coverage checks total
- 100 iterations averaged

**Results:**

| Implementation | Time | Throughput | Speedup |
|----------------|------|------------|---------|
| Scalar | 7.69 ms | 130 million checks/sec | 1.0× |
| SIMD (AVX-512) | 1.30 ms | 770 million checks/sec | **5.93×** |

### Performance Breakdown

**Theoretical Maximum**: 512× (processing 512 values per instruction)

**Actual**: 5.93× (1.2% efficiency)

**Overhead Sources:**
1. **Bit-plane transposition**: ~40% of SIMD time
   - Separate: Convert 8×64 bytes → 4×64 bytes bit-planes
   - Compose: Convert results back (not needed for coverage matrix)

2. **Memory bandwidth**: Processing faster than data can be loaded/stored

3. **Padding overhead**: Must process in 512-value chunks even if fewer minterms

4. **Simple operation**: Coverage check is just bit pattern matching
   - Scalar: Very fast (branch-free)
   - SIMD: Overhead dominates for simple operations

### When SIMD Wins

✅ **Good cases:**
- Large problems (>10,000 minterms)
- Multiple prime implicants (amortizes setup)
- Memory-bound workloads

❌ **Poor cases:**
- Small problems (<1,000 checks) - threshold prevents activation
- Very few prime implicants
- CPU without AVX-512F/GFNI

### Comparison with Other Operations

| Operation | SIMD Speedup | Notes |
|-----------|--------------|-------|
| CNF to DNF (32-bit) | 301× | Complex operation, SIMD dominates |
| CNF to DNF (64-bit) | 4.0× | Memory-bound, similar to coverage |
| **Coverage Matrix** | **5.93×** | Simple operation, overhead matters |
| Popcnt | 10-20× | Moderate complexity |

Coverage matrix gets modest but significant speedup, similar to other memory-intensive operations.

## Usage

### Automatic (Recommended)

The QMC algorithm automatically uses SIMD when beneficial:

```rust
use qm_agent::minimize_function;

let minterms = vec![/* ... many minterms ... */];
let result = minimize_function(&minterms, None, 4);
// SIMD automatically used if:
// - CPU has AVX-512F + GFNI
// - Problem size ≥ 1024 checks
// - Variables ≤ 4 (extensible)
```

### Manual (Testing/Benchmarking)

```rust
use qm_agent::qm::simd_coverage;
use qm_agent::qm::encoding::Enc16;
use qm_agent::qm::implicant::Implicant;

unsafe {
    let coverage = simd_coverage::build_coverage_matrix_simd_4bit(
        &prime_implicants,
        &minterms
    );
}
```

### Running the Benchmark

```bash
cargo run --release --example benchmark_simd_coverage
```

## Implementation Details

### Input Preparation

1. **Padding**: Minterms padded to multiple of 512
   ```rust
   let padded_size = ((num_mt + 511) / 512) * 512;
   padded_minterms.resize(padded_size, 0);
   ```

2. **Broadcasting**: Implicant and mask replicated to 512 values
   ```rust
   let implicant_bytes = vec![implicant_value; 512];
   let mask_bytes = vec![dont_care_mask; 512];
   ```

3. **Register Loading**: 512 bytes loaded into 8 ZMM registers
   ```rust
   for reg in 0..8 {
       minterm_regs[reg] = _mm512_loadu_si512(
           minterms[reg * 64..].as_ptr() as *const __m512i
       );
   }
   ```

### Coverage Logic

The coverage check implements:
```rust
fn covers(implicant: u8, mask: u8, minterm: u8) -> bool {
    (minterm & !mask) == (implicant & !mask)
}
```

- `mask`: Bit = 1 means don't care, 0 means must match
- `implicant`: The fixed bit values (0s and 1s)
- `minterm`: The value being checked

Example: Implicant `0X1X` (implicant=0b0010, mask=0b0101)
- Covers minterm 2 (0b0010): ✓ (2 & 0b1010) == (2 & 0b1010)
- Covers minterm 3 (0b0011): ✓ (3 & 0b1010) == (2 & 0b1010)
- Does not cover 0 (0b0000): ✗ (0 & 0b1010) != (2 & 0b1010)

### Result Decoding

Critical: Must use striped layout!

```rust
// WRONG - consecutive layout
let byte_idx = i / 8;
let bit_idx = i % 8;

// CORRECT - striped layout
let byte_idx = i % 64;
let bit_idx = i / 64;
```

## Testing

### Unit Tests

Location: `tests/test_simd_coverage.rs`

1. **`test_coverage_correctness_4bit`** - Verifies scalar coverage logic
2. **`test_simd_coverage_matrix`** - Compares SIMD vs expected results
3. **`test_simd_threshold_detection`** - Validates activation threshold
4. **`test_petricks_method_4bit_small`** - Integration test

Run tests:
```bash
cargo test --test test_simd_coverage
```

### Benchmark

Location: `examples/benchmark_simd_coverage.rs`

Features:
- Generates random test data
- Verifies correctness before benchmarking
- Measures warmup and steady-state performance
- Reports throughput, latency, and speedup
- Calculates efficiency vs theoretical max

## Future Enhancements

### Extensibility

Current implementation limited to 4-bit values. To extend:

1. **Add 8-bit support**: Use `bps_gfni_8to8` instead of `8to4`
2. **Generate larger functions**: Create `_mm512_covers_8_8_8_1` via ABC
3. **Update threshold**: Adjust `should_use_simd()` for different bit widths

### Optimization Opportunities

1. **Eliminate transpose overhead**: Process data in bit-plane format throughout
2. **Vectorize implicant processing**: Check multiple implicants simultaneously
3. **Reduce padding waste**: Use variable-length SIMD operations
4. **Pipeline batches**: Overlap computation and memory access

### Hardware Support

Required features:
- **AVX-512F**: Foundation instructions
- **GFNI**: Galois Field operations for bit-plane separation

Supported CPUs:
- Intel Ice Lake (10th gen) and newer
- AMD Zen 4 and newer (limited AVX-512)

## References

- **Bit-plane transposition**: See `bitwise-simd` crate documentation
- **Coverage bug fix**: `SIMD_COVERAGE_FIX.md` in bitwise-rust-agent repo
- **Generated functions**: `bitwise-simd/generated/rust/`
- **Benchmark code**: `examples/benchmark_simd_coverage.rs`

## Key Takeaways

1. **5.93× speedup** is excellent for a memory-bound operation
2. **Striped layout** is critical for correct bit-plane decoding
3. **Threshold-based activation** prevents slowdown on small problems
4. **Automatic fallback** ensures correctness on all CPUs
5. **Future extensibility** to wider bit widths and larger problems
