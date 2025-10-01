//! QM Agent Library
//!
//! A Rust library for Boolean function minimization using the Quine-McCluskey
//! algorithm with Petrick's method.

pub mod qm;        // Quine-McCluskey algorithm and solver
pub mod cnf_dnf;   // CNF to DNF conversion with SIMD
pub mod simplify;  // If-then-else simplification

// Re-export the main types
pub use qm::{QMSolver, QMResult};
pub use qm::{QuineMcCluskey, DummyImplicant, BitState};
pub use qm::PetricksMethod;

/// Convenience function to minimize a Boolean function
pub fn minimize_function(
    minterms: &[u32],
    dont_cares: Option<&[u32]>,
    variables: usize
) -> QMResult {
    let mut solver = QMSolver::new(variables);
    solver.set_minterms(minterms);

    if let Some(dc) = dont_cares {
        solver.set_dont_cares(dc);
    }

    solver.solve()
}

/// Generate variable names (A, B, C, ...)
pub fn generate_variable_names(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| ((b'A' + i as u8) as char).to_string())
        .collect()
}

/// Parse a minterm string like "1,3,7,15"
pub fn parse_minterms(input: &str) -> Result<Vec<u32>, std::num::ParseIntError> {
    input.split(',')
        .map(|s| s.trim().parse())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimize_simple() {
        let result = minimize_function(&[1, 3], None, 2);
        assert!(!result.minimized_expression.is_empty());
    }

    #[test]
    fn test_generate_variable_names() {
        let names = generate_variable_names(4);
        assert_eq!(names, vec!["A", "B", "C", "D"]);
    }

    #[test]
    fn test_parse_minterms() {
        let minterms = parse_minterms("1,3,7,15").unwrap();
        assert_eq!(minterms, vec![1, 3, 7, 15]);
    }
}