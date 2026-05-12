//! Implicant: Representation of implicants in the Quine-McCluskey algorithm
//!
//! Uses `SmallVec<[BitState; 64]>` so implicants for ≤64 variables stay inline
//! on the stack with zero heap allocation, while still supporting arbitrary sizes.

use super::encoding::{BitOps, MintermEncoding};
use smallvec::smallvec;
use crate::qm::quine_mccluskey::validate_prime_implicant;

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
/// 0, 1, or DontCare (X).  The `covered_minterms` list is a cache used
/// during the QM iteration to track which original minterms this cube
/// covers, enabling O(1) combination checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Implicant<E: MintermEncoding> {
    /// Per-variable bit states. Inline capacity of 64 covers all encoding types.
    pub bits: smallvec::SmallVec<[BitState; 64]>,
    /// Set of original minterms covered by this implicant.
    pub covered_minterms: Vec<E::Value>,
}

impl<E: MintermEncoding> Implicant<E> {
    /// Create from a single minterm (no dont-care bits).
    pub fn from_minterm(minterm: E::Value, variables: usize) -> Self {
        let mut bits = smallvec![BitState::Zero; variables];
        for i in 0..variables {
            bits[i] = if minterm.get_bit(i) {
                BitState::One
            } else {
                BitState::Zero
            };
        }

        Self {
            bits,
            covered_minterms: vec![minterm],
        }
    }

    /// Get a reference to the list of minterms covered by this implicant.
    #[inline]
    pub fn get_covered_minterms(&self) -> &[E::Value] {
        &self.covered_minterms
    }

    /// Get the bit state at position `index` in the implicant.
    ///
    /// Returns `BitState::DontCare` if the index is beyond the bit vector.
    #[inline]
    pub fn get_bit(&self, index: usize) -> BitState {
        self.bits.get(index).copied().unwrap_or(BitState::DontCare)
    }

    /// Check whether this implicant covers the given minterm.
    ///
    /// First checks the pre-computed `covered_minterms` list for speed,
    /// then falls back to bit-level matching (used for DontCare expansion
    /// from min-cubes).
    #[inline]
    pub fn covers_minterm(&self, minterm: E::Value) -> bool {
        if self.covered_minterms.contains(&minterm) {
            return true;
        }
        for (i, &state) in self.bits.iter().enumerate() {
            if state == BitState::DontCare {
                continue;
            }
            let expected = if state == BitState::One { 1u64 } else { 0u64 };
            let actual = (minterm.to_u64() >> i) & 1;
            if actual != expected {
                return false;
            }
        }
        true
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

    /// Convert implicant to raw encoding for batch operations.
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
        // data in lower bits, don't-care mask in upper bits; set data bits to 1 if don't care mask is set
        data | (dont_care << variables) | dont_care
    }

    /// Create from raw encoding (internal use only).
    pub(crate) fn from_raw_encoding(raw: E::Value, variables: usize) -> Self {
        let mask = (E::Value::one() << variables) - E::Value::one();
        let data = raw & mask;
        let dont_care_mask = raw >> variables;

        let mut bits = smallvec![BitState::Zero; variables];
        for i in 0..variables {
            let bit_pos = (variables - 1) - i;
            if dont_care_mask.get_bit(bit_pos) {
                bits[i] = BitState::DontCare;
            } else if data.get_bit(bit_pos) {
                bits[i] = BitState::One;
            } else {
                //bits[i] = BitState::Zero;
            }
        }

        Self {
            bits,
            covered_minterms: Vec::new(), // Empty - caller must set this!
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
