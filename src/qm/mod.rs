//! Quine-McCluskey Boolean minimization module
//!
//! This module provides comprehensive Boolean function minimization using the
//! Quine-McCluskey algorithm, including both idiomatic Rust implementations and
//! C++ API-compatible versions.
//!
//! ## Module Organization
//!
//! **Core Algorithm:**
//! - [`implicant`] - BitState enum and Implicant struct
//! - [`quine_mccluskey`] - QuineMcCluskey algorithm implementation
//! - [`petricks_method`] - Petrick's method for minimal cover selection
//!
//! **High-Level Interface:**
//! - [`qm_solver`] - QMSolver orchestration
//! - [`qm_result`] - QMResult output type
//!
//! **Encoding and Data Structures:**
//! - [`encoding`] - BitOps trait, MintermEncoding trait, Encoding16/32/64
//! - [`minterm_set`] - MintermSet data structure
//!
//! **Testing and Utilities:**
//! - [`random`] - Random minterm generation for testing and benchmarking
//!
//! **C++ Compatibility:**
//! - [`classic`] - C++ API-compatible functions and utilities

// Core algorithm modules
pub mod implicant;
pub mod quine_mccluskey;
pub mod petricks_method;

// High-level interface
pub mod qm_solver;
pub mod qm_result;

// Encoding and data structures
pub mod encoding;
pub mod minterm_set;

// Testing and utilities
pub mod random;

// C++ compatibility and utilities
pub mod classic;

// Re-export main types for convenience
pub use implicant::{BitState, Implicant};
pub use quine_mccluskey::QuineMcCluskey;
pub use petricks_method::PetricksMethod;
pub use qm_result::QMResult;
pub use qm_solver::QMSolver;

// Re-export encoding types
pub use encoding::{BitOps, Enc16, Enc32, Enc64, MintermEncoding};
pub use minterm_set::MintermSet;

// Re-export classic algorithm functions for backward compatibility
pub use classic::{
    reduce_minterms, reduce_minterms_classic, reduce_minterms_with_early_pruning, reduce_qm,
};
