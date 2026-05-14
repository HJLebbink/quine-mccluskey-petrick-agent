//! Minterm encoding types for different bit widths
//!
//! This module defines encoding schemes for Boolean minterms with support
//! for 16-bit (u32), 32-bit (u64), and 64-bit (u128) representations.

use std::fmt;
use std::ops::{BitAnd, BitOr, BitXor, Not, Shl, Shr, Sub};

use super::gray_code;
use crate::cnf_dnf::OptimizedFor;

/// Trait for integer types that can be used in bit operations.
///
/// Implemented for u32 (Encoding16), u64 (Encoding32), and u128 (Encoding64).
/// Provides low-level bitwise operations needed for minterm manipulations.
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
    + Shr<usize, Output = Self>
    + Sub<Output = Self>
    + fmt::Debug
{
    fn from_u64(val: u64) -> Self;
    fn to_u64(self) -> u64;
    fn count_ones(self) -> u32;
    fn trailing_zeros(self) -> u32;
    fn zero() -> Self;
    fn one() -> Self;

    /// Check if bit at position `pos` is set
    fn get_bit(self, pos: usize) -> bool;

    /// Set bit at position `pos` to 1
    fn set_bit(self, pos: usize) -> Self;
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
    fn trailing_zeros(self) -> u32 {
        self.trailing_zeros()
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
    #[inline]
    fn set_bit(self, pos: usize) -> Self {
        self | (1u32 << pos)
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
    fn trailing_zeros(self) -> u32 {
        self.trailing_zeros()
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
    #[inline]
    fn set_bit(self, pos: usize) -> Self {
        self | (1u64 << pos)
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
    fn trailing_zeros(self) -> u32 {
        self.trailing_zeros()
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
    #[inline]
    fn set_bit(self, pos: usize) -> Self {
        self | (1u128 << pos)
    }
}

/// Trait defining the encoding scheme for minterms in the QM algorithm.
///
/// Three implementations exist:
/// - `Enc16`: uses u32 values, supports up to 16 variables
/// - `Enc32`: uses u64 values, supports up to 32 variables
/// - `Enc64`: uses u128 values, supports up to 64 variables
///
/// The generic type parameters control the value width, which affects
/// SIMD data parallelism (more values = more parallelism per vector).
pub trait MintermEncoding: Copy + fmt::Debug {
    /// The integer type used for storing minterms and implicant data.
    type Value: BitOps;

    /// Bit offset for don't-care bits in the raw encoding layout.
    /// Data bits occupy `0..DK_OFFSET`, don't-care mask occupies `DK_OFFSET..`
    const DK_OFFSET: usize;

    /// Maximum number of Boolean variables this encoding supports.
    const MAX_VARS: usize;

    /// Width of the MintermSet bucket array (max_bit_count + 1).
    const BUCKET_WIDTH: usize;

    /// Get the recommended SIMD optimization strategy for this encoding width.
    fn recommended_optimized_for() -> OptimizedFor;

    /// Check if an OptimizedFor variant is compatible with this encoding's variable limit.
    /// Returns true if the variant can handle the encoding's `MAX_VARS`.
    fn is_compatible_with(of: OptimizedFor) -> bool {
        of.max_bits() >= Self::MAX_VARS
    }

    /// Find all Hamming-distance-1 pairs between two groups of implicants.
    ///
    /// Given indices into `raw_encodings` for group1 and group2, returns pairs
    /// `(i, j)` where the two implicants differ by exactly one bit. Dispatches to
    /// AVX-512 implementations (16/8/4 parallel lanes for u32/u64/u128) with
    /// scalar fallback when AVX-512 is unavailable.
    fn find_gray_code_pairs(
        group1_indices: &[usize],
        group2_indices: &[usize],
        raw_encodings: &[Self::Value],
    ) -> Vec<(usize, usize)>;
}

/// 16-bit encoding: uses u32 values, supports up to 16 variables.
///
/// The `DK_OFFSET` is 16, meaning data bits occupy positions 0-15
/// and don't-care mask occupies positions 16-31 in the raw encoding.
/// Recommended SIMD optimization: `Avx512_16bits`.
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

    fn find_gray_code_pairs(
        group1_indices: &[usize],
        group2_indices: &[usize],
        raw_encodings: &[Self::Value],
    ) -> Vec<(usize, usize)> {
        gray_code::find_gray_code_pairs_fxhash(group1_indices, group2_indices, raw_encodings)
        //gray_code::find_gray_code_pairs_avx512_u32(group1_indices, group2_indices, raw_encodings)
    }
}

/// 32-bit encoding: uses u64 values, supports up to 32 variables.
///
/// The `DK_OFFSET` is 32, meaning data bits occupy positions 0-31
/// and don't-care mask occupies positions 32-63 in the raw encoding.
/// Recommended SIMD optimization: `Avx512_32bits`.
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

    fn find_gray_code_pairs(
        group1_indices: &[usize],
        group2_indices: &[usize],
        raw_encodings: &[Self::Value],
    ) -> Vec<(usize, usize)> {
        gray_code::find_gray_code_pairs_fxhash(group1_indices, group2_indices, raw_encodings)
        //gray_code::find_gray_code_pairs_avx512_u64(group1_indices, group2_indices, raw_encodings)
    }
}

/// 64-bit encoding: uses u128 values, supports up to 64 variables.
///
/// The `DK_OFFSET` is 64, meaning data bits occupy positions 0-63
/// and don't-care mask occupies positions 64-127 in the raw encoding.
/// Recommended SIMD optimization: `Avx512_64bits`.
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

    fn find_gray_code_pairs(
        group1_indices: &[usize],
        group2_indices: &[usize],
        raw_encodings: &[Self::Value],
    ) -> Vec<(usize, usize)> {
        gray_code::find_gray_code_pairs_fxhash(group1_indices, group2_indices, raw_encodings)
        //gray_code::find_gray_code_pairs_avx512_u128(group1_indices, group2_indices, raw_encodings)
    }
}
