// Example: Comparison Operators in if-then-else simplification

use qm_agent::simplify::{format_result, simplify_branches, BoolExpr, BranchSet};

fn main() {
    println!("=== Comparison Operators Demo ===\n");

    // Example 1: Simple integer comparison
    example_1_simple_comparison();

    println!("\n{}\n", "=".repeat(60));

    // Example 2: Range checks
    example_2_range_checks();

    println!("\n{}\n", "=".repeat(60));

    // Example 3: Mixed boolean and integer
    example_3_mixed();

    println!("\n{}\n", "=".repeat(60));

    // Example 4: Dead code with comparisons
    example_4_dead_code();
}

fn example_1_simple_comparison() {
    println!("Example 1: Simple Integer Comparison");
    println!("Original code:");
    println!("  if x == 0  {{ return \"zero\"; }}");
    println!("  elif x == 1  {{ return \"one\"; }}");
    println!("  elif x == 2  {{ return \"two\"; }}");
    println!("  else         {{ return \"other\"; }}");
    println!();
    println!("Variable x is a 2-bit integer (0-3)");
    println!();

    let mut branches = BranchSet::new();
    branches.declare_int("x", 0, 3); // 2-bit integer

    branches.add_branch(BoolExpr::equals("x", 0), "zero");
    branches.add_branch(BoolExpr::equals("x", 1), "one");
    branches.add_branch(BoolExpr::equals("x", 2), "two");
    branches.set_default("other");

    let result = simplify_branches(&branches).unwrap();
    println!("{}", format_result(&result));

    println!("Analysis: Each equality check maps to exactly one minterm.");
    println!("          x=0 → minterm 0, x=1 → minterm 1, etc.");
}

fn example_2_range_checks() {
    println!("Example 2: Range Checks Optimization");
    println!("Original code:");
    println!("  if x < 2   {{ return \"small\"; }}");
    println!("  elif x < 4 {{ return \"medium\"; }}");
    println!("  else       {{ return \"large\"; }}");
    println!();
    println!("Variable x is a 3-bit integer (0-7)");
    println!();

    let mut branches = BranchSet::new();
    branches.declare_int("x", 0, 7); // 3-bit integer

    branches.add_branch(BoolExpr::less_than("x", 2), "small");
    branches.add_branch(BoolExpr::less_than("x", 4), "medium");
    branches.set_default("large");

    let result = simplify_branches(&branches).unwrap();
    println!("{}", format_result(&result));

    println!("Analysis: x < 2 covers [0, 1]");
    println!("          x < 4 covers [0, 1, 2, 3] but [0, 1] already taken");
    println!("          So second branch effectively covers [2, 3]");
    println!("          Else covers [4, 5, 6, 7]");
}

fn example_3_mixed() {
    println!("Example 3: Mixed Boolean and Integer");
    println!("Original code:");
    println!("  if enabled && count > 0  {{ return \"active\"; }}");
    println!("  elif enabled             {{ return \"ready\"; }}");
    println!("  else                     {{ return \"disabled\"; }}");
    println!();
    println!("enabled is Boolean, count is 2-bit integer (0-3)");
    println!();

    let mut branches = BranchSet::new();
    branches.declare_bool("enabled");
    branches.declare_int("count", 0, 3);

    branches.add_branch(
        BoolExpr::and(
            BoolExpr::var("enabled"),
            BoolExpr::greater_than("count", 0),
        ),
        "active",
    );
    branches.add_branch(BoolExpr::var("enabled"), "ready");
    branches.set_default("disabled");

    let result = simplify_branches(&branches).unwrap();
    println!("{}", format_result(&result));

    println!("Analysis: 2 boolean values × 4 integer values = 8 combinations");
    println!("          'active' when enabled=true AND count>0 → 3 combinations");
    println!("          'ready' when enabled=true AND count=0 → 1 combination");
    println!("          'disabled' when enabled=false → 4 combinations");
}

fn example_4_dead_code() {
    println!("Example 4: Dead Code Detection with Comparisons");
    println!("Original code:");
    println!("  if x < 3   {{ return \"small\"; }}");
    println!("  elif x == 2  {{ return \"two\"; }}    // DEAD! x=2 already covered");
    println!("  else       {{ return \"large\"; }}");
    println!();
    println!("Variable x is a 3-bit integer (0-7)");
    println!();

    let mut branches = BranchSet::new();
    branches.declare_int("x", 0, 7);

    branches.add_branch(BoolExpr::less_than("x", 3), "small"); // Covers [0,1,2]
    branches.add_branch(BoolExpr::equals("x", 2), "two"); // DEAD - 2 already covered!
    branches.set_default("large");

    let result = simplify_branches(&branches).unwrap();
    println!("{}", format_result(&result));

    println!("Analysis: x < 3 covers [0, 1, 2]");
    println!("          x == 2 only covers [2], which is already handled");
    println!("          Therefore branch 2 is unreachable dead code!");
}
