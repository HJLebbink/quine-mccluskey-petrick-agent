// Example: Dead Code Detection in if-then-else chains

use qm_agent::simplify::{format_result, parse_bool_expr, simplify_branches, BranchSet};

fn main() {
    println!("=== Dead Code Detection Demo ===\n");

    // Example 1: Fully covered dead code
    example_1_fully_covered();

    println!("\n{}\n", "=".repeat(60));

    // Example 2: Overlapping but not dead
    example_2_overlapping();

    println!("\n{}\n", "=".repeat(60));

    // Example 3: Missing coverage (gaps)
    example_3_coverage_gaps();

    println!("\n{}\n", "=".repeat(60));

    // Example 4: Multiple dead branches
    example_4_multiple_dead();
}

fn example_1_fully_covered() {
    println!("Example 1: Fully Covered Dead Code");
    println!("Original code:");
    println!("  if a || b    {{ return 1; }}  // Covers when a=1 OR b=1");
    println!("  elif a && b  {{ return 2; }}  // DEAD! Already covered by first branch");
    println!("  else         {{ return 0; }}");
    println!();

    let mut branches = BranchSet::new();

    // First branch: a || b (covers minterms where a=1 or b=1)
    branches.add_branch(parse_bool_expr("a || b").unwrap(), "1");

    // Second branch: a && b (DEAD CODE - already covered by a || b)
    branches.add_branch(parse_bool_expr("a && b").unwrap(), "2");

    branches.set_default("0");

    let result = simplify_branches(&branches).unwrap();
    println!("{}", format_result(&result));

    println!("Explanation: When both a and b are true, the first branch");
    println!("             (a || b) catches it, so the second branch never executes.");
}

fn example_2_overlapping() {
    println!("Example 2: Overlapping But Not Dead");
    println!("Original code:");
    println!("  if a && b  {{ return 1; }}  // Covers a=1, b=1");
    println!("  elif a     {{ return 2; }}  // Overlaps but also covers a=1, b=0");
    println!("  else       {{ return 0; }}");
    println!();

    let mut branches = BranchSet::new();

    branches.add_branch(parse_bool_expr("a && b").unwrap(), "1");
    branches.add_branch(parse_bool_expr("a").unwrap(), "2"); // Overlaps but not dead

    branches.set_default("0");

    let result = simplify_branches(&branches).unwrap();
    println!("{}", format_result(&result));

    println!("Explanation: Branch 2 overlaps with branch 1, but it also");
    println!("             covers the case where a=true and b=false,");
    println!("             so it's NOT dead code.");
}

fn example_3_coverage_gaps() {
    println!("Example 3: Coverage Gaps (Missing Test Cases)");
    println!("Original code:");
    println!("  if a && b && c  {{ return 1; }}  // Only covers a=1, b=1, c=1");
    println!("  // NO else clause - what about other cases?");
    println!();

    let mut branches = BranchSet::new();

    branches.add_branch(parse_bool_expr("a && b && c").unwrap(), "1");
    // No default!

    let result = simplify_branches(&branches).unwrap();
    println!("{}", format_result(&result));

    println!("Explanation: Only 1 out of 8 possible input combinations");
    println!("             is handled. The other 7 are coverage gaps -");
    println!("             missing test cases that could cause bugs!");
}

fn example_4_multiple_dead() {
    println!("Example 4: Multiple Dead Branches");
    println!("Original code:");
    println!("  if a          {{ return 1; }}  // Covers a=1");
    println!("  elif a && b   {{ return 2; }}  // DEAD! Subset of first");
    println!("  elif a && !b  {{ return 3; }}  // DEAD! Subset of first");
    println!("  elif b        {{ return 4; }}  // OK - covers a=0, b=1");
    println!("  else          {{ return 0; }}");
    println!();

    let mut branches = BranchSet::new();

    branches.add_branch(parse_bool_expr("a").unwrap(), "1");
    branches.add_branch(parse_bool_expr("a && b").unwrap(), "2"); // Dead
    branches.add_branch(parse_bool_expr("a && !b").unwrap(), "3"); // Dead
    branches.add_branch(parse_bool_expr("b").unwrap(), "4"); // OK

    branches.set_default("0");

    let result = simplify_branches(&branches).unwrap();
    println!("{}", format_result(&result));

    println!("Explanation: Branches 2 and 3 are both dead because");
    println!("             branch 1 already handles ALL cases where a=true.");
    println!("             Branch 4 is fine because it handles a=0, b=1.");
}
