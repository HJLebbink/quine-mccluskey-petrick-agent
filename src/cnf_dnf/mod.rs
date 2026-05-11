// CNF to DNF conversion module
//
// This module provides Boolean CNF (Conjunctive Normal Form) to DNF (Disjunctive Normal Form)
// conversion with SIMD optimizations for x86_64 platforms.

pub mod convert;
pub mod error; // Error types
pub mod optimized_for; // Optimization level selection
pub mod utils; // Utility functions (string conversions) // Main conversion logic and algorithms

#[cfg(target_arch = "x86_64")]
pub mod simd; // SIMD-optimized implementations (AVX2, AVX512)

// Re-export main types and functions for convenience
pub use convert::{
    // Encoding-aware APIs with const generic optimization selection
    cnf_to_dnf,
    cnf_to_dnf_minimal,
    cnf_to_dnf_minimal_reference,
    cnf_to_dnf_with_names,
};
pub use error::CnfDnfError;
pub use optimized_for::OptimizedFor;
pub use utils::{cnf_to_string, dnf_to_string};
