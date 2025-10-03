//! MintermSet: A data structure for organizing minterms by bit count
//!
//! The MintermSet provides efficient organization of minterms grouped by their
//! Hamming weight (number of 1 bits), which is essential for the QM algorithm.

use super::encoding::{BitOps, MintermEncoding};

/// A set of minterms organized by bit count for efficient QM algorithm processing
///
/// Generic over the encoding type `E`, which determines the storage type
/// (u32 for Encoding16, u64 for Encoding32) and bucket array size.
#[derive(Debug, Clone)]
pub struct MintermSet<E: MintermEncoding> {
    data: Vec<Vec<E::Value>>,
    max_bit_count: usize,
    _phantom: std::marker::PhantomData<E>,
}

impl<E: MintermEncoding> MintermSet<E> {
    pub fn new() -> Self {
        Self {
            data: vec![Vec::new(); E::BUCKET_WIDTH],
            max_bit_count: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn add(&mut self, value: E::Value) {
        let bit_count = value.count_ones() as usize;
        if bit_count > self.max_bit_count {
            self.max_bit_count = bit_count;
        }
        self.data[bit_count].push(value);
    }

    pub fn add_all(&mut self, values: &[E::Value]) {
        for &value in values {
            self.add(value);
        }
    }

    pub fn get(&self, bit_count: usize) -> &[E::Value] {
        &self.data[bit_count]
    }

    pub fn get_max_bit_count(&self) -> usize {
        self.max_bit_count
    }
}

impl<E: MintermEncoding> Default for MintermSet<E> {
    fn default() -> Self {
        Self::new()
    }
}
