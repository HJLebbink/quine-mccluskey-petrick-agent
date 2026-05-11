//! Min-cubes: prime implicant generation via condition combination enumeration
//!
//! Implements the min-cubes algorithm (from C++ reference at
//! `C:\Source\Private\cpp\Bitwise\minimize-cubes\`) for finding
//! all prime implicants via condition combination enumeration.

pub mod comb;
pub mod primes;
pub mod primes_adaptive;
pub mod setcover;

pub use primes::{
    TruthTable, find_prime_implicants, populate_covered_minterms_u64, prime_cubes_to_implicants,
};
#[allow(unused_imports)]
pub use primes_adaptive::find_prime_implicants_adaptive;
