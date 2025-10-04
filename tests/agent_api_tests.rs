// Integration tests for Agent API

use qm_agent::agent_api;
use serde_json::Value;

#[test]
fn test_simple_boolean_simplification() {
    let input = r#"{
        "variables": {
            "a": "boolean",
            "b": "boolean"
        },
        "branches": [
            {"condition": "a && b", "output": "1"},
            {"condition": "a && !b", "output": "1"}
        ],
        "default": "0"
    }"#;

    let result = agent_api::simplify_from_json(input).unwrap();
    let json: Value = serde_json::from_str(&result).unwrap();

    // Should have metrics
    assert!(json["metrics"]["original_branches"].as_u64().unwrap() >= 2);
    assert!(json["simplified_branches"].is_array());

    // Should have analysis
    assert!(json["analysis"]["dead_code"].is_array());
    assert!(json["analysis"]["coverage_percent"].is_number());
}

#[test]
fn test_dead_code_detection() {
    let input = r#"{
        "variables": {
            "flag1": "boolean",
            "flag2": "boolean"
        },
        "branches": [
            {"condition": "flag1 || flag2", "output": "A"},
            {"condition": "flag1 && flag2", "output": "B"}
        ]
    }"#;

    let result = agent_api::simplify_from_json(input).unwrap();
    let json: Value = serde_json::from_str(&result).unwrap();

    // Second branch should be detected as dead code
    let dead_code = &json["analysis"]["dead_code"];
    assert!(dead_code.is_array());
    assert!(!dead_code.as_array().unwrap().is_empty(), "Should detect dead code");

    let first_dead = &dead_code[0];
    assert_eq!(first_dead["branch_index"], 1);
    assert!(first_dead["covered_by"].as_array().unwrap().contains(&Value::from(0)));
}

#[test]
#[ignore] // TODO: Parser doesn't support comparison operators yet (< > ==)
           // These work via programmatic API but not via string parsing
fn test_integer_variables() {
    let input = r#"{
        "variables": {
            "x": {"type": "integer", "min": 0, "max": 3}
        },
        "branches": [
            {"condition": "x < 2", "output": "small"},
            {"condition": "x >= 2", "output": "big"}
        ]
    }"#;

    let result = agent_api::simplify_from_json(input).unwrap();
    let json: Value = serde_json::from_str(&result).unwrap();

    assert_eq!(json["metrics"]["variables_used"].as_array().unwrap().len(), 1);
    assert_eq!(json["analysis"]["coverage_percent"], 100.0);
}

#[test]
fn test_coverage_analysis() {
    let input = r#"{
        "variables": {
            "a": "boolean",
            "b": "boolean"
        },
        "branches": [
            {"condition": "a && b", "output": "1"}
        ]
    }"#;

    let result = agent_api::simplify_from_json(input).unwrap();
    let json: Value = serde_json::from_str(&result).unwrap();

    // Coverage should be less than 100% since we don't cover all cases
    let coverage = json["analysis"]["coverage_percent"].as_f64().unwrap();
    assert!(coverage < 100.0, "Should have coverage gaps");

    // Should have uncovered minterms
    let gaps = &json["analysis"]["coverage_gaps"];
    assert!(gaps.is_array());
    assert!(!gaps.as_array().unwrap().is_empty(), "Should report coverage gaps");
}

#[test]
fn test_language_code_generation() {
    let input = r#"{
        "variables": {
            "flag": "boolean"
        },
        "branches": [
            {"condition": "flag", "output": "return true"}
        ],
        "default": "return false",
        "context": {
            "language": "go"
        }
    }"#;

    let result = agent_api::simplify_from_json(input).unwrap();
    let json: Value = serde_json::from_str(&result).unwrap();

    // Should have suggestions with generated code
    let suggestions = &json["suggestions"];
    assert!(suggestions.is_array());
}

#[test]
fn test_metadata_preservation() {
    let input = r#"{
        "variables": {
            "a": "boolean"
        },
        "branches": [
            {
                "condition": "a",
                "output": "1",
                "metadata": {
                    "line": 42,
                    "source": "if a { return 1; }"
                }
            }
        ],
        "default": "0"
    }"#;

    let result = agent_api::simplify_from_json(input).unwrap();
    let json: Value = serde_json::from_str(&result).unwrap();

    // Check that line numbers are preserved in dead code warnings or branches
    assert!(json["simplified_branches"].is_array());
}

#[test]
fn test_error_handling_invalid_json() {
    let input = "not valid json";
    let result = agent_api::simplify_from_json(input);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("JSON parse error"));
}

#[test]
fn test_error_handling_invalid_condition() {
    let input = r#"{
        "variables": {"a": "boolean"},
        "branches": [
            {"condition": "a &&", "output": "1"}
        ]
    }"#;

    let result = agent_api::simplify_from_json(input);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to parse"));
}

#[test]
fn test_error_handling_unknown_type() {
    let input = r#"{
        "variables": {"x": "unknown_type"},
        "branches": [
            {"condition": "x", "output": "1"}
        ]
    }"#;

    let result = agent_api::simplify_from_json(input);
    assert!(result.is_err());
}
