use std::fmt;

/// Errors that can occur during CNF to DNF conversion
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CnfDnfError {
    /// The number of variables exceeds the encoding's maximum capacity
    EncodingCapacityExceeded {
        n_bits: usize,
        max_vars: usize,
    },
    /// The number of variables exceeds the optimization level's maximum capacity
    OptimizationLevelExceeded {
        n_bits: usize,
        optimization: String,
        max_bits: usize,
    },
    /// The number of variables exceeds the maximum supported (64)
    TooManyVariables {
        n_variables: usize,
    },
}

impl fmt::Display for CnfDnfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CnfDnfError::EncodingCapacityExceeded { n_bits, max_vars } => {
                write!(f, "n_bits ({}) exceeds encoding maximum ({})", n_bits, max_vars)
            }
            CnfDnfError::OptimizationLevelExceeded { n_bits, optimization, max_bits } => {
                write!(f, "n_bits ({}) exceeds {} maximum ({} bits)", n_bits, optimization, max_bits)
            }
            CnfDnfError::TooManyVariables { n_variables } => {
                write!(f, "too many different variables; found {} variables", n_variables)
            }
        }
    }
}

impl std::error::Error for CnfDnfError {}
