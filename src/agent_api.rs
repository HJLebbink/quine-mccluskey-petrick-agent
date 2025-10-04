//! JSON API for Claude integration
//!
//! This module provides a simple JSON-based API that allows Claude to use the QM agent
//! for boolean logic simplification across any programming language (Go, Rust, C++, etc.).
//!
//! Claude handles:
//! - Language parsing and understanding
//! - Variable type inference
//! - Side effect detection
//! - Code generation in target language
//!
//! QM Agent handles:
//! - Boolean algebra simplification
//! - Dead code detection
//! - Coverage analysis
//! - Optimization suggestions

use crate::simplify::{
    analyze_branches, format_bool_expr, parse_bool_expr, simplify_branches, BranchSet,
    SimplificationResult, VariableType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main request structure from Claude
#[derive(Debug, Deserialize)]
pub struct SimplificationRequest {
    /// Variable declarations with types and domains
    #[serde(default)]
    pub variables: HashMap<String, VariableSpec>,

    /// List of branches in order of evaluation
    pub branches: Vec<BranchSpec>,

    /// Default/else clause output
    #[serde(default)]
    pub default: Option<String>,

    /// Optional context about the code
    #[serde(default)]
    pub context: RequestContext,
}

/// Variable specification
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum VariableSpec {
    /// Simple type name: "boolean"
    Simple(String),
    /// Full specification with domain
    Full {
        #[serde(rename = "type")]
        var_type: String,
        min: Option<i32>,
        max: Option<i32>,
    },
}

impl VariableSpec {
    fn to_variable_type(&self) -> Result<VariableType, String> {
        match self {
            VariableSpec::Simple(s) => match s.as_str() {
                "boolean" | "bool" => Ok(VariableType::Boolean),
                _ => Err(format!("Unknown type: {}", s)),
            },
            VariableSpec::Full {
                var_type,
                min,
                max,
            } => match var_type.as_str() {
                "boolean" | "bool" => Ok(VariableType::Boolean),
                "integer" | "int" => {
                    let min = min.unwrap_or(0);
                    let max = max.ok_or_else(|| {
                        "Integer type requires 'max' field".to_string()
                    })?;
                    Ok(VariableType::Integer { min, max })
                }
                _ => Err(format!("Unknown type: {}", var_type)),
            },
        }
    }
}

/// Branch specification
#[derive(Debug, Deserialize)]
pub struct BranchSpec {
    /// Boolean condition as string (e.g., "a && b")
    pub condition: String,

    /// Output value or action (e.g., "return 1", "action_a()")
    pub output: String,

    /// Optional metadata about this branch
    #[serde(default)]
    pub metadata: BranchMetadata,
}

/// Metadata about a branch
#[derive(Debug, Default, Deserialize)]
pub struct BranchMetadata {
    /// Source line number
    pub line: Option<usize>,

    /// Whether this branch has side effects
    #[serde(default)]
    pub has_side_effects: bool,

    /// Original source code
    pub source: Option<String>,
}

/// Request context
#[derive(Debug, Default, Deserialize)]
pub struct RequestContext {
    /// Programming language: "go", "rust", "cpp", "python", etc.
    #[serde(default)]
    pub language: Option<String>,

    /// Whether to preserve evaluation order (for side effects)
    #[serde(default)]
    pub preserve_order: bool,

    /// Code style preference
    #[serde(default)]
    pub style: Option<String>,

    /// Whether this code was already analyzed by QM agent (skip re-analysis)
    #[serde(default)]
    pub already_analyzed: bool,

    /// Original source code (for including in suggestions when changes are made)
    #[serde(default)]
    pub original_code: Option<String>,
}

/// Main response structure to Claude
#[derive(Debug, Serialize, Deserialize)]
pub struct SimplificationResponse {
    /// Simplified branches
    pub simplified_branches: Vec<SimplifiedBranch>,

    /// Analysis results
    pub analysis: AnalysisResult,

    /// Code suggestions in target language
    pub suggestions: Vec<Suggestion>,

    /// Original complexity metrics
    pub metrics: ComplexityMetrics,
}

/// A simplified branch
#[derive(Debug, Serialize, Deserialize)]
pub struct SimplifiedBranch {
    /// Simplified condition
    pub condition: String,

    /// Output value
    pub output: String,

    /// Which original branches this combines
    pub original_lines: Vec<usize>,

    /// Is this the else/default clause?
    #[serde(default)]
    pub is_default: bool,
}

/// Analysis results
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Dead code warnings
    pub dead_code: Vec<DeadCodeWarning>,

    /// Coverage gaps (untested conditions)
    pub coverage_gaps: Vec<String>,

    /// Total coverage percentage
    pub coverage_percent: f64,

    /// Overlapping conditions
    pub overlaps: Vec<OverlapWarning>,
}

/// Dead code warning
#[derive(Debug, Serialize, Deserialize)]
pub struct DeadCodeWarning {
    /// Branch index
    pub branch_index: usize,

    /// Line number if available
    pub line: Option<usize>,

    /// Reason for dead code
    pub reason: String,

    /// Which branches already cover this
    pub covered_by: Vec<usize>,
}

/// Overlap warning
#[derive(Debug, Serialize, Deserialize)]
pub struct OverlapWarning {
    pub branch: usize,
    pub overlaps_with: Vec<usize>,
    pub message: String,
}

/// Code suggestion
#[derive(Debug, Serialize, Deserialize)]
pub struct Suggestion {
    /// Suggestion type: "simplification", "dead_code", "coverage"
    pub kind: String,

    /// Human-readable message
    pub message: String,

    /// Generated code (if applicable)
    pub code: Option<String>,

    /// Which lines this affects
    pub lines: Vec<usize>,
}

/// Complexity metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    pub original_branches: usize,
    pub simplified_branches: usize,
    pub complexity_reduction: f64,
    pub variables_used: Vec<String>,
}

/// Main entry point: simplify from JSON string
pub fn simplify_from_json(json: &str) -> Result<String, String> {
    let request: SimplificationRequest =
        serde_json::from_str(json).map_err(|e| format!("JSON parse error: {}", e))?;

    let response = process_request(request)?;

    serde_json::to_string_pretty(&response).map_err(|e| format!("JSON serialize error: {}", e))
}

/// Process a simplification request
fn process_request(request: SimplificationRequest) -> Result<SimplificationResponse, String> {
    // Check if code was already analyzed - skip re-analysis
    if request.context.already_analyzed {
        return Ok(SimplificationResponse {
            simplified_branches: vec![],
            analysis: AnalysisResult {
                dead_code: vec![],
                coverage_gaps: vec![],
                coverage_percent: 100.0,
                overlaps: vec![],
            },
            suggestions: vec![Suggestion {
                kind: "already_analyzed".to_string(),
                message: "Code was already analyzed by QM agent. Skipping re-analysis.".to_string(),
                code: None,
                lines: vec![],
            }],
            metrics: ComplexityMetrics {
                original_branches: 0,
                simplified_branches: 0,
                complexity_reduction: 0.0,
                variables_used: vec![],
            },
        });
    }

    // Convert to internal BranchSet
    let mut branch_set = BranchSet::new();

    // Register variables
    for (name, spec) in &request.variables {
        let var_type = spec.to_variable_type()?;
        match var_type {
            VariableType::Boolean => branch_set.declare_bool(name),
            VariableType::Integer { min, max } => branch_set.declare_int(name, min, max),
        }
    }

    // Parse and add branches
    for branch_spec in &request.branches {
        let condition = parse_bool_expr(&branch_spec.condition)
            .map_err(|e| format!("Failed to parse '{}': {}", branch_spec.condition, e))?;
        branch_set.add_branch(condition, &branch_spec.output);
    }

    // Set default if provided
    if let Some(ref default) = request.default {
        branch_set.set_default(default);
    }

    // Run simplification
    let result = simplify_branches(&branch_set)?;

    // Run analysis
    let analysis = analyze_branches(&branch_set)?;

    // Build response
    let response = build_response(request, result, analysis)?;

    Ok(response)
}

/// Build the response structure
fn build_response(
    request: SimplificationRequest,
    result: SimplificationResult,
    analysis: crate::simplify::SimplificationAnalysis,
) -> Result<SimplificationResponse, String> {
    // Convert simplified branches
    let mut simplified_branches = Vec::new();
    for (condition, output) in &result.simplified_conditions {
        let condition_str = format_bool_expr(condition);
        let is_default = condition_str == "true" || condition_str == "1";

        // Find which original lines this came from
        let original_lines: Vec<usize> = request
            .branches
            .iter()
            .enumerate()
            .filter_map(|(_i, b)| {
                if b.output == *output {
                    b.metadata.line
                } else {
                    None
                }
            })
            .collect();

        simplified_branches.push(SimplifiedBranch {
            condition: condition_str,
            output: output.clone(),
            original_lines,
            is_default,
        });
    }

    // Convert dead code warnings
    let dead_code: Vec<DeadCodeWarning> = analysis
        .dead_branches
        .iter()
        .map(|db| {
            let line = request
                .branches
                .get(db.branch_index)
                .and_then(|b| b.metadata.line);

            DeadCodeWarning {
                branch_index: db.branch_index,
                line,
                reason: format!("{:?}", db.reason),
                covered_by: db.covered_by.clone(),
            }
        })
        .collect();

    // Convert coverage gaps
    let coverage_gaps: Vec<String> = analysis
        .uncovered_minterms
        .iter()
        .take(10) // Limit to first 10
        .map(|&minterm| {
            crate::simplify::format_minterm(minterm, &result.variables)
        })
        .collect();

    // Find overlaps
    let overlaps: Vec<OverlapWarning> = analysis
        .branch_coverage
        .iter()
        .filter(|bc| !bc.overlaps_with.is_empty())
        .map(|bc| OverlapWarning {
            branch: bc.branch_index,
            overlaps_with: bc.overlaps_with.clone(),
            message: format!(
                "Branch {} overlaps with branches {:?}",
                bc.branch_index, bc.overlaps_with
            ),
        })
        .collect();

    let analysis_result = AnalysisResult {
        dead_code,
        coverage_gaps,
        coverage_percent: analysis.total_coverage_percent,
        overlaps,
    };

    // Generate suggestions
    let suggestions = generate_suggestions(
        &request,
        &result,
        &analysis_result,
        &simplified_branches,
    );

    // Calculate metrics
    let metrics = ComplexityMetrics {
        original_branches: result.original_branch_count,
        simplified_branches: result.simplified_branch_count,
        complexity_reduction: result.complexity_reduction(),
        variables_used: result.variables.clone(),
    };

    Ok(SimplificationResponse {
        simplified_branches,
        analysis: analysis_result,
        suggestions,
        metrics,
    })
}

/// Generate code suggestions
fn generate_suggestions(
    request: &SimplificationRequest,
    result: &SimplificationResult,
    analysis: &AnalysisResult,
    simplified: &[SimplifiedBranch],
) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    // Simplification suggestion - only if there's actual complexity reduction
    if result.complexity_reduction() > 0.0 {
        let language = request
            .context
            .language
            .as_deref()
            .unwrap_or("generic");

        let code = generate_code(simplified, language, request.context.original_code.as_deref());

        suggestions.push(Suggestion {
            kind: "simplification".to_string(),
            message: format!(
                "Simplified from {} to {} branches ({:.1}% reduction)",
                result.original_branch_count,
                result.simplified_branch_count,
                result.complexity_reduction()
            ),
            code: Some(code),
            lines: simplified
                .iter()
                .flat_map(|b| b.original_lines.clone())
                .collect(),
        });
    } else if result.complexity_reduction() == 0.0
        && analysis.dead_code.is_empty()
        && analysis.overlaps.is_empty() {
        // No simplification possible and no issues found
        suggestions.push(Suggestion {
            kind: "no_change".to_string(),
            message: "No simplification possible. The logic is already optimal.".to_string(),
            code: None,
            lines: vec![],
        });
    }

    // Dead code warnings
    for warning in &analysis.dead_code {
        suggestions.push(Suggestion {
            kind: "dead_code".to_string(),
            message: format!(
                "Branch at line {:?} is unreachable ({})",
                warning.line, warning.reason
            ),
            code: None,
            lines: warning.line.into_iter().collect(),
        });
    }

    // Coverage warnings
    if !analysis.coverage_gaps.is_empty() {
        suggestions.push(Suggestion {
            kind: "coverage".to_string(),
            message: format!(
                "Missing {} test cases. Coverage: {:.1}%",
                analysis.coverage_gaps.len(),
                analysis.coverage_percent
            ),
            code: None,
            lines: vec![],
        });
    }

    suggestions
}

/// Generate code in target language
fn generate_code(branches: &[SimplifiedBranch], language: &str, original_code: Option<&str>) -> String {
    let mut result = String::new();

    // Add original code as comments if provided
    if let Some(original) = original_code {
        result.push_str(&comment_out_code(original, language));
        result.push_str("\n");
        let comment = match language {
            "python" => "# QM-AGENT-SIMPLIFIED\n",
            _ => "// QM-AGENT-SIMPLIFIED\n",
        };
        result.push_str(comment);
    }

    // Generate new code
    let new_code = match language {
        "go" => generate_go_code(branches),
        "rust" => generate_rust_code(branches),
        "cpp" | "c++" => generate_cpp_code(branches),
        "python" => generate_python_code(branches),
        _ => generate_generic_code(branches),
    };

    result.push_str(&new_code);
    result
}

/// Comment out code based on language
fn comment_out_code(code: &str, language: &str) -> String {
    let comment_prefix = match language {
        "python" => "# ",
        _ => "// ", // C-style for Go, Rust, C++, etc.
    };

    let mut result = String::from(&format!("{}QM-AGENT-ORIGINAL:\n", comment_prefix));
    for line in code.lines() {
        result.push_str(&format!("{}{}\n", comment_prefix, line));
    }
    result
}

fn generate_go_code(branches: &[SimplifiedBranch]) -> String {
    let mut code = String::new();
    for (i, branch) in branches.iter().enumerate() {
        if branch.is_default {
            code.push_str(&format!("{}\n", branch.output));
        } else if i == 0 {
            code.push_str(&format!("if {} {{\n\t{}\n}}\n", branch.condition, branch.output));
        } else {
            code.push_str(&format!(
                "else if {} {{\n\t{}\n}}\n",
                branch.condition, branch.output
            ));
        }
    }
    code
}

fn generate_rust_code(branches: &[SimplifiedBranch]) -> String {
    let mut code = String::new();
    for (i, branch) in branches.iter().enumerate() {
        if branch.is_default {
            if i > 0 {
                code.push_str("else {\n\t");
            }
            code.push_str(&format!("{}\n", branch.output));
            if i > 0 {
                code.push_str("}\n");
            }
        } else if i == 0 {
            code.push_str(&format!("if {} {{\n\t{}\n}}\n", branch.condition, branch.output));
        } else {
            code.push_str(&format!(
                "else if {} {{\n\t{}\n}}\n",
                branch.condition, branch.output
            ));
        }
    }
    code
}

fn generate_cpp_code(branches: &[SimplifiedBranch]) -> String {
    let mut code = String::new();
    for (i, branch) in branches.iter().enumerate() {
        if branch.is_default {
            if i > 0 {
                code.push_str("else {\n\t");
            }
            code.push_str(&format!("{};\n", branch.output));
            if i > 0 {
                code.push_str("}\n");
            }
        } else if i == 0 {
            code.push_str(&format!(
                "if ({}) {{\n\t{};\n}}\n",
                branch.condition, branch.output
            ));
        } else {
            code.push_str(&format!(
                "else if ({}) {{\n\t{};\n}}\n",
                branch.condition, branch.output
            ));
        }
    }
    code
}

fn generate_python_code(branches: &[SimplifiedBranch]) -> String {
    let mut code = String::new();
    for (i, branch) in branches.iter().enumerate() {
        if branch.is_default {
            if i > 0 {
                code.push_str(&format!("else:\n\t{}\n", branch.output));
            } else {
                code.push_str(&format!("{}\n", branch.output));
            }
        } else if i == 0 {
            code.push_str(&format!("if {}:\n\t{}\n", branch.condition, branch.output));
        } else {
            code.push_str(&format!("elif {}:\n\t{}\n", branch.condition, branch.output));
        }
    }
    code
}

fn generate_generic_code(branches: &[SimplifiedBranch]) -> String {
    let mut code = String::new();
    for branch in branches {
        if branch.is_default {
            code.push_str(&format!("default: {}\n", branch.output));
        } else {
            code.push_str(&format!("if {}: {}\n", branch.condition, branch.output));
        }
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_json_request() {
        let json = r#"{
            "variables": {
                "a": "boolean",
                "b": "boolean"
            },
            "branches": [
                {"condition": "a && b", "output": "return 1"},
                {"condition": "a && !b", "output": "return 1"}
            ],
            "default": "return 0"
        }"#;

        let response = simplify_from_json(json).unwrap();
        assert!(response.contains("simplified_branches"));
        assert!(response.contains("analysis"));

        // Parse back to verify
        let parsed: SimplificationResponse = serde_json::from_str(&response).unwrap();
        assert!(parsed.simplified_branches.len() >= 1);
        assert_eq!(parsed.metrics.original_branches, 2);
    }

    #[test]
    #[ignore] // TODO: Parser doesn't support comparison operators yet
    fn test_integer_variables() {
        let json = r#"{
            "variables": {
                "x": {"type": "integer", "min": 0, "max": 3}
            },
            "branches": [
                {"condition": "x < 2", "output": "small"},
                {"condition": "x >= 2", "output": "big"}
            ]
        }"#;

        let response = simplify_from_json(json).unwrap();
        assert!(response.contains("simplified_branches"));
    }

    #[test]
    fn test_dead_code_detection() {
        let json = r#"{
            "variables": {
                "a": "boolean",
                "b": "boolean"
            },
            "branches": [
                {"condition": "a || b", "output": "1", "metadata": {"line": 10}},
                {"condition": "a && b", "output": "2", "metadata": {"line": 12}}
            ]
        }"#;

        let response = simplify_from_json(json).unwrap();
        let parsed: SimplificationResponse = serde_json::from_str(&response).unwrap();

        // Second branch should be detected as dead code
        assert!(!parsed.analysis.dead_code.is_empty());
    }

    #[test]
    fn test_code_generation_go() {
        let branches = vec![
            SimplifiedBranch {
                condition: "a".to_string(),
                output: "return 1".to_string(),
                original_lines: vec![10],
                is_default: false,
            },
            SimplifiedBranch {
                condition: "true".to_string(),
                output: "return 0".to_string(),
                original_lines: vec![15],
                is_default: true,
            },
        ];

        let code = generate_go_code(&branches);
        assert!(code.contains("if a {"));
        assert!(code.contains("return 1"));
        assert!(code.contains("return 0"));
    }
}
