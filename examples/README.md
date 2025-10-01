# CNF to DNF Examples

These examples demonstrate the CNF (Conjunctive Normal Form) to DNF (Disjunctive Normal Form) conversion algorithms from the C++ implementation.

## Running Examples

```bash
# Run a specific example
cargo run --example test0

# Run all examples (note: test_very_hard may take a while)
cargo run --example test0
cargo run --example test1
cargo run --example test2
cargo run --example test3
cargo run --example test4
cargo run --example test5
cargo run --example test_very_hard
```

## Examples Description

### test0 (`test0.rs`)
**Basic CNF to DNF conversion**
- CNF: `(1|2) & (3|4)`
- DNF: `(1&3) | (2&3) | (1&4) | (2&4)`
- Simple 2-clause CNF demonstrating the basic conversion
- Expected output matches observed output

### test1 (`test1.rs`)
**6-variable CNF problem**
- CNF: `(1|2) & (1|3) & (3|4) & (2|5) & (4|6) & (5|6)`
- DNF: `(1&4&5) | (2&3&4&5) | (2&3&6) | (1&2&4&6) | (1&3&5&6)`
- More complex problem with 6 clauses
- Answer verified with Wolfram Alpha

### test2 (`test2.rs`)
**Named variables example**
- Uses variable names (A, B, C, D, E, F) instead of bit positions
- CNF: `(A|B) & (A|C) & (B|E) & (C|D) & (D|F) & (E|F)`
- DNF: `(A&B&D&F) | (A&C&E&F) | (A&D&E) | (B&C&D&E) | (B&C&F)`
- Demonstrates string-based variable handling

### test3 (`test3.rs`)
**Random generation test (16-bit)**
- 500 conjunctions
- 16-bit variables
- 8 disjunctions per conjunction
- Seed: 42 (reproducible)
- Tests scalability with larger CNF formulas

### test4 (`test4.rs`)
**Random generation test (32-bit)**
- 20 conjunctions
- 32-bit variables
- 8 disjunctions per conjunction
- Seed: 42 (reproducible)
- Outputs full DNF formula

### test5 (`test5.rs`)
**Minimal DNF with early pruning (64-bit)**
- 10 conjunctions
- 64-bit variables
- 8 disjunctions per conjunction
- Uses `convert_cnf_to_dnf_minimal` with early pruning
- Demonstrates optimization to find only minimal DNF terms
- Seed: 42 (reproducible)

### test_very_hard (`test_very_hard.rs`)
**Complex 35-clause problem**
- Real-world difficult problem discovered when generating popcnt_6_3
- 35 conjunctions with 60 variables
- Uses minimal DNF conversion
- **Warning**: This may take significant time to compute
- Demonstrates algorithm behavior on hard instances

## Performance Notes

- `test0`, `test1`, `test2`: Run instantly
- `test3`: May take a few seconds (500 conjunctions)
- `test4`: Completes quickly (only 20 conjunctions)
- `test5`: Uses early pruning optimization
- `test_very_hard`: Can take considerable time (35 complex conjunctions)

## Algorithm Details

All examples use the `convert_cnf_to_dnf` or `convert_cnf_to_dnf_minimal` functions from the `cnf_to_dnf` module:

- **Standard conversion**: Converts CNF to full DNF
- **Minimal conversion**: Finds only terms with minimum number of literals
- **Early pruning**: Optimization that discards non-minimal terms during computation

## Quine-McCluskey Examples

### qm_simple_3bit (`qm_simple_3bit.rs`)
**Simple 3-bit Boolean minimization**
- Minterms: [3, 4, 5, 6, 7]
- Function: output = 1 when input > 2
- Result: BC + A
- Great for learning the basics
- **Runtime**: Instant

### qm_essential_implicants (`qm_essential_implicants.rs`)
**Example with essential prime implicants**
- 4-bit input with minterms: [0, 2, 5, 6, 7, 8, 10, 12, 13, 14, 15]
- Has primary essential prime implicants
- Petrick's method not needed
- Verified with multiple online tools
- **Runtime**: Instant

### qm_petricks_method (`qm_petricks_method.rs`)
**Example requiring Petrick's method**
- 4-bit input with minterms: [0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]
- No primary essential prime implicants
- Demonstrates full Petrick's method algorithm
- Result: (A'D') + (A'C) + (BC') + (AB')
- **Runtime**: Instant

### qm_wolfram_verified (`qm_wolfram_verified.rs`)
**Wolfram Alpha verified example**
- Boolean Function 49148 (BF4 0xBFFC)
- Minterms: [1, 3, 5, 8, 10, 11, 13]
- Verified against Wolfram Alpha and multiple QM calculators
- Demonstrates real-world verification
- **Runtime**: Instant

### qm_krook_dataset (`qm_krook_dataset.rs`)
**Real-world QCA dataset**
- 5-bit input from social science research
- Variables: ES, QU, WS, WM, LP (Economic/Labor factors)
- 9 minterms from Krook QCA study
- Demonstrates application in Qualitative Comparative Analysis
- **Runtime**: Instant

### qm_parity_8bit (`qm_parity_8bit.rs`)
**8-bit parity function**
- Complex function: parity of sum of two 4-bit numbers
- 256 possible inputs, many minterms
- Shows algorithm handling complex Boolean functions
- **Runtime**: 1-2 seconds

### qm_comparison_14bit (`qm_comparison_14bit.rs`)
**14-bit comparison circuit (i1 > i2)**
- Compares two 7-bit numbers
- 16384 possible inputs
- Real-world comparison circuit design
- **Runtime**: ⚠️ 10-15+ seconds (large problem)

### qm_random_12bit (`qm_random_12bit.rs`)
**12-bit random Boolean function**
- 550 random minterms (seed 42)
- Tests algorithm on arbitrary functions
- Shows compression ratio
- Reproducible with fixed seed
- **Runtime**: 1-2 seconds

## Original C++ Source

These examples are direct ports from test functions:

**CNF to DNF examples** (from `cnf_to_dnf.h`):
- `test0()` → `cnf_2_dnf_0.rs`
- `test1()` → `cnf_2_dnf_1.rs`
- `test2()` → `cnf_2_dnf_2.rs`
- `test3()` → `cnf_2_dnf_3.rs`
- `test4()` → `cnf_2_dnf_4.rs`
- `test5()` → `cnf_2_dnf_5.rs`
- `test_very_hard()` → `cnf_2_dnf_very_hard.rs`

**Quine-McCluskey examples** (from `quine_mccluskey.h` test code):
- Simple 3-bit example → `qm_simple_3bit.rs`
- Essential implicants example → `qm_essential_implicants.rs`
- Petrick's method example → `qm_petricks_method.rs`
- Wolfram verified example → `qm_wolfram_verified.rs`
- Krook QCA dataset → `qm_krook_dataset.rs`
- 8-bit parity function → `qm_parity_8bit.rs`
- 14-bit comparison (i1 > i2) → `qm_comparison_14bit.rs`
- 12-bit random function → `qm_random_12bit.rs`
