//! Implicant: Representation of implicants in the Quine-McCluskey algorithm

use super::encoding::{BitOps, MintermEncoding};

/// State of a bit in an implicant: Zero, One, or DontCare
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitState {
    Zero,
    One,
    DontCare,
}

/// An implicant in the Quine-McCluskey algorithm
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Implicant<E: MintermEncoding> {
    pub bits: Vec<BitState>,
    pub covered_minterms: Vec<E::Value>,
}

impl<E: MintermEncoding> Implicant<E> {
    pub fn from_minterm(minterm: E::Value, variables: usize) -> Self {
        let mut bits = Vec::with_capacity(variables);
        // Iterate in reverse order to build MSB-first without needing reverse()
        for i in (0..variables).rev() {
            bits.push(if minterm.get_bit(i) {
                BitState::One
            } else {
                BitState::Zero
            });
        }

        Self {
            bits,
            covered_minterms: vec![minterm],
        }
    }

    #[inline]
    pub fn get_covered_minterms(&self) -> &[E::Value] {
        &self.covered_minterms
    }

    #[inline]
    pub fn get_bit(&self, index: usize) -> BitState {
        self.bits.get(index).copied().unwrap_or(BitState::DontCare)
    }

    #[inline]
    pub fn covers_minterm(&self, minterm: E::Value) -> bool {
        self.covered_minterms.contains(&minterm)
    }

    /// Fast gray code check using raw encoding (like C++ is_gray_code)
    /// Returns true if two raw encoded values differ by exactly 1 bit
    /// This checks BOTH data and don't-care masks - they must be identical except for 1 data bit
    #[inline]
    pub fn is_gray_code(a: E::Value, b: E::Value) -> bool {
        (a ^ b).count_ones() == 1
    }

    /// Fast combine using raw encoding (like C++ replace_complements)
    /// Only called after is_gray_code returns true (don't-care masks are identical)
    /// Preserves existing don't-cares and marks the differing bit as don't-care
    #[inline]
    pub fn replace_complements(a: E::Value, b: E::Value, variables: usize) -> E::Value {
        a | b | ((a ^ b) << variables)
    }

    /// Convert to raw encoding: lower bits = data, upper bits = don't-care mask
    /// This matches the C++ encoding for fast operations
    pub fn to_raw_encoding(&self, variables: usize) -> E::Value {
        let mut data = E::Value::zero();
        let mut dont_care = E::Value::zero();

        for i in 0..variables {
            let bit_pos = (variables - 1) - i; // MSB first
            match self.bits.get(i).copied().unwrap_or(BitState::DontCare) {
                BitState::One => {
                    data = data.set_bit(bit_pos);
                }
                BitState::Zero => {
                    // data bit stays 0
                }
                BitState::DontCare => {
                    dont_care = dont_care.set_bit(bit_pos);
                }
            }
        }

        // Combine: data in lower bits, don't-care mask in upper bits
        data | (dont_care << variables)
    }

    /// Create from raw encoding (internal use only)
    ///
    /// WARNING: covered_minterms will be EMPTY and MUST be set by the caller!
    /// This function is only used internally by the QM algorithm where
    /// covered_minterms are tracked separately and set immediately after creation.
    pub(crate) fn from_raw_encoding(raw: E::Value, variables: usize) -> Self {
        let mask = (E::Value::one() << variables) - E::Value::one();
        let data = raw & mask;
        let dont_care_mask = raw >> variables;

        let mut bits = Vec::with_capacity(variables);
        for i in 0..variables {
            let bit_pos = (variables - 1) - i; // MSB first
            if dont_care_mask.get_bit(bit_pos) {
                bits.push(BitState::DontCare);
            } else if data.get_bit(bit_pos) {
                bits.push(BitState::One);
            } else {
                bits.push(BitState::Zero);
            }
        }

        Self {
            bits,
            covered_minterms: Vec::new(),  // Empty - caller must set this!
        }
    }
}
