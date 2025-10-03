//! Minterm encoding types for different bit widths
//!
//! This module defines encoding schemes for Boolean minterms with support
//! for 16-bit (u32), 32-bit (u64), and 64-bit (u128) representations.

use std::fmt;
use std::ops::{BitAnd, BitOr, BitXor, Not, Shl};

use crate::cnf_dnf::OptimizedFor;

/// Trait for integer types that can be used in bit operations
pub trait BitOps:
    Copy
    + Eq
    + Ord
    + std::hash::Hash
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Not<Output = Self>
    + Shl<usize, Output = Self>
    + fmt::Debug
{
    fn from_u64(val: u64) -> Self;
    fn to_u64(self) -> u64;
    fn count_ones(self) -> u32;
    fn zero() -> Self;
    fn one() -> Self;

    /// Check if bit at position `pos` is set
    fn get_bit(self, pos: usize) -> bool;
}

impl BitOps for u32 {
    #[inline]
    fn from_u64(val: u64) -> Self {
        val as u32
    }
    #[inline]
    fn to_u64(self) -> u64 {
        self as u64
    }
    #[inline]
    fn count_ones(self) -> u32 {
        self.count_ones()
    }
    #[inline]
    fn zero() -> Self {
        0u32
    }
    #[inline]
    fn one() -> Self {
        1u32
    }
    #[inline]
    fn get_bit(self, pos: usize) -> bool {
        (self & (1u32 << pos)) != 0
    }
}

impl BitOps for u64 {
    #[inline]
    fn from_u64(val: u64) -> Self {
        val
    }
    #[inline]
    fn to_u64(self) -> u64 {
        self
    }
    #[inline]
    fn count_ones(self) -> u32 {
        self.count_ones()
    }
    #[inline]
    fn zero() -> Self {
        0u64
    }
    #[inline]
    fn one() -> Self {
        1u64
    }
    #[inline]
    fn get_bit(self, pos: usize) -> bool {
        (self & (1u64 << pos)) != 0
    }
}

impl BitOps for u128 {
    #[inline]
    fn from_u64(val: u64) -> Self {
        val as u128
    }
    #[inline]
    fn to_u64(self) -> u64 {
        self as u64
    }
    #[inline]
    fn count_ones(self) -> u32 {
        self.count_ones()
    }
    #[inline]
    fn zero() -> Self {
        0u128
    }
    #[inline]
    fn one() -> Self {
        1u128
    }
    #[inline]
    fn get_bit(self, pos: usize) -> bool {
        (self & (1u128 << pos)) != 0
    }
}

/// Trait defining the encoding scheme for minterms
pub trait MintermEncoding: Copy + fmt::Debug {
    /// The integer type used for storing minterms
    type Value: BitOps;

    /// Offset for don't-care bits (16 for 16-bit mode, 32 for 32-bit mode, 64 for 64-bit mode)
    const DK_OFFSET: usize;

    /// Maximum number of variables supported
    const MAX_VARS: usize;

    /// Width of the MintermSet bucket array
    const BUCKET_WIDTH: usize;

    /// Get the recommended OptimizedFor variant for this encoding
    fn recommended_optimized_for() -> OptimizedFor;

    /// Check if an OptimizedFor variant is compatible with this encoding
    /// Returns true if the OptimizedFor can handle the encoding's MAX_VARS
    fn is_compatible_with(of: OptimizedFor) -> bool {
        of.max_bits() >= Self::MAX_VARS
    }
}

/// 16-bit encoding: uses u32, supports up to 16 variables
#[derive(Debug, Copy, Clone)]
pub struct Enc16;

impl MintermEncoding for Enc16 {
    type Value = u32;
    const DK_OFFSET: usize = 16;
    const MAX_VARS: usize = 16;
    const BUCKET_WIDTH: usize = 33;

    fn recommended_optimized_for() -> OptimizedFor {
        OptimizedFor::Avx512_16bits
    }
}

/// 32-bit encoding: uses u64, supports up to 32 variables
#[derive(Debug, Copy, Clone)]
pub struct Enc32;

impl MintermEncoding for Enc32 {
    type Value = u64;
    const DK_OFFSET: usize = 32;
    const MAX_VARS: usize = 32;
    const BUCKET_WIDTH: usize = 65;

    fn recommended_optimized_for() -> OptimizedFor {
        OptimizedFor::Avx512_32bits
    }
}

/// 64-bit encoding: uses u128, supports up to 64 variables
#[derive(Debug, Copy, Clone)]
pub struct Enc64;

impl MintermEncoding for Enc64 {
    type Value = u128;
    const DK_OFFSET: usize = 64;
    const MAX_VARS: usize = 64;
    const BUCKET_WIDTH: usize = 129;

    fn recommended_optimized_for() -> OptimizedFor {
        OptimizedFor::Avx512_64bits
    }
}
