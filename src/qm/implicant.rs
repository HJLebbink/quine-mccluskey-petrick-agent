//! Implicant: Representation of implicants in the Quine-McCluskey algorithm
//!
//! Uses packed E::Value for all bit state storage (One/Zero/DontCare per variable).

use super::encoding::{BitOps, MintermEncoding};
use crate::qm::quine_mccluskey::validate_prime_implicant;
use std::collections::HashSet;

/// State of a bit in an implicant: Zero, One, or DontCare
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitState {
    Zero,
    One,
    DontCare,
}

/// An implicant in the Quine-McCluskey algorithm.
///
/// An implicant represents a cube (a product term) in the Karnaugh map /
/// Boolean expression.  Each bit corresponds to a variable and can be
/// 0, 1, or DontCare (X).  The `covered_minterms` set is a cache used
/// during the QM iteration to track which original minterms this cube
/// covers, enabling O(1) combination checks and uniqueness.
///
/// Raw encoding layout stored in `bits`:
///   - Data bits: lower `n_variables` positions (1 = One, 0 = Zero)
///   - Don't-care bits: upper `n_variables` positions (1 = DontCare, 0 = One/Zero)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Implicant<E: MintermEncoding> {
    pub bits: E::Value,
    pub n_variables: usize,
    /// Set of original minterms covered by this implicant.
    pub covered_minterms: HashSet<E::Value>,
}

impl<E: MintermEncoding> Implicant<E> {
    /// Create from a single minterm (no dont-care bits).
    #[inline]
    pub fn from_minterm(minterm: E::Value, n_variables: usize) -> Self {
        Self {
            bits: minterm,
            n_variables,
            covered_minterms: {
                let mut s = HashSet::new();
                s.insert(minterm);
                s
            },
        }
    }

    /// Get an iterator over the minterms covered by this implicant.
    #[inline]
    pub fn covered_minterms_iter(&self) -> impl Iterator<Item = &E::Value> {
        self.covered_minterms.iter()
    }

    /// Get the bit state at position `index` in the implicant.
    #[inline]
    pub fn get_bit(&self, index: usize) -> BitState {
        if index < self.n_variables {
            if self.bits.get_bit(index + self.n_variables) {
                BitState::DontCare
            } else if self.bits.get_bit(index) {
                BitState::One
            } else {
                BitState::Zero
            }
        } else {
            BitState::DontCare
        }
    }

    /// Check whether this implicant covers the given minterm.
    ///
    /// First checks the pre-computed `covered_minterms` list for speed,
    /// then falls back to bit-level matching.
    #[inline]
    pub fn covers_minterm(&self, minterm: E::Value) -> bool {
        if self.covered_minterms.contains(&minterm) {
            return true;
        }
        let mask = self.get_dc_mask();
        (self.bits & !mask) == (minterm & !mask)
    }

    #[inline]
    fn get_dc_mask(&self) -> E::Value {
        self.bits >> self.n_variables
    }

    #[inline]
    pub fn is_gray_code(a: E::Value, b: E::Value) -> bool {
        (a ^ b).count_ones() == 1
    }

    #[inline]
    pub fn replace_complements(a: E::Value, b: E::Value, variables: usize) -> E::Value {
        let dont_care_mask: E::Value = a ^ b; // Bits that differ between a and b
        let result = a | b | (dont_care_mask << variables) | dont_care_mask;

        #[cfg(debug_assertions)]
        validate_prime_implicant::<E>(&result, variables);
        result
    }

    /// Create from raw encoding (internal use only).
    pub(crate) fn from_raw_encoding(raw: E::Value, n_variables: usize) -> Self {
        Self {
            bits: raw,
            n_variables,
            covered_minterms: HashSet::new(), // Empty - caller must set this!
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qm::encoding::Enc32;

    #[test]
    fn is_gray_code() {
        // Minterm 0 (0000) and minterm 1 (0001) differ by exactly 1 bit → gray code pair
        assert!(Implicant::<Enc32>::is_gray_code(0b00000000, 0b00000001));

        // Minterm 0 and minterm 3 (0011) differ by 2 bits → not gray code
        assert!(!Implicant::<Enc32>::is_gray_code(0b00000000, 0b00000011));

        // Minterm 3 and minterm 7 (0111) differ by exactly 1 bit → gray code pair
        assert!(Implicant::<Enc32>::is_gray_code(0b00000011, 0b00000111));

        // Minterm 5 (0101) and minterm 7 differ by exactly 1 bit → gray code pair
        assert!(Implicant::<Enc32>::is_gray_code(0b00000101, 0b00000111));

        // Identical values → 0 differing bits, not a gray code pair
        assert!(!Implicant::<Enc32>::is_gray_code(0, 0));

        // Minterm 0 and minterm 128 differ by exactly 1 bit → gray code pair
        assert!(Implicant::<Enc32>::is_gray_code(0, 1u64 << 7));
    }

    #[test]
    fn replace_complements() {
        {
            let a = 0b0000_0000_0000_0000;
            let b = 0b0000_0000_0000_0001;
            let c = 0b0000_0001_0000_0001;
            assert_eq!(Implicant::<Enc32>::replace_complements(a, b, 8), c);
        }
        {
            let a = 0b0000_0000_0000_0000;
            let b = 0b0000_0000_0000_0010;
            let c = 0b0000_0010_0000_0010;
            assert_eq!(Implicant::<Enc32>::replace_complements(a, b, 8), c);
        }
        {
            let a = 0b0000_0000_0000_0001;
            let b = 0b0000_0000_0000_0010;
            let c = 0b0000_0011_0000_0011;
            assert_eq!(Implicant::<Enc32>::replace_complements(a, b, 8), c);
        }
    }
}
