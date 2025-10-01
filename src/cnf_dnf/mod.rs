// CNF to DNF conversion module
//
// This module provides Boolean CNF (Conjunctive Normal Form) to DNF (Disjunctive Normal Form)
// conversion with SIMD optimizations for x86_64 platforms.

pub mod convert;  // Main conversion logic and algorithms

#[cfg(target_arch = "x86_64")]
pub mod simd;     // SIMD-optimized implementations (AVX2, AVX512)

// Re-export main types and functions for convenience
pub use convert::{
    cnf_to_string, convert_cnf_to_dnf, convert_cnf_to_dnf_minimal, convert_cnf_to_dnf_with_names,
    dnf_to_string, OptimizedFor,
};
