// Types for if-then-else simplification

use std::collections::HashMap;

/// A simple Boolean expression
#[derive(Debug, Clone, PartialEq)]
pub enum BoolExpr {
    Var(String),                         // a, b, flag
    Not(Box<BoolExpr>),                  // !a
    And(Box<BoolExpr>, Box<BoolExpr>),   // a && b
    Or(Box<BoolExpr>, Box<BoolExpr>),    // a || b

    // Comparison operators (for Phase 4)
    Equals(String, i32),                 // x == 5
    NotEquals(String, i32),              // x != 5
    LessThan(String, i32),               // x < 5
    LessOrEqual(String, i32),            // x <= 5
    GreaterThan(String, i32),            // x > 5
    GreaterOrEqual(String, i32),         // x >= 5
}

impl BoolExpr {
    pub fn var(name: &str) -> Self {
        BoolExpr::Var(name.to_string())
    }

    pub fn not(expr: BoolExpr) -> Self {
        BoolExpr::Not(Box::new(expr))
    }

    pub fn and(left: BoolExpr, right: BoolExpr) -> Self {
        BoolExpr::And(Box::new(left), Box::new(right))
    }

    pub fn or(left: BoolExpr, right: BoolExpr) -> Self {
        BoolExpr::Or(Box::new(left), Box::new(right))
    }

    // Comparison constructors
    pub fn equals(var: &str, value: i32) -> Self {
        BoolExpr::Equals(var.to_string(), value)
    }

    pub fn not_equals(var: &str, value: i32) -> Self {
        BoolExpr::NotEquals(var.to_string(), value)
    }

    pub fn less_than(var: &str, value: i32) -> Self {
        BoolExpr::LessThan(var.to_string(), value)
    }

    pub fn less_or_equal(var: &str, value: i32) -> Self {
        BoolExpr::LessOrEqual(var.to_string(), value)
    }

    pub fn greater_than(var: &str, value: i32) -> Self {
        BoolExpr::GreaterThan(var.to_string(), value)
    }

    pub fn greater_or_equal(var: &str, value: i32) -> Self {
        BoolExpr::GreaterOrEqual(var.to_string(), value)
    }
}

/// A single branch in an if-then-else chain
#[derive(Debug, Clone)]
pub struct Branch {
    pub condition: BoolExpr,
    pub output: String,  // For mini-MVP: just a string like "1" or "return true"
}

impl Branch {
    pub fn new(condition: BoolExpr, output: &str) -> Self {
        Self {
            condition,
            output: output.to_string(),
        }
    }
}

/// Variable type with domain information
#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Boolean,                    // True boolean variable
    Integer { min: i32, max: i32 },  // Integer with bounded domain
}

impl VariableType {
    /// Get the number of bits needed to represent this variable
    pub fn bit_count(&self) -> usize {
        match self {
            VariableType::Boolean => 1,
            VariableType::Integer { min, max } => {
                let range = (*max - *min + 1) as u32;
                (32 - range.leading_zeros()) as usize
            }
        }
    }

    /// Get the minimum value
    pub fn min_value(&self) -> i32 {
        match self {
            VariableType::Boolean => 0,
            VariableType::Integer { min, .. } => *min,
        }
    }

    /// Get the maximum value
    pub fn max_value(&self) -> i32 {
        match self {
            VariableType::Boolean => 1,
            VariableType::Integer { max, .. } => *max,
        }
    }
}

/// Collection of branches to simplify
#[derive(Debug, Clone)]
pub struct BranchSet {
    pub branches: Vec<Branch>,
    pub default_output: Option<String>,
    pub variable_types: HashMap<String, VariableType>,  // Variable domains
}

impl BranchSet {
    pub fn new() -> Self {
        Self {
            branches: Vec::new(),
            default_output: None,
            variable_types: HashMap::new(),
        }
    }

    pub fn add_branch(&mut self, condition: BoolExpr, output: &str) {
        self.branches.push(Branch::new(condition, output));
    }

    pub fn set_default(&mut self, output: &str) {
        self.default_output = Some(output.to_string());
    }

    /// Declare a variable type (needed for comparisons)
    pub fn declare_variable(&mut self, name: &str, var_type: VariableType) {
        self.variable_types.insert(name.to_string(), var_type);
    }

    /// Declare a boolean variable
    pub fn declare_bool(&mut self, name: &str) {
        self.declare_variable(name, VariableType::Boolean);
    }

    /// Declare an integer variable with bounded domain
    pub fn declare_int(&mut self, name: &str, min: i32, max: i32) {
        self.declare_variable(name, VariableType::Integer { min, max });
    }
}

/// Truth table representation for simplification
#[derive(Debug)]
pub struct TruthTable {
    pub variables: Vec<String>,
    pub output_groups: HashMap<String, Vec<u32>>,  // output -> list of minterms
    pub dont_cares: Vec<u32>,
}

impl TruthTable {
    pub fn new(variables: Vec<String>) -> Self {
        Self {
            variables,
            output_groups: HashMap::new(),
            dont_cares: Vec::new(),
        }
    }

    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }
}

/// Result of simplification
#[derive(Debug)]
pub struct SimplificationResult {
    pub variables: Vec<String>,
    pub simplified_conditions: Vec<(BoolExpr, String)>,  // (condition, output)
    pub original_branch_count: usize,
    pub simplified_branch_count: usize,
    pub analysis: SimplificationAnalysis,
}

impl SimplificationResult {
    pub fn complexity_reduction(&self) -> f64 {
        if self.original_branch_count == 0 {
            return 0.0;
        }
        if self.simplified_branch_count > self.original_branch_count {
            // Can happen when grouping by output creates more branches
            return 0.0;
        }
        let reduction = self.original_branch_count - self.simplified_branch_count;
        (reduction as f64 / self.original_branch_count as f64) * 100.0
    }
}

/// Analysis of branch coverage and dead code
#[derive(Debug, Clone)]
pub struct SimplificationAnalysis {
    pub branch_coverage: Vec<BranchCoverage>,
    pub dead_branches: Vec<DeadBranch>,
    pub uncovered_minterms: Vec<u32>,
    pub total_coverage_percent: f64,
}

impl SimplificationAnalysis {
    pub fn new() -> Self {
        Self {
            branch_coverage: Vec::new(),
            dead_branches: Vec::new(),
            uncovered_minterms: Vec::new(),
            total_coverage_percent: 0.0,
        }
    }

    pub fn has_dead_code(&self) -> bool {
        !self.dead_branches.is_empty()
    }

    pub fn has_coverage_gaps(&self) -> bool {
        !self.uncovered_minterms.is_empty()
    }
}

/// Coverage information for a single branch
#[derive(Debug, Clone)]
pub struct BranchCoverage {
    pub branch_index: usize,
    pub minterms_covered: Vec<u32>,
    pub coverage_count: usize,
    pub overlaps_with: Vec<usize>,  // Indices of branches that overlap
}

/// Information about unreachable dead code
#[derive(Debug, Clone)]
pub struct DeadBranch {
    pub branch_index: usize,
    pub reason: DeadCodeReason,
    pub covered_by: Vec<usize>,  // Which earlier branches make this unreachable
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeadCodeReason {
    FullyCovered,       // All conditions already handled by earlier branches
    Contradiction,      // Condition is logically impossible
    Redundant,         // Identical to an earlier branch
}
