//! If-Then-Else Simplification Module
//!
//! This module provides functionality to simplify complex if-then-else chains
//! using the Quine-McCluskey algorithm.
//!
//! # Mini-MVP Features
//! - Parse simple Boolean expressions (&&, ||, !)
//! - Build truth tables from branch conditions
//! - Apply QM minimization per output value
//! - Output simplified conditions
//!
//! # Example
//! ```
//! use qm_agent::simplify::{BoolExpr, BranchSet, simplify_branches, format_result};
//!
//! // Create branches: if a && b { 1 } elif a && !b { 1 } else { 0 }
//! let mut branches = BranchSet::new();
//! branches.add_branch(
//!     BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b")),
//!     "1"
//! );
//! branches.add_branch(
//!     BoolExpr::and(BoolExpr::var("a"), BoolExpr::negate(BoolExpr::var("b"))),
//!     "1"
//! );
//! branches.set_default("0");
//!
//! // Simplify
//! let result = simplify_branches(&branches).unwrap();
//! println!("{}", format_result(&result));
//! // Output: Simplified to 2 branches (from 2 original)
//! //         if a { return 1; }
//! //         else { return 0; }
//! ```

pub mod analyzer;
pub mod comparisons;
pub mod dead_code;
pub mod optimizer;
pub mod parser;
pub mod types;

// Re-export main types and functions
pub use comparisons::build_truth_table_with_comparisons;
pub use dead_code::{analyze_branches, format_minterm};
pub use optimizer::{format_bool_expr, simplify_branches};
pub use parser::parse_bool_expr;
pub use types::{
    BoolExpr, Branch, BranchCoverage, BranchSet, DeadBranch, DeadCodeReason,
    SimplificationAnalysis, SimplificationResult, VariableType,
};

/// Format simplification result as human-readable text
pub fn format_result(result: &SimplificationResult) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "Simplified to {} branches (from {} original)\n",
        result.simplified_branch_count, result.original_branch_count
    ));
    output.push_str(&format!(
        "Complexity reduction: {:.1}%\n",
        result.complexity_reduction()
    ));
    output.push_str(&format!(
        "Coverage: {:.1}%\n\n",
        result.analysis.total_coverage_percent
    ));

    output.push_str("Variables: ");
    output.push_str(&result.variables.join(", "));
    output.push_str("\n\n");

    // Show dead code warnings
    if result.analysis.has_dead_code() {
        output.push_str("⚠️  DEAD CODE DETECTED:\n");
        for dead in &result.analysis.dead_branches {
            output.push_str(&format!(
                "  Branch {} is unreachable (reason: {:?})\n",
                dead.branch_index, dead.reason
            ));
            if !dead.covered_by.is_empty() {
                output.push_str(&format!(
                    "    Already covered by branches: {:?}\n",
                    dead.covered_by
                ));
            }
        }
        output.push('\n');
    }

    // Show coverage gaps
    if result.analysis.has_coverage_gaps() {
        output.push_str("⚠️  COVERAGE GAPS (missing test cases):\n");
        for &minterm in result.analysis.uncovered_minterms.iter().take(5) {
            output.push_str(&format!(
                "  Uncovered: {}\n",
                format_minterm(minterm, &result.variables)
            ));
        }
        if result.analysis.uncovered_minterms.len() > 5 {
            output.push_str(&format!(
                "  ... and {} more\n",
                result.analysis.uncovered_minterms.len() - 5
            ));
        }
        output.push('\n');
    }

    output.push_str("Simplified conditions:\n");
    for (i, (condition, out)) in result.simplified_conditions.iter().enumerate() {
        if i == 0 {
            output.push_str("if ");
        } else if i == result.simplified_conditions.len() - 1
            && result.simplified_conditions.len() > 1
        {
            output.push_str("else ");
        } else {
            output.push_str("elif ");
        }

        output.push_str(&format_bool_expr(condition));
        output.push_str(&format!(" {{ return {}; }}\n", out));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_end_to_end_simplification() {
        // Test case: a && b || a && !b should simplify to just a
        let mut branches = BranchSet::new();
        branches.add_branch(
            BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b")),
            "1",
        );
        branches.add_branch(
            BoolExpr::and(BoolExpr::var("a"), BoolExpr::negate(BoolExpr::var("b"))),
            "1",
        );
        branches.set_default("0");

        let result = simplify_branches(&branches).unwrap();

        assert_eq!(result.original_branch_count, 2);
        assert_eq!(result.simplified_branch_count, 2); // "1" and "0" outputs

        let formatted = format_result(&result);
        println!("{}", formatted);

        assert!(formatted.contains("a"));
    }

    #[test]
    fn test_parse_and_simplify() {
        // Parse from strings
        let cond1 = parse_bool_expr("a && b").unwrap();
        let cond2 = parse_bool_expr("a && !b").unwrap();

        let mut branches = BranchSet::new();
        branches.add_branch(cond1, "1");
        branches.add_branch(cond2, "1");
        branches.set_default("0");

        let result = simplify_branches(&branches).unwrap();
        assert_eq!(result.simplified_branch_count, 2);
    }
}
