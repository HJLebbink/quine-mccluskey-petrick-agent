// Analyzer: Convert branches to truth tables

use super::types::{BoolExpr, BranchSet, TruthTable};
use std::collections::{HashMap, HashSet};

/// Extract all variables from a Boolean expression
pub fn extract_variables(expr: &BoolExpr) -> HashSet<String> {
    let mut vars = HashSet::new();
    extract_variables_recursive(expr, &mut vars);
    vars
}

fn extract_variables_recursive(expr: &BoolExpr, vars: &mut HashSet<String>) {
    match expr {
        BoolExpr::Var(name) => {
            vars.insert(name.clone());
        }
        BoolExpr::Not(inner) => {
            extract_variables_recursive(inner, vars);
        }
        BoolExpr::And(left, right) | BoolExpr::Or(left, right) => {
            extract_variables_recursive(left, vars);
            extract_variables_recursive(right, vars);
        }
        // Comparison operators
        BoolExpr::Equals(var, _)
        | BoolExpr::NotEquals(var, _)
        | BoolExpr::LessThan(var, _)
        | BoolExpr::LessOrEqual(var, _)
        | BoolExpr::GreaterThan(var, _)
        | BoolExpr::GreaterOrEqual(var, _) => {
            vars.insert(var.clone());
        }
    }
}

/// Evaluate a Boolean expression given variable assignments
/// For boolean variables: assignments map to bool
/// For integer variables: assignments map to i32 (stored as bool for compatibility)
pub fn evaluate(expr: &BoolExpr, assignments: &HashMap<String, bool>) -> bool {
    evaluate_with_ints(expr, assignments, &HashMap::new())
}

/// Evaluate with explicit integer assignments
pub fn evaluate_with_ints(
    expr: &BoolExpr,
    bool_assignments: &HashMap<String, bool>,
    int_assignments: &HashMap<String, i32>,
) -> bool {
    match expr {
        BoolExpr::Var(name) => *bool_assignments.get(name).unwrap_or(&false),
        BoolExpr::Not(inner) => !evaluate_with_ints(inner, bool_assignments, int_assignments),
        BoolExpr::And(left, right) => {
            evaluate_with_ints(left, bool_assignments, int_assignments)
                && evaluate_with_ints(right, bool_assignments, int_assignments)
        }
        BoolExpr::Or(left, right) => {
            evaluate_with_ints(left, bool_assignments, int_assignments)
                || evaluate_with_ints(right, bool_assignments, int_assignments)
        }
        // Comparison operators
        BoolExpr::Equals(var, value) => {
            int_assignments.get(var) == Some(value)
        }
        BoolExpr::NotEquals(var, value) => {
            int_assignments.get(var) != Some(value)
        }
        BoolExpr::LessThan(var, value) => {
            int_assignments.get(var).is_some_and(|v| v < value)
        }
        BoolExpr::LessOrEqual(var, value) => {
            int_assignments.get(var).is_some_and(|v| v <= value)
        }
        BoolExpr::GreaterThan(var, value) => {
            int_assignments.get(var).is_some_and(|v| v > value)
        }
        BoolExpr::GreaterOrEqual(var, value) => {
            int_assignments.get(var).is_some_and(|v| v >= value)
        }
    }
}

/// Convert branches to a truth table
///
/// Algorithm:
/// 1. Extract all unique variables from all branches
/// 2. For each possible input combination (2^n rows):
///    - Evaluate each branch condition in order
///    - First branch that evaluates to true determines the output
///    - If no branch matches, use default output (or mark as don't care)
/// 3. Group minterms by their output value
pub fn build_truth_table(branch_set: &BranchSet) -> Result<TruthTable, String> {
    // Collect all variables
    let mut all_vars = HashSet::new();
    for branch in &branch_set.branches {
        let vars = extract_variables(&branch.condition);
        all_vars.extend(vars);
    }

    let mut variables: Vec<String> = all_vars.into_iter().collect();
    variables.sort(); // Deterministic ordering

    let var_count = variables.len();
    if var_count == 0 {
        return Err("No variables found in conditions".to_string());
    }
    if var_count > 16 {
        return Err(format!(
            "Too many variables ({}). Maximum supported: 16",
            var_count
        ));
    }

    let total_rows = 1u32 << var_count;
    let mut output_groups: HashMap<String, Vec<u32>> = HashMap::new();
    let mut dont_cares = Vec::new();

    // Evaluate each possible input combination
    for minterm in 0..total_rows {
        // Build variable assignment map for this minterm
        let mut assignments = HashMap::new();
        for (i, var) in variables.iter().enumerate() {
            let bit_value = (minterm >> i) & 1;
            assignments.insert(var.clone(), bit_value == 1);
        }

        // Find first matching branch
        let mut output = None;
        for branch in &branch_set.branches {
            if evaluate(&branch.condition, &assignments) {
                output = Some(branch.output.clone());
                break;
            }
        }

        // If no branch matched, use default or mark as don't care
        match output {
            Some(out) => {
                output_groups.entry(out).or_default().push(minterm);
            }
            None => {
                if let Some(ref default) = branch_set.default_output {
                    output_groups
                        .entry(default.clone())
                        .or_default()
                        .push(minterm);
                } else {
                    dont_cares.push(minterm);
                }
            }
        }
    }

    Ok(TruthTable {
        variables,
        output_groups,
        dont_cares,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simplify::types::BoolExpr;

    #[test]
    fn test_extract_variables() {
        let expr = BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b"));
        let vars = extract_variables(&expr);
        assert_eq!(vars.len(), 2);
        assert!(vars.contains("a"));
        assert!(vars.contains("b"));
    }

    #[test]
    fn test_evaluate_simple() {
        let expr = BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b"));
        let mut assignments = HashMap::new();
        assignments.insert("a".to_string(), true);
        assignments.insert("b".to_string(), true);
        assert!(evaluate(&expr, &assignments));

        assignments.insert("b".to_string(), false);
        assert!(!evaluate(&expr, &assignments));
    }

    #[test]
    fn test_build_truth_table_simple() {
        // if a && b { return "1" } else { return "0" }
        let mut branch_set = BranchSet::new();
        branch_set.add_branch(
            BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b")),
            "1",
        );
        branch_set.set_default("0");

        let table = build_truth_table(&branch_set).unwrap();

        assert_eq!(table.variables.len(), 2);
        assert!(table.variables.contains(&"a".to_string()));
        assert!(table.variables.contains(&"b".to_string()));

        // With a, b ordering: minterm 3 = (a=1, b=1) should output "1"
        let ones = table.output_groups.get("1").unwrap();
        assert_eq!(ones.len(), 1);
        // The exact minterm depends on variable ordering, but there should be exactly 1
        assert!(ones.contains(&3) || ones.contains(&3)); // a=1, b=1

        let zeros = table.output_groups.get("0").unwrap();
        assert_eq!(zeros.len(), 3); // All other combinations
    }

    #[test]
    fn test_build_truth_table_multiple_branches() {
        // if a && b { return "1" }
        // elif a && !b { return "1" }
        // else { return "0" }
        let mut branch_set = BranchSet::new();
        branch_set.add_branch(
            BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b")),
            "1",
        );
        branch_set.add_branch(
            BoolExpr::and(BoolExpr::var("a"), BoolExpr::negate(BoolExpr::var("b"))),
            "1",
        );
        branch_set.set_default("0");

        let table = build_truth_table(&branch_set).unwrap();

        let ones = table.output_groups.get("1").unwrap();
        // Should have 2 minterms: a && b, and a && !b
        // This simplifies to just "a"
        assert_eq!(ones.len(), 2);
    }
}
