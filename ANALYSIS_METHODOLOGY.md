# Performance and Correctness Analysis Methodology

This document describes the methodology used to analyze and optimize critical functions in the QM algorithm implementation, using the `replace_complements` analysis as a reference case study.

## Case Study: `replace_complements` Analysis

### 1. Initial Observation

**Symptom**: Performance regression - new implementation 20x slower than old implementation
**Location**: `src/qm/implicant.rs::replace_complements`
**Context**: Function simplification removed masking operations to match C++ code

### 2. Hypothesis Formation

When you observe unexpected performance changes:

1. **Question correctness first**: "Are the results actually the same?"
2. **Don't assume faster = better**: Performance improvements that seem "too good to be true" often are
3. **Check reference implementations**: Look at C++ or other proven implementations

**Key insight**: 20x slowdown suggested algorithmic difference, not just micro-optimization

### 3. Testing Methodology

#### Step 1: Add Debug Comparison Code

Add code that compares old vs new implementations side-by-side:

```rust
// In the call site (quine_mccluskey.rs)
let raw_combined_old = Implicant::<E>::replace_complements(raw_i, raw_j, self.variables);
let raw_combined_new = Implicant::<E>::replace_complements_new(raw_i, raw_j, self.variables);

#[cfg(debug_assertions)]
{
    if raw_combined_old != raw_combined_new {
        println!("MISMATCH!");
        // Print detailed info...
    }
}

let raw_combined = raw_combined_old; // Use old for now
```

**Why this works**:
- Both functions run on identical inputs
- Differences caught immediately
- Debug-only, zero overhead in release builds
- Can switch between implementations easily

#### Step 2: Add Rich Debug Output

When mismatches are found, print enough context to understand the problem:

```rust
if raw_combined_old != raw_combined_new {
    let mask = (E::Value::one() << self.variables) - E::Value::one();

    // Separate data and don't-care bits for readability
    let data_old = (raw_combined_old & mask).to_u64();
    let dc_old = (raw_combined_old >> self.variables).to_u64();
    let data_new = (raw_combined_new & mask).to_u64();
    let dc_new = (raw_combined_new >> self.variables).to_u64();

    println!("MISMATCH at level {}!", level);
    println!("  Input i: data={:0width$b}, dc={:0width$b}",
             (raw_i & mask).to_u64(),
             (raw_i >> self.variables).to_u64(),
             width=self.variables);
    println!("  Input j: data={:0width$b}, dc={:0width$b}",
             (raw_j & mask).to_u64(),
             (raw_j >> self.variables).to_u64(),
             width=self.variables);
    println!("  OLD: data={:0width$b}, dc={:0width$b}", data_old, dc_old, width=self.variables);
    println!("  NEW: data={:0width$b}, dc={:0width$b}", data_new, dc_new, width=self.variables);
}
```

**What to print**:
- **Inputs**: Both operands with full context
- **Outputs**: Both results with clear labeling
- **Context**: Algorithm level, iteration count, etc.
- **Format**: Binary representation for bit operations

#### Step 3: Run Comprehensive Tests

Run tests that exercise multi-level combinations:

```bash
# Simple tests might pass even with bugs
cargo run --example qm_simple_3bit  # Might not catch the bug

# Complex tests reveal issues
cargo run --example qm_petricks_method  # Multi-level combinations!
```

**Why multi-level matters**: Bugs in bit operations often only manifest when:
- Combining already-combined implicants (level 2+)
- Existing don't-cares need to be preserved
- Edge cases with specific bit patterns

### 4. Root Cause Analysis

#### Step 1: Examine the Difference Pattern

From the debug output:
```
Input i: data=0000, dc=0100  (represents "0X00")
Input j: data=0010, dc=0100  (represents "0X10")
OLD: data=0010, dc=0010  (produces "0X10")
NEW: data=0010, dc=0110  (produces "0XX0")
```

**Observation**: The NEW version preserves the input don't-care (bit 2), while OLD loses it!

#### Step 2: Compare with Reference Implementation

Read the C++ code carefully:

```cpp
template <typename T>
[[nodiscard]] constexpr T replace_complements(T a, T b) noexcept
{
    const T neq = a ^ b;
    if constexpr (MAX_16_BITS) {
        return a | neq | (neq << 16);  // Uses full 'a', not masked!
    }
    else {
        return a | neq | (neq << 32);
    }
}
```

**Key discovery**: C++ uses `a | neq`, not `(a & mask) | neq`

#### Step 3: Trace the Logic

**OLD implementation** (WRONG):
```rust
let mask = (E::Value::one() << variables) - E::Value::one();
let data_a = a & mask;           // Extracts ONLY data bits, loses don't-cares!
let neq = (a ^ b) & mask;        // Masks the XOR
data_a | neq | (neq << variables)  // Result only has new don't-care
```

**NEW implementation** (CORRECT):
```rust
let neq = a ^ b;                 // No mask - preserves upper bits
a | neq | (neq << variables)     // 'a' includes don't-cares in upper bits!
```

**Why NEW is correct**:
- `a` has format: `[data bits][don't-care mask bits]`
- `a | neq` preserves upper bits (existing don't-cares)
- `neq << variables` adds new don't-care bit
- Result: old don't-cares + new don't-care = correct

**Why OLD was wrong**:
- `a & mask` strips away don't-care bits
- Only the newly combined bit becomes don't-care
- Previous don't-cares lost forever!

### 5. Mathematical Verification

Work through a concrete example:

```
Variables: 4 bits
a = 0000|0100  (data=0000, dc=0100) represents "0X00"
b = 0010|0100  (data=0010, dc=0100) represents "0X10"

They differ at bit 1 (value 0010)

OLD method:
  data_a = a & 0xF = 0000
  neq = (a ^ b) & 0xF = 0010
  result = 0000 | 0010 | (0010 << 4) = 0010 | 0x20 = 0x22
  In binary: 0010|0010 = "0X10" ❌ Lost the original X!

NEW method:
  neq = a ^ b = (0000|0100) ^ (0010|0100) = 0010|0000
  result = (0000|0100) | (0010|0000) | (0010 << 4)
         = 0010|0100 | 0x20 = 0010|0110
  In binary: 0010|0110 = "0XX0" ✓ Preserves both X's!
```

### 6. Verification Steps

#### Step 1: Switch to New Implementation
```rust
let raw_combined = raw_combined_new;  // Changed from old
```

#### Step 2: Run All Tests
```bash
cargo test
```

#### Step 3: Run Examples
```bash
cargo run --example qm_petricks_method
cargo run --example qm_wolfram_verified
```

#### Step 4: Check for Semantic Correctness

Even if tests pass, verify the algorithm is semantically correct:
- Are prime implicants correctly identified?
- Are all minterms covered?
- Does the result match known-good outputs?

### 7. Performance Impact Analysis

**Expected**: New version might be slower because it's MORE correct

**Why**:
- Preserving don't-cares creates more general implicants
- More general implicants change algorithm exploration path
- Old buggy version accidentally pruned search space
- This is NOT an optimization - it's a bug fix!

**Key lesson**: Sometimes "slower" is correct!

### 8. Code Cleanup

Once verified correct:

1. **Remove old implementation**: Don't keep buggy code around
2. **Remove debug comparison**: Keep code clean
3. **Update documentation**: Explain why the implementation is this way
4. **Add explanatory comments**: Help future maintainers

```rust
/// Fast combine using raw encoding (like C++ replace_complements)
/// Only called after is_gray_code returns true (don't-care masks are identical)
/// Preserves existing don't-cares and marks the differing bit as don't-care
#[inline]
pub fn replace_complements(a: E::Value, b: E::Value, variables: usize) -> E::Value {
    let neq = a ^ b;
    // C++ code: return a | neq | (neq << variables);
    // This preserves don't-cares in upper bits and marks new don't-care
    a | neq | (neq << variables)
}
```

## General Analysis Checklist

Use this checklist for any optimization or refactoring:

### Before Making Changes
- [ ] Understand what the function SHOULD do (algorithm spec)
- [ ] Check reference implementations (C++, papers, etc.)
- [ ] Identify edge cases and multi-level scenarios
- [ ] Have comprehensive test coverage

### During Analysis
- [ ] Add debug comparison code (old vs new)
- [ ] Print rich, formatted output (binary for bit ops)
- [ ] Test on complex examples, not just simple ones
- [ ] Trace through concrete examples by hand
- [ ] Verify mathematical correctness

### After Changes
- [ ] Run ALL tests (unit + integration + examples)
- [ ] Compare output with reference implementations
- [ ] Benchmark if performance-critical
- [ ] Document WHY the implementation is this way
- [ ] Clean up debug code

## Red Flags to Watch For

1. **"Too good to be true" performance improvements**
   - If removing operations gives huge speedup, check correctness
   - Real optimizations rarely give >2-3x speedup

2. **Tests passing doesn't mean it's correct**
   - Tests might not cover multi-level scenarios
   - Final results might be equivalent even if intermediate steps differ
   - Need semantic verification, not just test coverage

3. **Divergence from reference implementation**
   - If C++ does X and Rust does Y, understand WHY
   - Don't assume Rust needs to be different
   - Port the algorithm, don't rewrite it (unless you have a good reason)

4. **Silent changes in intermediate results**
   - Algorithm correctness depends on intermediate values
   - Use debug assertions to catch these
   - Compare at every step, not just final output

## Tools and Techniques

### Debug Assertions
```rust
#[cfg(debug_assertions)]
{
    // Expensive checks only in debug builds
    assert_eq!(expected, actual);
}
```

### Comparison Testing
```rust
let result_old = old_implementation();
let result_new = new_implementation();

#[cfg(debug_assertions)]
if result_old != result_new {
    panic!("Mismatch: old={:?}, new={:?}", result_old, result_new);
}
```

### Binary Formatting
```rust
// Print with leading zeros, specific width
println!("Value: {:0width$b}", value, width=num_bits);

// Separate into logical parts
let data = value & mask;
let metadata = value >> shift;
println!("data={:016b}, meta={:016b}", data, metadata);
```

### Incremental Testing
```bash
# Start simple
cargo test test_simple

# Gradually increase complexity
cargo run --example simple_case
cargo run --example complex_case
cargo run --example edge_cases
```

## Conclusion

The key to successful performance optimization is:

1. **Verify correctness FIRST**: Use comparison testing
2. **Understand reference implementations**: Don't assume you know better
3. **Test on complex cases**: Simple cases often hide bugs
4. **Document your reasoning**: Future you will thank you
5. **Accept that "slower" might be "correct"**: Performance is useless if results are wrong

When in doubt, trust the reference implementation and verify your understanding of the algorithm.
