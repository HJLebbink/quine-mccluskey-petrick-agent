// Comparison operator support for if-then-else simplification

use super::analyzer::{evaluate_with_ints, extract_variables};
use super::types::{BranchSet, TruthTable, VariableType};
use std::collections::{HashMap, HashSet};

/// Build truth table with support for integer variables and comparisons
///
/// This version handles:
/// - Boolean variables (0-1 domain)
/// - Integer variables with bounded domains (e.g., 0-7)
/// - Comparison operators (==, <, >, etc.)
///
/// Algorithm:
/// 1. Extract all variables and their types from branch_set
/// 2. Enumerate all possible value combinations
/// 3. For each combination, evaluate all branches in order
/// 4. Map to output groups
pub fn build_truth_table_with_comparisons(
    branch_set: &BranchSet,
) -> Result<TruthTable, String> {
    // Collect all variables and infer types if not declared
    let mut all_vars = HashSet::new();
    for branch in &branch_set.branches {
        let vars = extract_variables(&branch.condition);
        all_vars.extend(vars);
    }

    let mut variables: Vec<String> = all_vars.into_iter().collect();
    variables.sort();

    if variables.is_empty() {
        return Err("No variables found in conditions".to_string());
    }

    // Get or infer variable types
    let mut var_types: HashMap<String, VariableType> = HashMap::new();
    for var in &variables {
        let var_type = branch_set
            .variable_types
            .get(var)
            .cloned()
            .unwrap_or(VariableType::Boolean); // Default to Boolean
        var_types.insert(var.clone(), var_type);
    }

    // Calculate total number of combinations
    let mut total_combinations = 1usize;
    for var in &variables {
        let var_type = &var_types[var];
        let range = (var_type.max_value() - var_type.min_value() + 1) as usize;
        total_combinations = total_combinations
            .checked_mul(range)
            .ok_or_else(|| "Too many variable combinations".to_string())?;
    }

    if total_combinations > 65536 {
        return Err(format!(
            "Too many combinations ({}). Maximum: 65536",
            total_combinations
        ));
    }

    let mut output_groups: HashMap<String, Vec<u32>> = HashMap::new();
    let mut dont_cares = Vec::new();

    // Enumerate all combinations
    let mut assignments: Vec<i32> = variables
        .iter()
        .map(|v| var_types[v].min_value())
        .collect();

    for minterm_idx in 0..total_combinations as u32 {
        // Build assignment maps
        let mut bool_assignments = HashMap::new();
        let mut int_assignments = HashMap::new();

        for (i, var) in variables.iter().enumerate() {
            let value = assignments[i];
            match &var_types[var] {
                VariableType::Boolean => {
                    bool_assignments.insert(var.clone(), value != 0);
                }
                VariableType::Integer { .. } => {
                    int_assignments.insert(var.clone(), value);
                }
            }
        }

        // Find first matching branch
        let mut output = None;
        for branch in &branch_set.branches {
            if evaluate_with_ints(&branch.condition, &bool_assignments, &int_assignments) {
                output = Some(branch.output.clone());
                break;
            }
        }

        // Record output
        match output {
            Some(out) => {
                output_groups
                    .entry(out)
                    .or_default()
                    .push(minterm_idx);
            }
            None => {
                if let Some(ref default) = branch_set.default_output {
                    output_groups
                        .entry(default.clone())
                        .or_default()
                        .push(minterm_idx);
                } else {
                    dont_cares.push(minterm_idx);
                }
            }
        }

        // Increment to next combination (like odometer)
        let mut carry = true;
        for i in 0..variables.len() {
            if carry {
                assignments[i] += 1;
                let var_type = &var_types[&variables[i]];
                if assignments[i] > var_type.max_value() {
                    assignments[i] = var_type.min_value();
                } else {
                    carry = false;
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
    use crate::simplify::types::{BoolExpr, BranchSet};

    #[test]
    fn test_comparison_equals() {
        // if x == 2 { return "A" } else { return "B" }
        // where x is 0-3 (2-bit)
        let mut branches = BranchSet::new();
        branches.declare_int("x", 0, 3);
        branches.add_branch(BoolExpr::equals("x", 2), "A");
        branches.set_default("B");

        let table = build_truth_table_with_comparisons(&branches).unwrap();

        // Should have 4 combinations (x = 0, 1, 2, 3)
        let a_minterms = table.output_groups.get("A").unwrap();
        assert_eq!(a_minterms.len(), 1); // Only x=2

        let b_minterms = table.output_groups.get("B").unwrap();
        assert_eq!(b_minterms.len(), 3); // x=0,1,3
    }

    #[test]
    fn test_comparison_less_than() {
        // if x < 2 { return "small" } else { return "big" }
        // where x is 0-3
        let mut branches = BranchSet::new();
        branches.declare_int("x", 0, 3);
        branches.add_branch(BoolExpr::less_than("x", 2), "small");
        branches.set_default("big");

        let table = build_truth_table_with_comparisons(&branches).unwrap();

        let small = table.output_groups.get("small").unwrap();
        assert_eq!(small.len(), 2); // x=0,1

        let big = table.output_groups.get("big").unwrap();
        assert_eq!(big.len(), 2); // x=2,3
    }

    #[test]
    fn test_mixed_bool_and_int() {
        // if a && x > 1 { return "1" } else { return "0" }
        // where a is boolean, x is 0-3
        let mut branches = BranchSet::new();
        branches.declare_bool("a");
        branches.declare_int("x", 0, 3);

        branches.add_branch(
            BoolExpr::and(
                BoolExpr::var("a"),
                BoolExpr::greater_than("x", 1),
            ),
            "1",
        );
        branches.set_default("0");

        let table = build_truth_table_with_comparisons(&branches).unwrap();

        // Total combinations: 2 (for a) * 4 (for x) = 8
        // "1" when a=true AND x>1: (a=1, x=2), (a=1, x=3) = 2 combinations
        let ones = table.output_groups.get("1").unwrap();
        assert_eq!(ones.len(), 2);

        let zeros = table.output_groups.get("0").unwrap();
        assert_eq!(zeros.len(), 6);
    }
}
