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
        let mut bits = Vec::new();
        for i in 0..variables {
            if minterm.get_bit(i) {
                bits.push(BitState::One);
            } else {
                bits.push(BitState::Zero);
            }
        }
        bits.reverse(); // MSB first

        Self {
            bits,
            covered_minterms: vec![minterm],
        }
    }

    pub fn get_bit(&self, index: usize) -> BitState {
        self.bits.get(index).copied().unwrap_or(BitState::DontCare)
    }

    pub fn can_combine(&self, other: &Implicant<E>) -> bool {
        let mut diff_count = 0;
        for i in 0..self.bits.len() {
            if self.bits[i] != other.bits[i] {
                diff_count += 1;
                if diff_count > 1 {
                    return false;
                }
            }
        }
        diff_count == 1
    }

    pub fn combine(&self, other: &Implicant<E>) -> Option<Implicant<E>> {
        if !self.can_combine(other) {
            return None;
        }

        let mut new_bits = Vec::new();
        let mut covered = self.covered_minterms.clone();
        covered.extend(&other.covered_minterms);
        covered.sort_unstable();
        covered.dedup();

        for i in 0..self.bits.len() {
            if self.bits[i] == other.bits[i] {
                new_bits.push(self.bits[i]);
            } else {
                new_bits.push(BitState::DontCare);
            }
        }

        Some(Implicant {
            bits: new_bits,
            covered_minterms: covered,
        })
    }

    pub fn covers_minterm(&self, minterm: E::Value) -> bool {
        self.covered_minterms.contains(&minterm)
    }
}
