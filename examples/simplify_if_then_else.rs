// Example: Simplify if-then-else conditions using Quine-McCluskey

use qm_agent::simplify::{format_bool_expr, format_result, parse_bool_expr, simplify_branches, BoolExpr, BranchSet};

fn main() {
    println!("=== If-Then-Else Simplification Demo ===\n");

    // Example 1: Simple case - a && b || a && !b simplifies to just a
    example_1();

    println!("\n{}\n", "=".repeat(60));

    // Example 2: Multiple outputs
    example_2();

    println!("\n{}\n", "=".repeat(60));

    // Example 3: Three-variable condition
    example_3();
}

fn example_1() {
    println!("Example 1: Basic Simplification");
    println!("Original code:");
    println!("  if a && b    {{ return 1; }}");
    println!("  elif a && !b {{ return 1; }}");
    println!("  else         {{ return 0; }}");
    println!();

    let mut branches = BranchSet::new();
    branches.add_branch(
        BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b")),
        "1",
    );
    branches.add_branch(
        BoolExpr::and(BoolExpr::var("a"), BoolExpr::not(BoolExpr::var("b"))),
        "1",
    );
    branches.set_default("0");

    let result = simplify_branches(&branches).unwrap();

    println!("{}", format_result(&result));
    println!("Analysis: Both branches that return 1 require 'a' to be true,");
    println!("          so the condition simplifies to just checking 'a'.");
}

fn example_2() {
    println!("Example 2: Multiple Distinct Outputs");
    println!("Original code:");
    println!("  if a && b && c     {{ return 1; }}");
    println!("  elif a && b && !c  {{ return 1; }}");
    println!("  elif a && !b && c  {{ return 2; }}");
    println!("  elif !a && b && c  {{ return 2; }}");
    println!("  else               {{ return 0; }}");
    println!();

    let mut branches = BranchSet::new();

    // Branches returning 1
    branches.add_branch(
        parse_bool_expr("a && b && c").unwrap(),
        "1",
    );
    branches.add_branch(
        parse_bool_expr("a && b && !c").unwrap(),
        "1",
    );

    // Branches returning 2
    branches.add_branch(
        parse_bool_expr("a && !b && c").unwrap(),
        "2",
    );
    branches.add_branch(
        parse_bool_expr("!a && b && c").unwrap(),
        "2",
    );

    branches.set_default("0");

    let result = simplify_branches(&branches).unwrap();

    println!("{}", format_result(&result));
    println!("Analysis: QM groups conditions by their output value and");
    println!("          minimizes each group independently.");
}

fn example_3() {
    println!("Example 3: Using String Parser");
    println!("Original code:");
    println!("  if (a || b) && c    {{ return 1; }}");
    println!("  elif a && b && !c   {{ return 1; }}");
    println!("  else                {{ return 0; }}");
    println!();

    let mut branches = BranchSet::new();

    // Parse from string expressions
    branches.add_branch(
        parse_bool_expr("(a || b) && c").unwrap(),
        "1",
    );
    branches.add_branch(
        parse_bool_expr("a && b && !c").unwrap(),
        "1",
    );
    branches.set_default("0");

    let result = simplify_branches(&branches).unwrap();

    println!("{}", format_result(&result));

    // Show the minterms covered
    println!("Truth table analysis:");
    println!("  Variables: a, b, c (2³ = 8 possible combinations)");
    println!("  Output '1' minterms:");
    for (cond, output) in &result.simplified_conditions {
        if output == "1" {
            println!("    Condition: {}", format_bool_expr(cond));
        }
    }
}
