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
    /// Create a new empty minterm set with buckets for all possible bit counts.
    ///
    /// The bucket count is determined by `E::BUCKET_WIDTH`, which depends
    /// on the encoding type (33 for Encoding16, 65 for Encoding32, 129 for Encoding64).
    pub fn new() -> Self {
        Self {
            data: vec![Vec::new(); E::BUCKET_WIDTH],
            max_bit_count: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Add a single minterm to the set, placing it in the bucket
    /// corresponding to its Hamming weight (number of 1 bits).
    pub fn add(&mut self, value: E::Value) {
        let bit_count = value.count_ones() as usize;
        if bit_count > self.max_bit_count {
            self.max_bit_count = bit_count;
        }
        self.data[bit_count].push(value);
    }

    /// Add multiple minterms to the set in a single call.
    pub fn add_all(&mut self, values: &[E::Value]) {
        for &value in values {
            self.add(value);
        }
    }

    /// Get a reference to the bucket for the given Hamming weight.
    pub fn get(&self, bit_count: usize) -> &[E::Value] {
        &self.data[bit_count]
    }

    /// Get the maximum Hamming weight of any minterm added so far.
    pub fn get_max_bit_count(&self) -> usize {
        self.max_bit_count
    }
}

impl<E: MintermEncoding> Default for MintermSet<E> {
    fn default() -> Self {
        Self::new()
    }
}
