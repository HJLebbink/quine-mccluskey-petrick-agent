// Optimizer: Apply QM minimization and generate simplified conditions

use super::types::{BoolExpr, BranchSet, SimplificationResult, TruthTable};
use crate::qm::QMSolver;

/// Simplify a set of branches using Quine-McCluskey minimization
pub fn simplify_branches(branch_set: &BranchSet) -> Result<SimplificationResult, String> {
    // Analyze for dead code first
    let analysis = super::dead_code::analyze_branches(branch_set)?;

    // Check if we have integer variables
    let has_int_vars = branch_set
        .variable_types
        .values()
        .any(|t| matches!(t, super::types::VariableType::Integer { .. }));

    // For integer variables, skip QM minimization and keep original conditions
    // (QM doesn't work well with integer comparisons as they're already minimal)
    if has_int_vars {
        return simplify_with_integer_vars(branch_set, analysis);
    }

    // Build truth table from branches (boolean-only)
    let table = super::analyzer::build_truth_table(branch_set)?;

    let original_count = branch_set.branches.len();
    let mut simplified_conditions = Vec::new();

    // For each unique output value, run QM minimization
    for (output, minterms) in &table.output_groups {
        let minimized_expr = minimize_for_output(&table, minterms, &table.dont_cares)?;
        simplified_conditions.push((minimized_expr, output.clone()));
    }

    // Sort by output for deterministic results
    simplified_conditions.sort_by(|a, b| a.1.cmp(&b.1));

    let simplified_count = simplified_conditions.len();

    Ok(SimplificationResult {
        variables: table.variables.clone(),
        simplified_conditions,
        original_branch_count: original_count,
        simplified_branch_count: simplified_count,
        analysis,
    })
}

/// Simplify branches with integer variables (skip QM, keep original conditions)
fn simplify_with_integer_vars(
    branch_set: &BranchSet,
    analysis: super::types::SimplificationAnalysis,
) -> Result<SimplificationResult, String> {
    use std::collections::HashSet;

    // Extract all variables
    let mut all_vars = HashSet::new();
    for branch in &branch_set.branches {
        let vars = super::analyzer::extract_variables(&branch.condition);
        all_vars.extend(vars);
    }
    let mut variables: Vec<String> = all_vars.into_iter().collect();
    variables.sort();

    // Identify dead branches
    let dead_indices: HashSet<usize> = analysis
        .dead_branches
        .iter()
        .map(|db| db.branch_index)
        .collect();

    // Keep non-dead branches in original order
    let mut simplified_conditions = Vec::new();
    for (idx, branch) in branch_set.branches.iter().enumerate() {
        if !dead_indices.contains(&idx) {
            simplified_conditions.push((branch.condition.clone(), branch.output.clone()));
        }
    }

    // Add default if present
    if let Some(ref default) = branch_set.default_output {
        // Use tautology to represent else clause
        let else_condition = BoolExpr::or(
            BoolExpr::var(&variables[0]),
            BoolExpr::not(BoolExpr::var(&variables[0])),
        );
        simplified_conditions.push((else_condition, default.clone()));
    }

    let simplified_count = simplified_conditions.len();

    Ok(SimplificationResult {
        variables,
        simplified_conditions,
        original_branch_count: branch_set.branches.len(),
        simplified_branch_count: simplified_count,
        analysis,
    })
}

/// Apply QM minimization for a single output value
fn minimize_for_output(
    table: &TruthTable,
    minterms: &[u32],
    dont_cares: &[u32],
) -> Result<BoolExpr, String> {
    let var_count = table.variable_count();

    // Use QMSolver from existing module with our variable names
    let mut solver = QMSolver::with_variable_names(var_count, table.variables.clone());
    solver.set_minterms(minterms);
    solver.set_dont_cares(dont_cares);

    let result = solver.solve();

    // Convert minimal cover to BoolExpr
    if result.minimized_expression == "0" {
        return Err("Contradiction: no valid conditions".to_string());
    }

    if result.minimized_expression == "1" {
        // Tautology - always true
        // Return first variable as placeholder (we'll handle this better later)
        return Ok(BoolExpr::or(
            BoolExpr::var(&table.variables[0]),
            BoolExpr::not(BoolExpr::var(&table.variables[0])),
        ));
    }

    // Parse the minimized expression back to BoolExpr
    // The QM result is like: "AB + A'C + BC"
    parse_qm_result(&result.minimized_expression, &table.variables)
}

/// Parse QM output (SOP form) back to BoolExpr
/// Input format: "AB + A'C + BC" where A,B,C are variable names
fn parse_qm_result(expr: &str, variables: &[String]) -> Result<BoolExpr, String> {
    if expr.is_empty() || expr == "0" {
        return Err("Empty expression".to_string());
    }

    // Split by " + " for OR terms
    let or_terms: Vec<&str> = expr.split(" + ").collect();

    if or_terms.is_empty() {
        return Err("No terms in expression".to_string());
    }

    let mut result: Option<BoolExpr> = None;

    for term in or_terms {
        let term = term.trim();
        if term.is_empty() {
            continue;
        }

        // Parse this AND term
        let and_expr = parse_and_term(term, variables)?;

        result = match result {
            None => Some(and_expr),
            Some(existing) => Some(BoolExpr::or(existing, and_expr)),
        };
    }

    result.ok_or_else(|| "Failed to parse expression".to_string())
}

/// Parse a single AND term like "AB'C" or "A'B"
/// Note: This only handles Boolean variables from QM output, not comparisons
fn parse_and_term(term: &str, variables: &[String]) -> Result<BoolExpr, String> {
    let mut result: Option<BoolExpr> = None;
    let mut i = 0;
    let chars: Vec<char> = term.chars().collect();

    while i < chars.len() {
        let ch = chars[i];

        // Check if this is a variable name (A-Z or a-z)
        if ch.is_alphabetic() {
            // Try to match the longest variable name from our list
            let remaining: String = chars[i..].iter().collect();
            let mut matched_var: Option<String> = None;

            // Find the longest matching variable name
            for var in variables {
                if remaining.starts_with(var.as_str()) {
                    if matched_var.is_none() || var.len() > matched_var.as_ref().unwrap().len() {
                        matched_var = Some(var.clone());
                    }
                }
            }

            let var_name = if let Some(var) = matched_var {
                i += var.len();
                var
            } else {
                return Err(format!("Unknown variable at position {}: {}", i, remaining));
            };

            // Look for prime (negation)
            let negated = if i < chars.len() && chars[i] == '\'' {
                i += 1;
                true
            } else {
                false
            };

            let var_expr = if negated {
                BoolExpr::not(BoolExpr::var(&var_name))
            } else {
                BoolExpr::var(&var_name)
            };

            result = match result {
                None => Some(var_expr),
                Some(existing) => Some(BoolExpr::and(existing, var_expr)),
            };
        } else {
            i += 1;
        }
    }

    result.ok_or_else(|| format!("Failed to parse AND term: {}", term))
}

/// Format a comparison expression as a string
fn format_comparison(expr: &BoolExpr) -> String {
    match expr {
        BoolExpr::Equals(var, val) => format!("{} == {}", var, val),
        BoolExpr::NotEquals(var, val) => format!("{} != {}", var, val),
        BoolExpr::LessThan(var, val) => format!("{} < {}", var, val),
        BoolExpr::LessOrEqual(var, val) => format!("{} <= {}", var, val),
        BoolExpr::GreaterThan(var, val) => format!("{} > {}", var, val),
        BoolExpr::GreaterOrEqual(var, val) => format!("{} >= {}", var, val),
        _ => format_bool_expr(expr),
    }
}

/// Format a BoolExpr as a human-readable string
pub fn format_bool_expr(expr: &BoolExpr) -> String {
    match expr {
        BoolExpr::Var(name) => name.clone(),
        BoolExpr::Not(inner) => format!("!{}", format_bool_expr_with_parens(inner)),
        BoolExpr::And(left, right) => format!(
            "{} && {}",
            format_bool_expr_with_parens(left),
            format_bool_expr_with_parens(right)
        ),
        BoolExpr::Or(left, right) => {
            format!("{} || {}", format_and_expr(left), format_and_expr(right))
        }
        // Comparison operators
        _ => format_comparison(expr),
    }
}

fn format_bool_expr_with_parens(expr: &BoolExpr) -> String {
    match expr {
        BoolExpr::Var(_) | BoolExpr::Not(_) => format_bool_expr(expr),
        _ => format!("({})", format_bool_expr(expr)),
    }
}

fn format_and_expr(expr: &BoolExpr) -> String {
    match expr {
        BoolExpr::Or(_, _) => format!("({})", format_bool_expr(expr)),
        _ => format_bool_expr(expr),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simplify::types::BranchSet;

    #[test]
    fn test_simplify_basic_and() {
        // if a && b { return "1" }
        // elif a && !b { return "1" }
        // else { return "0" }
        // Should simplify to a simpler condition
        let mut branch_set = BranchSet::new();
        branch_set.add_branch(
            BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b")),
            "1",
        );
        branch_set.add_branch(
            BoolExpr::and(BoolExpr::var("a"), BoolExpr::not(BoolExpr::var("b"))),
            "1",
        );
        branch_set.set_default("0");

        let result = simplify_branches(&branch_set).unwrap();

        assert_eq!(result.original_branch_count, 2);
        assert_eq!(result.simplified_branch_count, 2); // "1" and "0"

        // Find the "1" output
        let one_branch = result
            .simplified_conditions
            .iter()
            .find(|(_, out)| out == "1")
            .unwrap();

        let formatted = format_bool_expr(&one_branch.0);
        println!("Simplified condition: {}", formatted);

        // The simplified condition should be simpler (no compound AND)
        // It should be a single variable (either a or b depending on QM output)
        assert!(!formatted.contains("&&"), "Should not contain compound AND");
    }

    #[test]
    fn test_parse_and_term() {
        let vars = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let expr = parse_and_term("ab", &vars).unwrap();
        assert_eq!(
            expr,
            BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b"))
        );

        let expr2 = parse_and_term("a'b", &vars).unwrap();
        assert_eq!(
            expr2,
            BoolExpr::and(BoolExpr::not(BoolExpr::var("a")), BoolExpr::var("b"))
        );
    }

    #[test]
    fn test_parse_and_term_multi_char() {
        // Test with multi-character variable names
        let vars = vec!["count".to_string(), "enabled".to_string()];

        let expr = parse_and_term("count", &vars).unwrap();
        assert_eq!(expr, BoolExpr::var("count"));

        let expr2 = parse_and_term("countenabled", &vars).unwrap();
        assert_eq!(
            expr2,
            BoolExpr::and(BoolExpr::var("count"), BoolExpr::var("enabled"))
        );

        let expr3 = parse_and_term("count'enabled", &vars).unwrap();
        assert_eq!(
            expr3,
            BoolExpr::and(
                BoolExpr::not(BoolExpr::var("count")),
                BoolExpr::var("enabled")
            )
        );
    }

    #[test]
    fn test_format_bool_expr() {
        let expr = BoolExpr::and(BoolExpr::var("a"), BoolExpr::var("b"));
        assert_eq!(format_bool_expr(&expr), "a && b");

        let expr2 = BoolExpr::or(BoolExpr::var("a"), BoolExpr::var("b"));
        assert_eq!(format_bool_expr(&expr2), "a || b");

        let expr3 = BoolExpr::not(BoolExpr::var("a"));
        assert_eq!(format_bool_expr(&expr3), "!a");
    }
}
