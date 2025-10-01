// Dead code detection and coverage analysis

use super::analyzer::{evaluate_with_ints, extract_variables};
use super::types::{
    BranchCoverage, BranchSet, DeadBranch, DeadCodeReason, SimplificationAnalysis, VariableType,
};
use std::collections::{HashMap, HashSet};

/// Analyze branch coverage and detect dead code
///
/// This function evaluates each branch in order and tracks:
/// - Which minterms each branch covers
/// - Which branches are unreachable (dead code)
/// - Overlapping conditions between branches
/// - Uncovered input combinations
pub fn analyze_branches(branch_set: &BranchSet) -> Result<SimplificationAnalysis, String> {
    // Collect all variables
    let mut all_vars = HashSet::new();
    for branch in &branch_set.branches {
        let vars = extract_variables(&branch.condition);
        all_vars.extend(vars);
    }

    let mut variables: Vec<String> = all_vars.into_iter().collect();
    variables.sort();

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

    // Get or infer variable types
    let mut var_types: HashMap<String, VariableType> = HashMap::new();
    for var in &variables {
        let var_type = branch_set
            .variable_types
            .get(var)
            .cloned()
            .unwrap_or(VariableType::Boolean);
        var_types.insert(var.clone(), var_type);
    }

    // Calculate total combinations
    let mut total_combinations = 1usize;
    for var in &variables {
        let var_type = &var_types[var];
        let range = (var_type.max_value() - var_type.min_value() + 1) as usize;
        total_combinations = total_combinations
            .checked_mul(range)
            .ok_or_else(|| "Too many variable combinations".to_string())?;
    }

    let total_rows = total_combinations as u32;
    let mut covered_minterms = HashSet::new();
    let mut branch_coverage: Vec<BranchCoverage> = Vec::new();
    let mut dead_branches = Vec::new();

    // Initialize assignments
    let mut assignments: Vec<i32> = variables
        .iter()
        .map(|v| var_types[v].min_value())
        .collect();

    // Analyze each branch in order
    for (branch_idx, branch) in branch_set.branches.iter().enumerate() {
        let mut minterms_for_this_branch = Vec::new();
        let mut overlaps_with = Vec::new();

        // Reset assignments for enumeration
        for i in 0..variables.len() {
            assignments[i] = var_types[&variables[i]].min_value();
        }

        // Evaluate which minterms this branch covers
        for minterm_idx in 0..total_rows {
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

            if evaluate_with_ints(&branch.condition, &bool_assignments, &int_assignments) {
                minterms_for_this_branch.push(minterm_idx);

                // Check if this minterm was already covered by an earlier branch
                if covered_minterms.contains(&minterm_idx) {
                    // Find which branch(es) already covered this
                    for (prev_idx, prev_coverage) in branch_coverage.iter().enumerate() {
                        if prev_coverage.minterms_covered.contains(&minterm_idx) {
                            if !overlaps_with.contains(&prev_idx) {
                                overlaps_with.push(prev_idx);
                            }
                        }
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

        // Check if this branch is dead code
        let new_coverage: HashSet<u32> = minterms_for_this_branch
            .iter()
            .filter(|&&m| !covered_minterms.contains(&m))
            .copied()
            .collect();

        if new_coverage.is_empty() && !minterms_for_this_branch.is_empty() {
            // This branch covers no new minterms - it's dead code
            let reason = if minterms_for_this_branch.is_empty() {
                DeadCodeReason::Contradiction
            } else {
                DeadCodeReason::FullyCovered
            };

            dead_branches.push(DeadBranch {
                branch_index: branch_idx,
                reason,
                covered_by: overlaps_with.clone(),
            });
        }

        // Add to covered set
        for &minterm in &minterms_for_this_branch {
            covered_minterms.insert(minterm);
        }

        // Record coverage for this branch
        branch_coverage.push(BranchCoverage {
            branch_index: branch_idx,
            minterms_covered: minterms_for_this_branch,
            coverage_count: new_coverage.len(),
            overlaps_with,
        });
    }

    // Check for uncovered minterms (only if there's no default)
    let mut uncovered_minterms = Vec::new();
    if branch_set.default_output.is_none() {
        for minterm in 0..total_rows {
            if !covered_minterms.contains(&minterm) {
                uncovered_minterms.push(minterm);
            }
        }
    }

    let total_coverage_percent = if total_rows > 0 {
        (covered_minterms.len() as f64 / total_rows as f64) * 100.0
    } else {
        0.0
    };

    Ok(SimplificationAnalysis {
        branch_coverage,
        dead_branches,
        uncovered_minterms,
        total_coverage_percent,
    })
}

/// Format a minterm as variable assignments
pub fn format_minterm(minterm: u32, variables: &[String]) -> String {
    let mut parts = Vec::new();
    for (i, var) in variables.iter().enumerate() {
        let bit_value = (minterm >> i) & 1;
        if bit_value == 1 {
            parts.push(var.clone());
        } else {
            parts.push(format!("!{}", var));
        }
    }
    parts.join(" && ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simplify::types::{BoolExpr, BranchSet};

    #[test]
    fn test_detect_fully_covered() {
        // if a || b { return "1" }  // Covers [1,2,3]
        // elif a && b { return "2" }  // Covers [3] - already covered! DEAD CODE
        let mut branches = BranchSet::new();
        branches.add_branch(
            BoolExpr::or(BoolExpr::var("a"), BoolExpr::var("b")),
            "1",
        );
        branches.add_branch(
            BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b")),
            "2",
        );

        let analysis = analyze_branches(&branches).unwrap();

        // Branch 1 should cover [1, 2, 3] (a||b is true when a=1 or b=1 or both)
        assert_eq!(analysis.branch_coverage[0].minterms_covered.len(), 3);

        // Branch 2 should be detected as dead code
        assert_eq!(analysis.dead_branches.len(), 1);
        assert_eq!(analysis.dead_branches[0].branch_index, 1);
        assert_eq!(analysis.dead_branches[0].reason, DeadCodeReason::FullyCovered);
        assert_eq!(analysis.dead_branches[0].covered_by, vec![0]);
    }

    #[test]
    fn test_detect_overlapping() {
        // if a && b { return "1" }  // Covers [3]
        // elif a { return "2" }     // Covers [2, 3] - overlaps with branch 0!
        let mut branches = BranchSet::new();
        branches.add_branch(
            BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b")),
            "1",
        );
        branches.add_branch(BoolExpr::var("a"), "2");

        let analysis = analyze_branches(&branches).unwrap();

        // Branch 1 should overlap with branch 0
        assert!(!analysis.branch_coverage[1].overlaps_with.is_empty());
        assert!(analysis.branch_coverage[1].overlaps_with.contains(&0));

        // But branch 1 is not dead because it covers new minterm [2]
        assert_eq!(analysis.dead_branches.len(), 0);
    }

    #[test]
    fn test_detect_uncovered() {
        // if a && b { return "1" }  // Only covers [3]
        // No default - minterms [0, 1, 2] are uncovered
        let mut branches = BranchSet::new();
        branches.add_branch(
            BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b")),
            "1",
        );
        // No default

        let analysis = analyze_branches(&branches).unwrap();

        assert_eq!(analysis.uncovered_minterms.len(), 3);
        assert!(analysis.uncovered_minterms.contains(&0)); // !a && !b
        assert!(analysis.uncovered_minterms.contains(&1)); // a && !b
        assert!(analysis.uncovered_minterms.contains(&2)); // !a && b
    }

    #[test]
    fn test_format_minterm() {
        let vars = vec!["a".to_string(), "b".to_string()];

        // minterm 0: a=0, b=0
        assert_eq!(format_minterm(0, &vars), "!a && !b");

        // minterm 1: a=1, b=0
        assert_eq!(format_minterm(1, &vars), "a && !b");

        // minterm 3: a=1, b=1
        assert_eq!(format_minterm(3, &vars), "a && b");
    }
}
