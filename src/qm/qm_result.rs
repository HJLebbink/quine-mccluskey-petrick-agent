//! QMResult: Result type for Quine-McCluskey minimization

/// Result of Quine-McCluskey minimization
#[derive(Debug, Clone, PartialEq)]
pub struct QMResult {
    /// The minimized sum-of-products expression (e.g. "A'B + AC")
    pub minimized_expression: String,
    /// All prime implicants found, as formatted strings
    pub prime_implicants: Vec<String>,
    /// Essential prime implicants that must appear in any minimal cover
    pub essential_prime_implicants: Vec<String>,
    /// Step-by-step description of the minimization process
    pub solution_steps: Vec<String>,
    /// Original cost = number of minterms × number of variables
    pub cost_original: usize,
    /// Minimized cost = number of selected prime implicants × 2
    pub cost_minimized: usize,
}
