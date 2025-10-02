// SIMD-optimized CNF to DNF conversion using AVX2 and AVX512 intrinsics
//
// This module provides vectorized implementations for maximum performance
// on modern CPUs with AVX2/AVX512 support.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Test if a bit is set at a given position
#[inline]
fn test_bit(value: u64, pos: usize) -> bool {
    (value >> pos) & 1 == 1
}

/// Handle remaining elements not processed by SIMD (tail processing)
fn handle_tail_x64(
    result_dnf_next: &[u64],
    z: u64,
    start_index: usize,
    index_to_delete: &mut Vec<usize>,
) -> bool {
    for (index, &q) in result_dnf_next.iter().enumerate().skip(start_index) {
        let p = z | q;
        if p == z {
            return false; // z is subsumed
        }
        if p == q {
            index_to_delete.push(index);
        }
    }
    true
}

/// AVX512 optimized for 8-bit elements (64 elements per vector)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f,avx512bw")]
unsafe fn optimized_for_avx512_epi8(result_dnf_next: &[u64], z: u64) -> (Vec<usize>, bool) {
    const NB: usize = 6; // log2(64)
    let mut index_to_delete = Vec::with_capacity(64);

    let n = result_dnf_next.len();
    let n_blocks = n >> NB;

    unsafe {
        let z2 = _mm512_set1_epi8(z as i8);
        let ptr = result_dnf_next.as_ptr() as *const __m512i;

        for block in 0..n_blocks {
            let q = _mm512_loadu_si512(ptr.add(block));
            let p = _mm512_or_si512(z2, q);

            let mask1 = _mm512_cmpeq_epi8_mask(p, z2);
            if mask1 != 0 {
                return (Vec::new(), false);
            }

            let mask2 = _mm512_cmpeq_epi8_mask(p, q);
            if mask2 != 0 {
                for i in 0..(1 << NB) {
                    if test_bit(mask2, i) {
                        index_to_delete.push((block << NB) + i);
                    }
                }
            }
        }
    }

    let add_z = handle_tail_x64(result_dnf_next, z, n_blocks << NB, &mut index_to_delete);
    (index_to_delete, add_z)
}

/// AVX512 optimized for 16-bit elements (32 elements per vector)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn optimized_for_avx512_epi16_internal(result_dnf_next: &[u16], z: u16) -> (Vec<usize>, bool) {
    const NB: usize = 5; // log2(32)
    let mut index_to_delete = Vec::with_capacity(32);

    let n = result_dnf_next.len();
    let n_blocks = n >> NB;

    unsafe {
        let z2 = _mm512_set1_epi16(z as i16);
        let ptr = result_dnf_next.as_ptr() as *const __m512i;

        for block in 0..n_blocks {
            let q = _mm512_loadu_si512(ptr.add(block));
            let p = _mm512_or_si512(z2, q);

            let mask1 = _mm512_cmpeq_epi16_mask(p, z2);
            if mask1 != 0 {
                return (Vec::new(), false);
            }

            let mask2 = _mm512_cmpeq_epi16_mask(p, q);
            if mask2 != 0 {
                for i in 0..(1 << NB) {
                    if test_bit(mask2 as u64, i) {
                        index_to_delete.push((block << NB) + i);
                    }
                }
            }
        }
    }

    // Handle tail with u64 conversion for compatibility
    let result_u64: Vec<u64> = result_dnf_next.iter().map(|&x| x as u64).collect();
    let add_z = handle_tail_x64(&result_u64, z as u64, n_blocks << NB, &mut index_to_delete);
    (index_to_delete, add_z)
}

/// AVX512 optimized for 32-bit elements (16 elements per vector)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn optimized_for_avx512_epi32_internal(result_dnf_next: &[u32], z: u32) -> (Vec<usize>, bool) {
    const NB: usize = 4; // log2(16)
    let mut index_to_delete = Vec::with_capacity(16);

    let n = result_dnf_next.len();
    let n_blocks = n >> NB;

    unsafe {
        let z2 = _mm512_set1_epi32(z as i32);
        let ptr = result_dnf_next.as_ptr() as *const __m512i;

        for block in 0..n_blocks {
            let q = _mm512_loadu_si512(ptr.add(block));
            let p = _mm512_or_si512(z2, q);

            let mask1 = _mm512_cmpeq_epi32_mask(p, z2);
            if mask1 != 0 {
                return (Vec::new(), false);
            }

            let mask2 = _mm512_cmpeq_epi32_mask(p, q);
            if mask2 != 0 {
                for i in 0..(1 << NB) {
                    if test_bit(mask2 as u64, i) {
                        index_to_delete.push((block << NB) + i);
                    }
                }
            }
        }
    }

    // Handle tail with u64 conversion for compatibility
    let result_u64: Vec<u64> = result_dnf_next.iter().map(|&x| x as u64).collect();
    let add_z = handle_tail_x64(&result_u64, z as u64, n_blocks << NB, &mut index_to_delete);
    (index_to_delete, add_z)
}

/// AVX512 optimized for 64-bit elements (8 elements per vector)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn optimized_for_avx512_epi64_internal(result_dnf_next: &[u64], z: u64) -> (Vec<usize>, bool) {
    const NB: usize = 3; // log2(8)
    let mut index_to_delete = Vec::with_capacity(8);

    let n = result_dnf_next.len();
    let n_blocks = n >> NB;

    unsafe {
        let z2 = _mm512_set1_epi64(z as i64);
        let ptr = result_dnf_next.as_ptr() as *const __m512i;

        for block in 0..n_blocks {
            let q = _mm512_loadu_si512(ptr.add(block));
            let p = _mm512_or_si512(z2, q);

            let mask1 = _mm512_cmpeq_epi64_mask(p, z2);
            if mask1 != 0 {
                return (Vec::new(), false);
            }

            let mask2 = _mm512_cmpeq_epi64_mask(p, q);
            if mask2 != 0 {
                for i in 0..(1 << NB) {
                    if test_bit(mask2 as u64, i) {
                        index_to_delete.push((block << NB) + i);
                    }
                }
            }
        }
    }

    let add_z = handle_tail_x64(result_dnf_next, z, n_blocks << NB, &mut index_to_delete);
    (index_to_delete, add_z)
}

/// AVX2 optimized for 64-bit elements (4 elements per vector)
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn optimized_for_avx2_epi64_internal(result_dnf_next: &[u64], z: u64) -> (Vec<usize>, bool) {
    const NB: usize = 2; // log2(4)
    let mut index_to_delete = Vec::with_capacity(4);

    let n = result_dnf_next.len();
    let n_blocks = n >> NB;

    unsafe {
        let z2 = _mm256_set1_epi64x(z as i64);
        let ptr = result_dnf_next.as_ptr() as *const __m256i;

        for block in 0..n_blocks {
            let q = _mm256_loadu_si256(ptr.add(block));
            let p = _mm256_or_si256(z2, q);

            let cmp1 = _mm256_cmpeq_epi64(p, z2);
            let mask1 = _mm256_movemask_epi8(cmp1);
            if mask1 != 0 {
                return (Vec::new(), false);
            }

            let cmp2 = _mm256_cmpeq_epi64(p, q);
            let mask2 = _mm256_movemask_epi8(cmp2);
            if mask2 != 0 {
                for i in 0..(1 << NB) {
                    if test_bit(mask2 as u64, i << 3) {
                        index_to_delete.push((block << NB) + i);
                    }
                }
            }
        }
    }

    let add_z = handle_tail_x64(result_dnf_next, z, n_blocks << NB, &mut index_to_delete);
    (index_to_delete, add_z)
}

/// Public safe wrapper for AVX512 64-bit optimization
#[cfg(target_arch = "x86_64")]
pub fn run_avx512_64bits(result_dnf_next: &[u64], z: u64) -> (Vec<usize>, bool) {
    if is_x86_feature_detected!("avx512f") {
        // No type conversion needed - u64 is processed natively
        unsafe { optimized_for_avx512_epi64_internal(result_dnf_next, z) }
    } else {
        super::convert::optimized_for_x64(result_dnf_next, z)
    }
}

/// Public safe wrapper for AVX512 32-bit optimization
#[cfg(target_arch = "x86_64")]
pub fn run_avx512_32bits(result_dnf_next: &[u64], z: u64) -> (Vec<usize>, bool) {
    if is_x86_feature_detected!("avx512f") {
        // This ensures the SIMD operations work on correctly sized elements
        let result_u32: Vec<u32> = result_dnf_next.iter().map(|&x| x as u32).collect();
        unsafe { optimized_for_avx512_epi32_internal(&result_u32, z as u32) }
    } else {
        super::convert::optimized_for_x64(result_dnf_next, z)
    }
}

/// Public safe wrapper for AVX512 16-bit optimization
#[cfg(target_arch = "x86_64")]
pub fn run_avx512_16bits(result_dnf_next: &[u64], z: u64) -> (Vec<usize>, bool) {
    if is_x86_feature_detected!("avx512f") {
        // This ensures the SIMD operations work on correctly sized elements
        let result_u16: Vec<u16> = result_dnf_next.iter().map(|&x| x as u16).collect();
        unsafe { optimized_for_avx512_epi16_internal(&result_u16, z as u16) }
    } else {
        super::convert::optimized_for_x64(result_dnf_next, z)
    }
}

/// Public safe wrapper for AVX512 8-bit optimization
#[cfg(target_arch = "x86_64")]
pub fn run_avx512_8bits(result_dnf_next: &[u64], z: u64) -> (Vec<usize>, bool) {
    if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
        unsafe { optimized_for_avx512_epi8(result_dnf_next, z) }
    } else {
        super::convert::optimized_for_x64(result_dnf_next, z)
    }
}

/// Public safe wrapper for AVX2 64-bit optimization
#[cfg(target_arch = "x86_64")]
pub fn run_avx2_64bits(result_dnf_next: &[u64], z: u64) -> (Vec<usize>, bool) {
    if is_x86_feature_detected!("avx2") {
        // No type conversion needed - u64 is processed natively
        unsafe { optimized_for_avx2_epi64_internal(result_dnf_next, z) }
    } else {
        super::convert::optimized_for_x64(result_dnf_next, z)
    }
}

// Fallback implementations for non-x86_64 architectures
#[cfg(not(target_arch = "x86_64"))]
pub fn run_avx512_64bits(result_dnf_next: &[u64], z: u64) -> (Vec<usize>, bool) {
    super::convert::optimized_for_x64(result_dnf_next, z)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn run_avx512_32bits(result_dnf_next: &[u64], z: u64) -> (Vec<usize>, bool) {
    super::convert::optimized_for_x64(result_dnf_next, z)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn run_avx512_16bits(result_dnf_next: &[u64], z: u64) -> (Vec<usize>, bool) {
    super::convert::optimized_for_x64(result_dnf_next, z)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn run_avx512_8bits(result_dnf_next: &[u64], z: u64) -> (Vec<usize>, bool) {
    super::convert::optimized_for_x64(result_dnf_next, z)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn run_avx2_64bits(result_dnf_next: &[u64], z: u64) -> (Vec<usize>, bool) {
    super::convert::optimized_for_x64(result_dnf_next, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_availability() {
        #[cfg(target_arch = "x86_64")]
        {
            println!("AVX2 support: {}", is_x86_feature_detected!("avx2"));
            println!("AVX512F support: {}", is_x86_feature_detected!("avx512f"));
            println!("AVX512BW support: {}", is_x86_feature_detected!("avx512bw"));
        }
    }

    #[test]
    fn test_handle_tail() {
        // Test case 1: z is subsumed (p == z on first iteration)
        let data = vec![0b0110u64, 0b1010, 0b0011];
        let z = 0b1111u64;
        let mut index_to_delete = Vec::new();

        let result = handle_tail_x64(&data, z, 0, &mut index_to_delete);

        // z | 0b0110 = 0b1111 (equals z, so z is subsumed - return false immediately)
        assert_eq!(index_to_delete, Vec::<usize>::new());
        assert!(!result);

        // Test case 2: q's are subsumed (p == q)
        let data2 = vec![0b1111u64, 0b1110, 0b1111];
        let z2 = 0b0001u64;
        let mut index_to_delete2 = Vec::new();

        let result2 = handle_tail_x64(&data2, z2, 0, &mut index_to_delete2);

        // z | 0b1111 = 0b1111 (equals q, so q[0] is subsumed)
        // z | 0b1110 = 0b1111 (not equal to z or q, no action)
        // z | 0b1111 = 0b1111 (equals q, so q[2] is subsumed)
        assert_eq!(index_to_delete2, vec![0, 2]);
        assert!(result2);
    }
}
