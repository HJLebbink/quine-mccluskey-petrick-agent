//! QM Agent Library
//!
//! A Rust library for Boolean function minimization using the Quine-McCluskey
//! algorithm with Petrick's method.

#![feature(adt_const_params)]
#![allow(incomplete_features)]

pub mod qm;        // Quine-McCluskey algorithm and solver
pub mod cnf_dnf;   // CNF to DNF conversion with SIMD
pub mod simplify;  // If-then-else simplification

// Re-export the main types
pub use qm::{QMSolver, QMResult};
pub use qm::{QuineMcCluskey, Implicant, BitState};
pub use qm::PetricksMethod;
pub use qm::{Enc16, Enc32, Enc64};

/// Convenience function to minimize a Boolean function (up to 64 variables)
///
/// Automatically selects the most efficient encoding:
/// - Enc16 (u32 storage) for up to 16 variables
/// - Enc32 (u64 storage) for 17-32 variables
/// - Enc64 (u128 storage) for 33-64 variables
pub fn minimize_function(
    minterms: &[u64],
    dont_cares: Option<&[u64]>,
    variables: usize
) -> QMResult {
    if variables <= 16 {
        // Use Enc16 with u32 storage
        let mut solver = QMSolver::<Enc16>::new(variables);

        // Convert u64 to u32 for Enc16
        let minterms_u32: Vec<u32> = minterms.iter().map(|&x| x as u32).collect();
        solver.set_minterms(&minterms_u32);

        if let Some(dc) = dont_cares {
            let dc_u32: Vec<u32> = dc.iter().map(|&x| x as u32).collect();
            solver.set_dont_cares(&dc_u32);
        }

        solver.solve()
    } else if variables <= 32 {
        // Use Enc32 with u64 storage
        let mut solver = QMSolver::<Enc32>::new(variables);
        solver.set_minterms(minterms);

        if let Some(dc) = dont_cares {
            solver.set_dont_cares(dc);
        }

        solver.solve()
    } else if variables <= 64 {
        // Use Enc64 with u128 storage
        let mut solver = QMSolver::<Enc64>::new(variables);

        // Convert u64 to u128 for Enc64
        let minterms_u128: Vec<u128> = minterms.iter().map(|&x| x as u128).collect();
        solver.set_minterms(&minterms_u128);

        if let Some(dc) = dont_cares {
            let dc_u128: Vec<u128> = dc.iter().map(|&x| x as u128).collect();
            solver.set_dont_cares(&dc_u128);
        }

        solver.solve()
    } else {
        panic!("Variables must be <= 64");
    }
}

/// Generate variable names (A, B, C, ...)
pub fn generate_variable_names(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| ((b'A' + i as u8) as char).to_string())
        .collect()
}

/// Parse a minterm string like "1,3,7,15"
pub fn parse_minterms(input: &str) -> Result<Vec<u64>, std::num::ParseIntError> {
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

    #[test]
    fn test_minimize_64_variables() {
        // Test with 40 variables (should use Enc64)
        let minterms: Vec<u64> = vec![1, 3, 7];
        let result = minimize_function(&minterms, None, 40);
        assert!(!result.minimized_expression.is_empty());
    }

    #[test]
    #[should_panic(expected = "Variables must be <= 64")]
    fn test_minimize_too_many_variables() {
        let minterms: Vec<u64> = vec![1, 3, 7];
        minimize_function(&minterms, None, 65);
    }
}