//! QMResult: Result type for Quine-McCluskey minimization

/// Result of Quine-McCluskey minimization
#[derive(Debug, Clone, PartialEq)]
pub struct QMResult {
    pub minimized_expression: String,
    pub prime_implicants: Vec<String>,
    pub essential_prime_implicants: Vec<String>,
    pub solution_steps: Vec<String>,
    pub cost_original: usize,
    pub cost_minimized: usize,
}
