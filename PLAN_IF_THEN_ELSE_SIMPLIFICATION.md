# Plan: If-Then-Else Simplification using Quine-McCluskey

## Overview

Add support to simplify complex if-then-else conditions in programming languages using the Quine-McCluskey algorithm to produce minimal Boolean expressions.

## Use Case Examples

### Example 1: Simple Boolean Simplification
**Before:**
```rust
if (a && b) || (a && c) || (b && c) {
    return true;
} else {
    return false;
}
```

**After QM Minimization:**
```rust
if (a && b) || (a && c) {  // Simplified: removed redundant (b && c)
    return true;
} else {
    return false;
}
```

### Example 2: Multi-Branch Conditions
**Before:**
```rust
if a && b && c {
    return 1;
} else if a && b && !c {
    return 1;
} else if a && !b && c {
    return 2;
} else if !a && b && c {
    return 2;
} else {
    return 0;
}
```

**After QM Minimization:**
```rust
if a && b {              // Covers: a && b && c, a && b && !c
    return 1;
} else if c && (a || b) { // Covers: a && !b && c, !a && b && c
    return 2;
} else {
    return 0;
}
```

### Example 3: Dead Code Detection
**Before:**
```rust
if (a && b) || (a && !b) {  // Always true when a is true
    println!("Path 1");
} else if !a {
    println!("Path 2");
} else {
    println!("Dead code!");  // Unreachable!
}
```

**After Analysis:**
```rust
if a {
    println!("Path 1");
} else {  // !a
    println!("Path 2");
}
// Dead code removed
```

## Architecture

### Component Structure

```
src/
├── qm/                    # Existing QM module
├── cnf_dnf/              # Existing CNF/DNF module
└── simplify/             # New simplification module
    ├── mod.rs            # Module interface
    ├── parser.rs         # Parse if-then-else conditions
    ├── analyzer.rs       # Analyze conditions and extract truth tables
    ├── optimizer.rs      # Apply QM minimization
    ├── codegen.rs        # Generate optimized code
    └── types.rs          # Data structures
```

### Data Flow

```
Source Code
    ↓
[Parser] → AST (Abstract Syntax Tree)
    ↓
[Analyzer] → Truth Table + Output Mapping
    ↓
[QM Minimizer] → Minimal Boolean Expressions (per output)
    ↓
[Code Generator] → Optimized If-Then-Else
    ↓
Simplified Code
```

## Core Data Structures

### 1. Condition AST
```rust
pub enum Condition {
    Variable(String),              // a, b, x, flag
    Not(Box<Condition>),           // !a
    And(Box<Condition>, Box<Condition>),  // a && b
    Or(Box<Condition>, Box<Condition>),   // a || b

    // For future: comparison operators
    Equals(Box<Expr>, Box<Expr>),     // x == 5
    LessThan(Box<Expr>, Box<Expr>),   // x < 10
    // ... other comparisons
}
```

### 2. Branch Structure
```rust
pub struct Branch {
    pub condition: Condition,
    pub output: OutputValue,
}

pub enum OutputValue {
    ReturnValue(String),    // return 1, return "hello"
    Statement(String),      // println!("foo")
    Block(String),          // { complex code }
}
```

### 3. Truth Table Representation
```rust
pub struct ConditionalTruthTable {
    pub variables: Vec<String>,        // ["a", "b", "c"]
    pub variable_count: usize,         // 3
    pub output_map: HashMap<u32, OutputValue>,  // minterm → output
    pub dont_cares: Vec<u32>,          // Unreachable conditions
}
```

### 4. Simplified Result
```rust
pub struct SimplificationResult {
    pub original_branches: usize,
    pub simplified_branches: usize,
    pub removed_dead_code: Vec<String>,
    pub optimized_code: String,
    pub analysis: SimplificationAnalysis,
}

pub struct SimplificationAnalysis {
    pub condition_coverage: f64,       // % of input space covered
    pub dead_branches: Vec<String>,    // Unreachable code detected
    pub overlapping_conditions: Vec<String>,  // Redundant checks
    pub complexity_reduction: f64,     // % improvement
}
```

## Implementation Phases

### Phase 1: Boolean-Only Parser (Simplest Case)
**Scope:** Handle pure Boolean conditions with &&, ||, ! operators only

**Input Format:**
```
if (a && b) || (c && !d):
    return 1
elif (!a && b):
    return 2
else:
    return 0
```

**Components:**
- `BooleanParser`: Parse Boolean expressions
- `TruthTableExtractor`: Generate truth table from branches
- `OutputMapper`: Map each minterm to its output value
- `QMOptimizer`: Apply existing QM algorithm per output
- `SimpleCodeGen`: Generate simplified if-then-else

**Output:**
```rust
// Analysis
Variables: [a, b, c, d]
Truth table: 16 rows (2^4)
Output mapping:
  - Return 1: minterms [...]
  - Return 2: minterms [...]
  - Return 0: don't cares (default)

// Minimized
if (a && b) || c {
    return 1;
} else if !a && b {
    return 2;
} else {
    return 0;
}
```

### Phase 2: Multi-Output Optimization
**Scope:** Optimize when multiple branches have same output

**Key Algorithm:**
1. Group branches by output value
2. For each unique output:
   - Collect all minterms that produce this output
   - Run QM minimization
   - Generate minimal condition
3. Order conditions by specificity (most specific first)
4. Generate optimized if-then-else chain

**Example:**
```rust
// Input: 5 branches
if a && b && c     { return 1 }  // minterm 7
if a && b && !c    { return 1 }  // minterm 6
if a && !b && c    { return 2 }  // minterm 5
if !a && b && c    { return 2 }  // minterm 3
else               { return 0 }  // all other minterms

// QM Grouping:
// Output 1: minterms [7, 6] → QM → a && b
// Output 2: minterms [5, 3] → QM → c && (a ^ b)  [using XOR]
// Output 0: default (don't cares)

// Optimized:
if a && b {
    return 1;
} else if c && (a != b) {
    return 2;
} else {
    return 0;
}
```

### Phase 3: Dead Code Detection
**Scope:** Identify unreachable branches using QM analysis

**Detection Strategy:**
1. Build truth table from conditions
2. Track which minterms are covered by each branch (in order)
3. If a branch's minterms are all covered by earlier branches → dead code
4. Report uncovered minterms → missing test cases

**Example:**
```rust
// Input
if a && b           { return 1 }  // covers [3]
if a || b           { return 2 }  // covers [1, 2, 3] - but 3 already covered!
if !a && !b         { return 3 }  // covers [0]

// Analysis:
// Branch 1: covers minterm 3 (a=1, b=1)
// Branch 2: tries to cover [1,2,3] but 3 is already covered by branch 1
//           Only effectively covers [1, 2]
// Branch 3: covers minterm 0 (a=0, b=0)
// All minterms covered: ✓

// Warning: Branch 2 has overlapping condition with Branch 1
```

### Phase 4: Comparison Operator Support
**Scope:** Handle ==, !=, <, >, <=, >= with constants

**Approach:**
- For bounded integer comparisons, enumerate possible values
- Use bit-level representation for ranges
- Example: `x < 4` where x is 3-bit → minterms [0,1,2,3]

**Example:**
```rust
// Input
if x == 0      { return "zero" }
if x == 1      { return "one" }
if x == 2      { return "two" }
if x < 4       { return "small" }  // Overlaps with above!
else           { return "large" }

// Analysis (x is 3-bit: 0-7):
// x == 0: minterm 0
// x == 1: minterm 1
// x == 2: minterm 2
// x < 4:  minterms [0,1,2,3] - overlaps with 0,1,2!
// else:   minterms [4,5,6,7]

// Warning: "x < 4" branch is unreachable (all its cases handled earlier)
```

### Phase 5: Multi-Variable Comparison
**Scope:** Handle conditions like `(a < b) && (b < c)`

**Challenges:**
- Infinite/large input space
- Need to symbolic reasoning or constraint solving
- May require SMT solver integration

**Possible Approaches:**
1. **Bounded enumeration**: If variables have small domains (e.g., 4-bit ints)
2. **Symbolic analysis**: Use constraint solving (Z3, CVC4)
3. **Heuristic simplification**: Apply known algebraic rules

**Phase 5 Decision:** May be out of scope for initial version. Focus on Phases 1-4.

## CLI Interface

### Subcommand: `simplify`
```bash
# From file
qm-agent simplify --file conditions.txt --lang rust

# From stdin
qm-agent simplify --lang python << EOF
if a && b:
    return 1
elif c || d:
    return 2
else:
    return 0
EOF

# Show analysis only (no code generation)
qm-agent simplify --file conditions.txt --analyze-only

# Output formats
qm-agent simplify --file conditions.txt --format json
qm-agent simplify --file conditions.txt --format rust
qm-agent simplify --file conditions.txt --format c
```

### Input File Format Options

**Option 1: Pseudo-code (Language-agnostic)**
```
variables: a, b, c
branches:
  - condition: a && b
    output: return 1
  - condition: c || !a
    output: return 2
  - default: return 0
```

**Option 2: Python-like Syntax**
```python
if a and b:
    return 1
elif c or not a:
    return 2
else:
    return 0
```

**Option 3: JSON**
```json
{
  "variables": ["a", "b", "c"],
  "branches": [
    {"condition": "a && b", "output": "return 1"},
    {"condition": "c || !a", "output": "return 2"}
  ],
  "default": "return 0"
}
```

## Code Generation

### Target Languages

**Priority 1: Rust**
```rust
if condition1 {
    return value1;
} else if condition2 {
    return value2;
} else {
    return default;
}
```

**Priority 2: C/C++**
```c
if (condition1) {
    return value1;
} else if (condition2) {
    return value2;
} else {
    return default;
}
```

**Priority 3: Python**
```python
if condition1:
    return value1
elif condition2:
    return value2
else:
    return default
```

### Code Generator Design
```rust
pub trait CodeGenerator {
    fn generate_condition(&self, expr: &BooleanExpr) -> String;
    fn generate_branch(&self, condition: &str, body: &str) -> String;
    fn generate_else(&self, body: &str) -> String;
}

pub struct RustCodeGen;
pub struct CCodeGen;
pub struct PythonCodeGen;
```

## Advanced Features

### 1. Switch Statement Generation
When all conditions are equality checks on same variable:
```rust
// Instead of:
if x == 1      { return "one" }
else if x == 2 { return "two" }
else if x == 3 { return "three" }
else           { return "unknown" }

// Generate:
match x {
    1 => "one",
    2 => "two",
    3 => "three",
    _ => "unknown",
}
```

### 2. Condition Factoring
```rust
// Before
if a && b && c { return 1 }
if a && b && d { return 1 }
if a && !b     { return 2 }

// After (factor common 'a')
if a {
    if b {
        if c || d { return 1 }
    } else {
        return 2;
    }
}
```

### 3. Karnaugh Map Visualization
Generate K-maps for visualization of the optimization:
```
Variables: a, b, c
Output: 1

      c=0 c=1
    +---+---+
a=0 | 0 | 0 | b=0
    +---+---+
a=0 | 1 | 1 | b=1  ← Can merge these
    +---+---+
a=1 | 1 | 1 | b=0  ← Can merge these
    +---+---+
a=1 | 1 | 0 | b=1
    +---+---+

Minimal: (a && b) || (!a && b)
        = b && (a || !a)
        = b
```

## Testing Strategy

### Unit Tests

**Parser Tests:**
- Parse simple Boolean expressions
- Parse nested expressions
- Handle operator precedence
- Error handling for malformed input

**Analyzer Tests:**
- Extract variables correctly
- Build truth tables
- Map outputs to minterms
- Handle don't-cares

**Optimizer Tests:**
- Apply QM correctly per output
- Handle overlapping conditions
- Detect dead code
- Calculate complexity metrics

**Code Generator Tests:**
- Generate syntactically correct code
- Preserve semantics
- Handle all target languages

### Integration Tests

**End-to-End Examples:**
```rust
#[test]
fn test_simple_or_to_simplified() {
    let input = "
    if a && b     { return 1 }
    if a && !b    { return 1 }
    else          { return 0 }
    ";

    let result = simplify(input, Language::Rust);

    assert_eq!(result.simplified_branches, 2);
    assert!(result.optimized_code.contains("if a {"));
    assert_eq!(result.analysis.complexity_reduction, 0.333); // 1/3 reduction
}
```

### Benchmark Tests
- Large condition trees (100+ branches)
- Deep nesting (10+ levels)
- Many variables (20+ variables)
- Performance: < 1s for typical cases

## Complexity Metrics

### Before/After Comparison
```rust
pub struct ComplexityMetrics {
    pub condition_count: usize,        // Number of Boolean operations
    pub branch_count: usize,           // Number of if/elif branches
    pub nesting_depth: usize,          // Max nesting level
    pub cyclomatic_complexity: usize,  // McCabe complexity
    pub line_count: usize,             // Total lines
}
```

### Improvement Calculation
```
complexity_reduction = (original - optimized) / original * 100%
```

## Edge Cases & Challenges

### 1. Non-Boolean Actions
**Challenge:** Branches with side effects or complex logic
**Solution:** Treat as atomic "black box" outputs, don't try to merge

### 2. Variable Scope
**Challenge:** Variables may have different scope/lifetime
**Solution:** Only simplify within same scope, warn about scope boundaries

### 3. Short-Circuit Evaluation
**Challenge:** Original code may rely on short-circuit (e.g., `a && b.unwrap()`)
**Solution:** Preserve evaluation order, add warnings about potential changes

### 4. Tautologies and Contradictions
**Challenge:** Conditions that are always true/false
```rust
if a || !a {  // Always true
    return 1;
}
```
**Solution:** Detect and simplify to unconditional code

### 5. Overlapping Conditions (Order Matters)
```rust
if a || b    { return 1 }  // Catches (a,b)=(0,1), (1,0), (1,1)
if a && b    { return 2 }  // Unreachable! Already covered by above
```
**Solution:** Analyze in order, mark unreachable branches

## Documentation

### User Guide Sections
1. **Quick Start**: Simple examples
2. **Input Formats**: All supported syntaxes
3. **Output Formats**: How to read results
4. **Limitations**: What it can't handle
5. **Best Practices**: When to use, when not to use
6. **Examples Gallery**: Real-world use cases

### API Documentation
```rust
/// Simplify a collection of if-then-else branches
///
/// # Arguments
/// * `branches` - List of condition/output pairs
/// * `variables` - Variable names (or auto-detect)
/// * `options` - Simplification options
///
/// # Returns
/// Simplified condition structure with analysis
///
/// # Example
/// ```
/// let branches = vec![
///     Branch::new("a && b", "return 1"),
///     Branch::new("a && !b", "return 1"),
/// ];
/// let result = simplify_conditions(branches, None, Options::default())?;
/// assert_eq!(result.simplified_branches, 1); // Merged to "if a"
/// ```
pub fn simplify_conditions(
    branches: Vec<Branch>,
    variables: Option<Vec<String>>,
    options: SimplificationOptions,
) -> Result<SimplificationResult>;
```

## Success Criteria

### MVP (Minimum Viable Product) - Phase 1
- ✅ Parse Boolean expressions with &&, ||, !
- ✅ Extract truth tables from conditions
- ✅ Apply QM minimization per output
- ✅ Generate simplified Rust code
- ✅ CLI interface working
- ✅ 20+ test cases passing
- ✅ Documentation with 5+ examples

### Future Enhancements (Post-MVP)
- Comparison operators (==, <, >, etc.)
- Multiple target languages (C, Python)
- Dead code detection
- Switch statement generation
- Karnaugh map visualization
- IDE plugin (VS Code extension)
- Web interface

## Timeline Estimate

### Phase 1: Boolean-Only Parser (1-2 weeks)
- Days 1-3: Design data structures and parser
- Days 4-6: Implement truth table extraction
- Days 7-9: Integrate with QM algorithm
- Days 10-12: Code generation and testing
- Days 13-14: CLI and documentation

### Phase 2: Multi-Output Optimization (3-5 days)
- Days 1-2: Output grouping logic
- Days 3-4: Testing and refinement
- Day 5: Documentation

### Phase 3: Dead Code Detection (3-5 days)
- Days 1-2: Coverage analysis
- Days 3-4: Reporting and warnings
- Day 5: Testing

### Phase 4: Comparison Operators (1 week)
- Days 1-3: Parser extensions
- Days 4-5: Range analysis
- Days 6-7: Testing

**Total: 3-4 weeks for full implementation**

## References & Related Work

### Academic Papers
- Quine-McCluskey Algorithm (1952, 1956)
- Karnaugh Maps (1953)
- Boolean Function Minimization
- Program Optimization via Boolean Analysis

### Tools
- Espresso Logic Minimizer
- ABC (Berkeley Logic Synthesis)
- CUDD (Decision Diagrams)
- Z3 SMT Solver (for advanced constraints)

### Existing Projects
- `boolean.py` - Python Boolean algebra
- `logic` crate - Rust Boolean logic
- `simplify-js` - JavaScript expression simplifier

## Open Questions

1. **Should we support floating-point comparisons?**
   - Likely too complex for Phase 1
   - Floating-point has precision issues
   - **Decision:** No, integers only for now

2. **How to handle function calls in conditions?**
   ```rust
   if is_valid() && check_permission() {
   ```
   - Treat as atomic Boolean variables?
   - **Decision:** Yes, treat as opaque variables

3. **Should we attempt to simplify nested if-then-else inside branches?**
   - Recursive simplification
   - **Decision:** Phase 1: No. Future: Maybe

4. **What about pattern matching optimization?**
   ```rust
   match value {
     Some(x) if x > 0 => { }
   ```
   - Complex semantic analysis needed
   - **Decision:** Out of scope for now

## Conclusion

This feature adds significant practical value to the QM agent by applying Boolean minimization to real-world code simplification. The phased approach allows for incremental development with clear milestones, starting with Boolean-only simplification (most practical) and expanding to more complex scenarios.

**Next Steps:**
1. Review this plan with stakeholders
2. Create detailed design document for Phase 1
3. Set up project structure (`src/simplify/`)
4. Begin implementation with parser and data structures
5. Iterate with tests and examples
