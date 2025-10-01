// Quine-McCluskey Boolean minimization module
//
// This module provides comprehensive Boolean function minimization using the
// Quine-McCluskey algorithm, including both idiomatic Rust implementations and
// C++ API-compatible versions.

pub mod algorithm;  // Core QM algorithm (DummyImplicant, BitState, QuineMcCluskey)
pub mod classic;    // C++ port with preserved naming (reduce_minterms_CLASSIC, etc.)
pub mod petricks;   // Petrick's method for minimal cover selection
pub mod solver;     // QMSolver orchestration and public API
pub mod utils;      // Utility functions

// Re-export main types for convenience
pub use algorithm::{BitState, DummyImplicant, QuineMcCluskey};
pub use petricks::PetricksMethod;
pub use solver::{QMResult, QMSolver};

// Re-export classic algorithm functions for backward compatibility
pub use classic::{
    reduce_minterms, reduce_minterms_CLASSIC, reduce_minterms_X, MintermSet,
};
