//! SIMD-accelerated coverage matrix computation
//!
//! Uses bit-plane transposition and AVX-512 to check coverage for 512
//! minterm-implicant pairs simultaneously.

#[cfg(all(target_arch = "x86_64", feature = "simd"))]
use super::encoding::{BitOps, MintermEncoding};
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
use super::implicant::{BitState, Implicant};

/// Bit-packed coverage matrix for memory-efficient storage
///
/// Stores coverage as packed bits (8 bits per byte) in row-major order.
/// This is 8× more memory efficient than Vec<Vec<bool>> and allows direct
/// storage of SIMD output without unpacking.
#[derive(Debug, Clone)]
pub struct CoverageMatrix {
    /// Bit-packed data: data[row * row_bytes + col/8] contains bit for (row, col)
    data: Vec<u8>,
    /// Number of rows (prime implicants)
    num_rows: usize,
    /// Number of columns (minterms)
    num_cols: usize,
    /// Bytes per row (rounded up to nearest byte)
    row_bytes: usize,
}

impl CoverageMatrix {
    /// Create a new coverage matrix with all bits initialized to false
    pub fn new(num_rows: usize, num_cols: usize) -> Self {
        let row_bytes = (num_cols + 7) / 8;
        let data = vec![0u8; num_rows * row_bytes];
        Self {
            data,
            num_rows,
            num_cols,
            row_bytes,
        }
    }

    /// Get the coverage value at (row, col)
    #[inline]
    pub fn get(&self, row: usize, col: usize) -> bool {
        debug_assert!(row < self.num_rows);
        debug_assert!(col < self.num_cols);
        let byte_idx = row * self.row_bytes + col / 8;
        let bit_idx = col % 8;
        (self.data[byte_idx] >> bit_idx) & 1 == 1
    }

    /// Set the coverage value at (row, col)
    #[inline]
    pub fn set(&mut self, row: usize, col: usize, value: bool) {
        debug_assert!(row < self.num_rows);
        debug_assert!(col < self.num_cols);
        let byte_idx = row * self.row_bytes + col / 8;
        let bit_idx = col % 8;
        if value {
            self.data[byte_idx] |= 1 << bit_idx;
        } else {
            self.data[byte_idx] &= !(1 << bit_idx);
        }
    }

    /// Get the number of rows
    #[inline]
    pub fn num_rows(&self) -> usize {
        self.num_rows
    }

    /// Get the number of columns
    #[inline]
    pub fn num_cols(&self) -> usize {
        self.num_cols
    }

    /// Get mutable access to a row's raw bytes (for direct SIMD output)
    #[inline]
    pub fn row_bytes_mut(&mut self, row: usize) -> &mut [u8] {
        debug_assert!(row < self.num_rows);
        let start = row * self.row_bytes;
        &mut self.data[start..start + self.row_bytes]
    }

    /// Write striped bits directly to a row range (optimized for SIMD)
    ///
    /// Transposes from striped layout (bit-plane) to consecutive layout and writes to row.
    /// - striped[byte_idx] bit N contains value for index: byte_idx + N*64
    /// - Writes to columns starting at `col_offset`
    #[inline]
    pub fn write_striped_bits(&mut self, row: usize, col_offset: usize, striped: &[u8; 64]) {
        debug_assert!(row < self.num_rows);
        debug_assert!(col_offset + 512 <= self.num_cols || col_offset < self.num_cols);

        let start_byte = col_offset / 8;
        let start_bit = col_offset % 8;
        let num_cols = self.num_cols; // Save before mutable borrow

        // Fast path: aligned write (col_offset is multiple of 8)
        if start_bit == 0 {
            let num_bytes = ((num_cols - col_offset).min(512) + 7) / 8;
            let num_bits = num_cols - col_offset;
            let row_bytes = self.row_bytes_mut(row);
            transpose_striped_to_consecutive(striped, &mut row_bytes[start_byte..start_byte + num_bytes], num_bits);
        } else {
            // Slow path: unaligned write (rare, only for partial batches)
            for i in 0..512.min(num_cols - col_offset) {
                let byte_idx = i % 64;
                let bit_idx = i / 64;
                let value = (striped[byte_idx] >> bit_idx) & 1 == 1;
                self.set(row, col_offset + i, value);
            }
        }
    }
}

/// Transpose bits from striped layout (bit-plane) to consecutive layout
///
/// Input: 64 bytes where byte[i] bit N contains value for index: i + N*64
/// Output: up to 64 bytes where byte[i] bit N contains value for index: i*8 + N
///
/// This is an optimized transpose operation that avoids bit-by-bit iteration.
#[inline]
fn transpose_striped_to_consecutive(striped: &[u8; 64], output: &mut [u8], num_bits: usize) {
    let num_output_bytes = (num_bits.min(512) + 7) / 8;

    // Process 8 bytes at a time for better performance
    for out_group in 0..(num_output_bytes / 8) {
        // Each group of 8 output bytes
        for byte_in_group in 0..8 {
            let out_idx = out_group * 8 + byte_in_group;
            if out_idx >= num_output_bytes {
                break;
            }

            let mut out_byte = 0u8;

            // Gather 8 bits from striped layout
            // Output byte N contains bits for indices N*8 .. N*8+7
            let base_idx = out_idx * 8;
            for bit in 0..8 {
                let idx = base_idx + bit;
                if idx < num_bits.min(512) {
                    let in_byte = idx % 64;
                    let in_bit = idx / 64;
                    let bit_val = (striped[in_byte] >> in_bit) & 1;
                    out_byte |= bit_val << bit;
                }
            }

            output[out_idx] = out_byte;
        }
    }

    // Handle remaining bytes (if num_output_bytes not multiple of 8)
    for out_idx in (num_output_bytes / 8) * 8..num_output_bytes {
        let mut out_byte = 0u8;
        let base_idx = out_idx * 8;
        for bit in 0..8 {
            let idx = base_idx + bit;
            if idx < num_bits.min(512) {
                let in_byte = idx % 64;
                let in_bit = idx / 64;
                let bit_val = (striped[in_byte] >> in_bit) & 1;
                out_byte |= bit_val << bit;
            }
        }
        output[out_idx] = out_byte;
    }
}

/// Threshold for using SIMD optimization
/// Below this, the bit-plane conversion overhead dominates
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
const SIMD_THRESHOLD: usize = 1024;

/// Check if SIMD acceleration is available and worthwhile
pub fn should_use_simd(num_checks: usize, num_bits: usize) -> bool {
    // Only supports 4-bit for now
    if num_bits > 4 {
        return false;
    }

    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    {
        num_checks >= SIMD_THRESHOLD
            && is_x86_feature_detected!("avx512f")
            && is_x86_feature_detected!("gfni")
    }

    #[cfg(not(all(target_arch = "x86_64", feature = "simd")))]
    {
        let _ = num_checks;
        false
    }
}

/// Build coverage matrix using SIMD acceleration
///
/// For each prime implicant, checks which minterms it covers by processing
/// 512 minterms at a time using AVX-512.
///
/// Returns: CoverageMatrix with bit-packed storage where [i][j] = true if prime_implicant[i] covers minterm[j]
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
pub unsafe fn build_coverage_matrix_simd_4bit<E: MintermEncoding>(
    prime_implicants: &[Implicant<E>],
    minterms: &[E::Value],
) -> CoverageMatrix {
    let num_pi = prime_implicants.len();
    let num_mt = minterms.len();

    // Initialize result matrix (bit-packed)
    let mut coverage_matrix = CoverageMatrix::new(num_pi, num_mt);

    // Convert minterms to u8
    let minterms_u8: Vec<u8> = minterms
        .iter()
        .map(|&mt| mt.to_u64() as u8)
        .collect();

    // Process minterms in batches of 512
    let num_batches = (num_mt + 511) / 512;
    let padded_size = num_batches * 512;

    // Pad minterms to multiple of 512
    let mut padded_minterms = minterms_u8;
    padded_minterms.resize(padded_size, 0);

    // For each prime implicant
    for (pi_idx, pi) in prime_implicants.iter().enumerate() {
        // Extract implicant value and don't care mask
        let (implicant_value, dont_care_mask) = extract_implicant_representation(pi);

        // Check coverage for all minterms (512 at a time)
        for batch_idx in 0..num_batches {
            let offset = batch_idx * 512;

            // Prepare inputs for 512 coverage checks
            let coverage_bits = unsafe {
                check_coverage_batch_4bit(
                    implicant_value,
                    dont_care_mask,
                    &padded_minterms[offset..offset + 512],
                )
            };

            // Store results directly to coverage matrix (optimized bulk write)
            // Convert from striped layout to consecutive and write directly
            let coverage_array: [u8; 64] = coverage_bits.try_into().expect("Vec should be 64 bytes");
            coverage_matrix.write_striped_bits(pi_idx, offset, &coverage_array);
        }
    }

    coverage_matrix
}

/// Check coverage for a batch of 512 minterms
///
/// Returns: 64 bytes where bits indicate coverage (512 bits total)
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
unsafe fn check_coverage_batch_4bit(
    implicant_value: u8,
    dont_care_mask: u8,
    minterms: &[u8], // Must be exactly 512 values
) -> Vec<u8> {
    use bitwise_simd::bit_plane::*;
    use bitwise_simd::generated::_mm512_covers_4_4_4_1::_mm512_covers_4_4_4_1;
    use std::arch::x86_64::*;

    assert_eq!(minterms.len(), 512);

    unsafe {
        // Broadcast implicant and mask to 512 values
        let implicant_bytes = vec![implicant_value; 512];
        let mask_bytes = vec![dont_care_mask; 512];

        // Load into ZMM registers (8 registers × 64 bytes = 512 bytes)
        let mut implicant_regs = [_mm512_setzero_si512(); 8];
        let mut mask_regs = [_mm512_setzero_si512(); 8];
        let mut minterm_regs = [_mm512_setzero_si512(); 8];

        for reg in 0..8 {
            implicant_regs[reg] =
                _mm512_loadu_si512(implicant_bytes[reg * 64..].as_ptr() as *const __m512i);
            mask_regs[reg] = _mm512_loadu_si512(mask_bytes[reg * 64..].as_ptr() as *const __m512i);
            minterm_regs[reg] =
                _mm512_loadu_si512(minterms[reg * 64..].as_ptr() as *const __m512i);
        }

        // Separate into bit planes (only first 4 used for 4-bit values)
        let mut implicant_planes = [_mm512_setzero_si512(); 4];
        let mut mask_planes = [_mm512_setzero_si512(); 4];
        let mut minterm_planes = [_mm512_setzero_si512(); 4];

        bps_gfni_8to4(&implicant_regs, &mut implicant_planes);
        bps_gfni_8to4(&mask_regs, &mut mask_planes);
        bps_gfni_8to4(&minterm_regs, &mut minterm_planes);

        // Combine into input array: [minterm bits, mask bits, implicant bits]
        let mut input = [_mm512_setzero_si512(); 12];
        input[0..4].copy_from_slice(&minterm_planes);
        input[4..8].copy_from_slice(&mask_planes);
        input[8..12].copy_from_slice(&implicant_planes);

        // Execute coverage check for all 512 values
        let mut output = [_mm512_setzero_si512(); 1];
        _mm512_covers_4_4_4_1(&input, &mut output);

        // Extract results (512 bits packed in one ZMM register)
        let mut result = vec![0u8; 64];
        _mm512_storeu_si512(result.as_mut_ptr() as *mut __m512i, output[0]);

        result
    }
}

/// Extract implicant representation for coverage checking
///
/// Returns: (implicant_value, dont_care_mask)
/// - implicant_value: The fixed bit values (0s and 1s)
/// - dont_care_mask: 1 = don't care, 0 = must match
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
fn extract_implicant_representation<E: MintermEncoding>(implicant: &Implicant<E>) -> (u8, u8) {
    let mut value = 0u8;
    let mut mask = 0u8;

    for (i, bit) in implicant.bits.iter().enumerate() {
        match bit {
            BitState::Zero => {
                // value bit stays 0, mask bit stays 0 (must match)
            }
            BitState::One => {
                value |= 1 << i; // Set bit in value
                                 // mask bit stays 0 (must match)
            }
            BitState::DontCare => {
                mask |= 1 << i; // Set bit in mask (don't care)
            }
        }
    }

    (value, mask)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qm::encoding::Enc16;

    #[test]
    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    fn test_extract_implicant_representation() {
        // Test implicant: 0X1X
        let mut pi = Implicant::<Enc16>::from_minterm(0, 4);
        pi.bits = vec![
            BitState::Zero,     // bit 0: must be 0
            BitState::DontCare, // bit 1: don't care
            BitState::One,      // bit 2: must be 1
            BitState::DontCare, // bit 3: don't care
        ];

        let (value, mask) = extract_implicant_representation(&pi);

        // Value should have bit 2 set: 0b0100 = 4
        assert_eq!(value, 0b0100);

        // Mask should have bits 1 and 3 set: 0b1010 = 10
        assert_eq!(mask, 0b1010);
    }

    #[test]
    fn test_should_use_simd() {
        // Small problem: should not use SIMD
        assert!(!should_use_simd(100, 4));

        // Large problem: might use SIMD (if hardware supports it)
        let should_use = should_use_simd(10000, 4);
        // Can't assert true/false since depends on CPU features
        println!("SIMD available for large problem: {}", should_use);

        // 5-bit problem: not supported
        assert!(!should_use_simd(10000, 5));
    }
}
