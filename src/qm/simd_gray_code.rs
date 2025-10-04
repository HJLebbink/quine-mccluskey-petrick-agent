//! SIMD-accelerated gray code checking for Quine-McCluskey algorithm
//!
//! Uses AVX512 to vectorize the hot inner loop that checks if pairs of
//! implicants differ by exactly one bit.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Process gray code checks in batches using AVX512
/// Returns vector of (i, j) pairs that are gray codes
#[cfg(target_arch = "x86_64")]
pub fn find_gray_code_pairs_avx512_u64(
    group1_indices: &[usize],
    group2_indices: &[usize],
    raw_encodings: &[u64],
) -> Vec<(usize, usize)> {
    const LANES: usize = 8; // ZMM holds 8x u64

    if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vpopcntdq") {
        // Fallback to scalar
        return find_gray_code_pairs_scalar_u64(group1_indices, group2_indices, raw_encodings);
    }

    let mut pairs = Vec::new();

    // Gather group2 values into contiguous array for better vectorization
    let group2_values: Vec<u64> = group2_indices.iter()
        .map(|&idx| raw_encodings[idx])
        .collect();

    unsafe {
        for &i_idx in group1_indices {
            let raw_i = raw_encodings[i_idx];
            let raw_i_vec = _mm512_set1_epi64(raw_i as i64); // Broadcast
            let ones = _mm512_set1_epi64(1);

            let mut j_pos = 0;

            // Vectorized loop: process 8 j values at a time
            while j_pos + LANES <= group2_values.len() {
                // Load 8 raw_j values
                let raw_j_vec = _mm512_loadu_epi64(group2_values.as_ptr().add(j_pos) as *const i64);

                // XOR: Find differences
                let xor_vec = _mm512_xor_epi64(raw_i_vec, raw_j_vec);

                // Popcount: Count number of differing bits
                let popcount_vec = _mm512_popcnt_epi64(xor_vec);

                // Compare to 1: Find entries that differ by exactly 1 bit
                let mask = _mm512_cmpeq_epi64_mask(popcount_vec, ones);

                // Extract matches
                if mask != 0 {
                    for lane in 0..LANES {
                        if (mask & (1 << lane)) != 0 {
                            let j_idx = group2_indices[j_pos + lane];
                            pairs.push((i_idx, j_idx));
                        }
                    }
                }

                j_pos += LANES;
            }

            // Handle remainder with scalar code
            while j_pos < group2_values.len() {
                let j_idx = group2_indices[j_pos];
                let raw_j = raw_encodings[j_idx];
                if (raw_i ^ raw_j).count_ones() == 1 {
                    pairs.push((i_idx, j_idx));
                }
                j_pos += 1;
            }
        }
    }

    pairs
}

/// AVX512 version for u32 (processes 16 at a time)
#[cfg(target_arch = "x86_64")]
pub fn find_gray_code_pairs_avx512_u32(
    group1_indices: &[usize],
    group2_indices: &[usize],
    raw_encodings: &[u32],
) -> Vec<(usize, usize)> {
    const LANES: usize = 16; // ZMM holds 16x u32

    if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vpopcntdq") {
        return find_gray_code_pairs_scalar_u32(group1_indices, group2_indices, raw_encodings);
    }

    let mut pairs = Vec::new();
    let group2_values: Vec<u32> = group2_indices.iter()
        .map(|&idx| raw_encodings[idx])
        .collect();

    unsafe {
        for &i_idx in group1_indices {
            let raw_i = raw_encodings[i_idx];
            let raw_i_vec = _mm512_set1_epi32(raw_i as i32);
            let ones = _mm512_set1_epi32(1);

            let mut j_pos = 0;

            while j_pos + LANES <= group2_values.len() {
                let raw_j_vec = _mm512_loadu_epi32(group2_values.as_ptr().add(j_pos) as *const i32);
                let xor_vec = _mm512_xor_epi32(raw_i_vec, raw_j_vec);
                let popcount_vec = _mm512_popcnt_epi32(xor_vec);
                let mask = _mm512_cmpeq_epi32_mask(popcount_vec, ones);

                if mask != 0 {
                    for lane in 0..LANES {
                        if (mask & (1 << lane)) != 0 {
                            let j_idx = group2_indices[j_pos + lane];
                            pairs.push((i_idx, j_idx));
                        }
                    }
                }

                j_pos += LANES;
            }

            while j_pos < group2_values.len() {
                let j_idx = group2_indices[j_pos];
                let raw_j = raw_encodings[j_idx];
                if (raw_i ^ raw_j).count_ones() == 1 {
                    pairs.push((i_idx, j_idx));
                }
                j_pos += 1;
            }
        }
    }

    pairs
}

/// AVX512 version for u128 (processes 4 pairs of u64 at a time)
#[cfg(target_arch = "x86_64")]
pub fn find_gray_code_pairs_avx512_u128(
    group1_indices: &[usize],
    group2_indices: &[usize],
    raw_encodings: &[u128],
) -> Vec<(usize, usize)> {
    const LANES: usize = 4; // Process 4x u128 as 8x u64

    if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512vpopcntdq") {
        return find_gray_code_pairs_scalar_u128(group1_indices, group2_indices, raw_encodings);
    }

    let mut pairs = Vec::new();
    let group2_values: Vec<u128> = group2_indices.iter()
        .map(|&idx| raw_encodings[idx])
        .collect();

    unsafe {
        for &i_idx in group1_indices {
            let raw_i = raw_encodings[i_idx];
            let raw_i_lo = (raw_i & 0xFFFFFFFFFFFFFFFF) as u64;
            let raw_i_hi = (raw_i >> 64) as u64;

            let mut j_pos = 0;

            while j_pos + LANES <= group2_values.len() {
                // Process 4x u128 as 8x u64
                let mut lo_arr = [0u64; 4];
                let mut hi_arr = [0u64; 4];
                for k in 0..LANES {
                    let val = group2_values[j_pos + k];
                    lo_arr[k] = (val & 0xFFFFFFFFFFFFFFFF) as u64;
                    hi_arr[k] = (val >> 64) as u64;
                }

                // XOR low parts
                let raw_i_lo_vec = _mm256_set1_epi64x(raw_i_lo as i64);
                let raw_j_lo_vec = _mm256_loadu_epi64(lo_arr.as_ptr() as *const i64);
                let xor_lo = _mm256_xor_si256(raw_i_lo_vec, raw_j_lo_vec);

                // XOR high parts
                let raw_i_hi_vec = _mm256_set1_epi64x(raw_i_hi as i64);
                let raw_j_hi_vec = _mm256_loadu_epi64(hi_arr.as_ptr() as *const i64);
                let xor_hi = _mm256_xor_si256(raw_i_hi_vec, raw_j_hi_vec);

                // Popcount both parts
                let pop_lo = _mm256_popcnt_epi64(xor_lo);
                let pop_hi = _mm256_popcnt_epi64(xor_hi);

                // Add popcounts
                let total_pop = _mm256_add_epi64(pop_lo, pop_hi);

                // Compare to 1
                let ones = _mm256_set1_epi64x(1);
                let cmp = _mm256_cmpeq_epi64(total_pop, ones);
                let mask = _mm256_movemask_epi8(cmp);

                if mask != 0 {
                    for lane in 0..LANES {
                        if (mask & (0xFF << (lane * 8))) != 0 {
                            let j_idx = group2_indices[j_pos + lane];
                            pairs.push((i_idx, j_idx));
                        }
                    }
                }

                j_pos += LANES;
            }

            while j_pos < group2_values.len() {
                let j_idx = group2_indices[j_pos];
                let raw_j = raw_encodings[j_idx];
                if (raw_i ^ raw_j).count_ones() == 1 {
                    pairs.push((i_idx, j_idx));
                }
                j_pos += 1;
            }
        }
    }

    pairs
}

// Scalar fallbacks
fn find_gray_code_pairs_scalar_u64(
    group1_indices: &[usize],
    group2_indices: &[usize],
    raw_encodings: &[u64],
) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for &i in group1_indices {
        let raw_i = raw_encodings[i];
        for &j in group2_indices {
            let raw_j = raw_encodings[j];
            if (raw_i ^ raw_j).count_ones() == 1 {
                pairs.push((i, j));
            }
        }
    }
    pairs
}

fn find_gray_code_pairs_scalar_u32(
    group1_indices: &[usize],
    group2_indices: &[usize],
    raw_encodings: &[u32],
) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for &i in group1_indices {
        let raw_i = raw_encodings[i];
        for &j in group2_indices {
            let raw_j = raw_encodings[j];
            if (raw_i ^ raw_j).count_ones() == 1 {
                pairs.push((i, j));
            }
        }
    }
    pairs
}

fn find_gray_code_pairs_scalar_u128(
    group1_indices: &[usize],
    group2_indices: &[usize],
    raw_encodings: &[u128],
) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for &i in group1_indices {
        let raw_i = raw_encodings[i];
        for &j in group2_indices {
            let raw_j = raw_encodings[j];
            if (raw_i ^ raw_j).count_ones() == 1 {
                pairs.push((i, j));
            }
        }
    }
    pairs
}

#[cfg(not(target_arch = "x86_64"))]
pub fn find_gray_code_pairs_avx512_u64(
    group1_indices: &[usize],
    group2_indices: &[usize],
    raw_encodings: &[u64],
) -> Vec<(usize, usize)> {
    find_gray_code_pairs_scalar_u64(group1_indices, group2_indices, raw_encodings)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn find_gray_code_pairs_avx512_u32(
    group1_indices: &[usize],
    group2_indices: &[usize],
    raw_encodings: &[u32],
) -> Vec<(usize, usize)> {
    find_gray_code_pairs_scalar_u32(group1_indices, group2_indices, raw_encodings)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn find_gray_code_pairs_avx512_u128(
    group1_indices: &[usize],
    group2_indices: &[usize],
    raw_encodings: &[u128],
) -> Vec<(usize, usize)> {
    find_gray_code_pairs_scalar_u128(group1_indices, group2_indices, raw_encodings)
}
