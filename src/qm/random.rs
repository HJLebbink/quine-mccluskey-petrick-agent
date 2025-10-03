//! Random minterm generation utilities
//!
//! This module provides utilities for generating random minterms for testing
//! and benchmarking the Quine-McCluskey algorithm.

use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::HashSet;
use std::hash::Hash;
use rand::distr::uniform::SampleUniform;

/// Trait for types that can be used as minterm values in random generation
pub trait RandomMinterm: SampleUniform + Ord + Hash + Copy + Sized {
    /// Maximum number of variables this type can represent
    const MAX_VARS: usize;

    /// Generate a random value in range [0, 2^n_variables)
    fn random_in_range(rng: &mut StdRng, n_variables: usize) -> Self;
}

impl RandomMinterm for u32 {
    const MAX_VARS: usize = 32;

    fn random_in_range(rng: &mut StdRng, n_variables: usize) -> Self {
        if n_variables >= 32 {
            rng.random_range(0..=u32::MAX)
        } else {
            let max = ((1u64 << n_variables) - 1) as u32;
            rng.random_range(0..=max)
        }
    }
}

impl RandomMinterm for u64 {
    const MAX_VARS: usize = 64;

    fn random_in_range(rng: &mut StdRng, n_variables: usize) -> Self {
        if n_variables >= 64 {
            rng.random_range(0..=u64::MAX)
        } else {
            let max = (1u64 << n_variables) - 1;
            rng.random_range(0..=max)
        }
    }
}

impl RandomMinterm for u128 {
    const MAX_VARS: usize = 128;

    fn random_in_range(rng: &mut StdRng, n_variables: usize) -> Self {
        if n_variables >= 128 {
            rng.random_range(0..=u128::MAX)
        } else {
            let max = (1u128 << n_variables) - 1;
            rng.random_range(0..=max)
        }
    }
}

/// Generate a vector of random unique minterms
///
/// This is a generic function that works with u32, u64, or u128.
///
/// # Type Parameters
/// * `T` - The integer type for minterms (u32, u64, or u128)
///
/// # Arguments
/// * `n_variables` - Number of Boolean variables (≤32 for u32, ≤64 for u64, ≤128 for u128)
/// * `n_minterms` - Number of unique minterms to generate
/// * `seed` - Random seed for reproducibility
///
/// # Returns
/// Vector of unique random minterms sorted in ascending order
///
/// # Panics
/// Panics if `n_variables` exceeds the type's capacity or is zero
///
/// # Examples
/// ```
/// use qm_agent::qm::random::generate_random_minterms;
///
/// // Generate 100 random minterms for 20 variables as u32
/// let minterms: Vec<u32> = generate_random_minterms(20, 100, 42);
/// assert_eq!(minterms.len(), 100);
///
/// // Generate 50 random minterms for 50 variables as u64
/// let minterms: Vec<u64> = generate_random_minterms(50, 50, 123);
/// assert_eq!(minterms.len(), 50);
///
/// // Generate 30 random minterms for 100 variables as u128
/// let minterms: Vec<u128> = generate_random_minterms(100, 30, 456);
/// assert_eq!(minterms.len(), 30);
/// ```
pub fn generate_random_minterms<T: RandomMinterm>(
    n_variables: usize,
    n_minterms: usize,
    seed: u64,
) -> Vec<T> {
    assert!(
        n_variables <= T::MAX_VARS,
        "Number of variables ({}) exceeds type capacity (max {})",
        n_variables,
        T::MAX_VARS
    );
    assert!(n_variables > 0, "Number of variables must be positive");

    let mut rng = StdRng::seed_from_u64(seed);
    let mut minterms = HashSet::new();

    // Generate unique random minterms
    while minterms.len() < n_minterms {
        let minterm = T::random_in_range(&mut rng, n_variables);
        minterms.insert(minterm);
    }

    let mut result: Vec<T> = minterms.into_iter().collect();
    result.sort_unstable();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_random_minterms_u32() {
        let minterms: Vec<u32> = generate_random_minterms(10, 50, 42);
        assert_eq!(minterms.len(), 50);
        // Check all unique
        let unique_count = minterms.iter().collect::<HashSet<_>>().len();
        assert_eq!(unique_count, 50);
        // Check sorted
        let mut sorted = minterms.clone();
        sorted.sort_unstable();
        assert_eq!(minterms, sorted);
        // Check all values fit in 10 bits
        assert!(minterms.iter().all(|&m| m < (1u32 << 10)));
    }

    #[test]
    fn test_generate_random_minterms_u64() {
        let minterms: Vec<u64> = generate_random_minterms(40, 100, 123);
        assert_eq!(minterms.len(), 100);
        let unique_count = minterms.iter().collect::<HashSet<_>>().len();
        assert_eq!(unique_count, 100);
        assert!(minterms.iter().all(|&m| m < (1u64 << 40)));
    }

    #[test]
    fn test_generate_random_minterms_u128() {
        let minterms: Vec<u128> = generate_random_minterms(80, 50, 999);
        assert_eq!(minterms.len(), 50);
        let unique_count = minterms.iter().collect::<HashSet<_>>().len();
        assert_eq!(unique_count, 50);
        assert!(minterms.iter().all(|&m| m < (1u128 << 80)));
    }

    #[test]
    fn test_reproducibility() {
        let minterms1: Vec<u32> = generate_random_minterms(16, 100, 42);
        let minterms2: Vec<u32> = generate_random_minterms(16, 100, 42);
        assert_eq!(minterms1, minterms2);
    }

    #[test]
    #[should_panic(expected = "exceeds type capacity")]
    fn test_u32_overflow() {
        let _: Vec<u32> = generate_random_minterms(33, 10, 42);
    }

    #[test]
    #[should_panic(expected = "must be positive")]
    fn test_zero_variables() {
        let _: Vec<u32> = generate_random_minterms(0, 10, 42);
    }
}
