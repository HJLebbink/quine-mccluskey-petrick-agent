// Test consistency between reference (HashMap) and optimized (FxHash) implementations
// using random data to ensure both produce identical results

use qm_agent::qm::gray_code::{find_gray_code_pairs_fxhash, find_gray_code_pairs_ref};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;

/// Generate random u32 values with approximately k bits set
fn generate_random_encodings_u32(rng: &mut StdRng, count: usize, target_bits: u32) -> Vec<u32> {
    let mut encodings = HashSet::new();

    while encodings.len() < count {
        let mut pattern = 0u32;
        let mut bits_set = 0;

        while bits_set < target_bits {
            let bit_pos = rng.random_range(0..32);
            if (pattern & (1 << bit_pos)) == 0 {
                pattern |= 1 << bit_pos;
                bits_set += 1;
            }
        }

        encodings.insert(pattern);
    }

    encodings.into_iter().collect()
}

/// Generate random u64 values with approximately k bits set
fn generate_random_encodings_u64(rng: &mut StdRng, count: usize, target_bits: u32) -> Vec<u64> {
    let mut encodings = HashSet::new();

    while encodings.len() < count {
        let mut pattern = 0u64;
        let mut bits_set = 0;

        while bits_set < target_bits {
            let bit_pos = rng.random_range(0..64);
            if (pattern & (1 << bit_pos)) == 0 {
                pattern |= 1 << bit_pos;
                bits_set += 1;
            }
        }

        encodings.insert(pattern);
    }

    encodings.into_iter().collect()
}

/// Generate random u128 values with approximately k bits set
fn generate_random_encodings_u128(rng: &mut StdRng, count: usize, target_bits: u32) -> Vec<u128> {
    let mut encodings = HashSet::new();

    while encodings.len() < count {
        let mut pattern = 0u128;
        let mut bits_set = 0;

        while bits_set < target_bits {
            let bit_pos = rng.random_range(0..128);
            if (pattern & (1 << bit_pos)) == 0 {
                pattern |= 1 << bit_pos;
                bits_set += 1;
            }
        }

        encodings.insert(pattern);
    }

    encodings.into_iter().collect()
}

/// Helper: Convert pairs to a set for order-independent comparison
fn pairs_to_set(pairs: &[(usize, usize)]) -> HashSet<(usize, usize)> {
    pairs.iter().copied().collect()
}

#[test]
fn test_consistency_u32_small() {
    let mut rng = StdRng::seed_from_u64(42);

    let group1_encodings = generate_random_encodings_u32(&mut rng, 50, 8);
    let group2_encodings = generate_random_encodings_u32(&mut rng, 50, 9);

    let mut raw_encodings = group1_encodings.clone();
    raw_encodings.extend(group2_encodings);

    let group1_indices: Vec<usize> = (0..50).collect();
    let group2_indices: Vec<usize> = (50..100).collect();

    let pairs_ref = find_gray_code_pairs_ref(&group1_indices, &group2_indices, &raw_encodings);
    let pairs_fxhash =
        find_gray_code_pairs_fxhash(&group1_indices, &group2_indices, &raw_encodings);

    // Both now return (i, j) indices, so we can compare directly
    let set_ref = pairs_to_set(&pairs_ref);
    let set_fxhash = pairs_to_set(&pairs_fxhash);

    assert_eq!(
        set_ref.len(),
        set_fxhash.len(),
        "Different number of pairs: ref={}, fxhash={}",
        pairs_ref.len(),
        pairs_fxhash.len()
    );

    assert_eq!(set_ref, set_fxhash, "Pairs don't match");
}

#[test]
fn test_consistency_u32_medium() {
    let mut rng = StdRng::seed_from_u64(123);

    let group1_encodings = generate_random_encodings_u32(&mut rng, 200, 10);
    let group2_encodings = generate_random_encodings_u32(&mut rng, 200, 11);

    let mut raw_encodings = group1_encodings.clone();
    raw_encodings.extend(group2_encodings);

    let group1_indices: Vec<usize> = (0..200).collect();
    let group2_indices: Vec<usize> = (200..400).collect();

    let pairs_ref = find_gray_code_pairs_ref(&group1_indices, &group2_indices, &raw_encodings);
    let pairs_fxhash =
        find_gray_code_pairs_fxhash(&group1_indices, &group2_indices, &raw_encodings);

    let set_ref = pairs_to_set(&pairs_ref);
    let set_fxhash = pairs_to_set(&pairs_fxhash);

    assert_eq!(set_ref, set_fxhash, "Pairs don't match at medium size");
}

#[test]
fn test_consistency_u32_large() {
    let mut rng = StdRng::seed_from_u64(456);

    let group1_encodings = generate_random_encodings_u32(&mut rng, 1000, 12);
    let group2_encodings = generate_random_encodings_u32(&mut rng, 1000, 13);

    let mut raw_encodings = group1_encodings.clone();
    raw_encodings.extend(group2_encodings);

    let group1_indices: Vec<usize> = (0..1000).collect();
    let group2_indices: Vec<usize> = (1000..2000).collect();

    let pairs_ref = find_gray_code_pairs_ref(&group1_indices, &group2_indices, &raw_encodings);
    let pairs_fxhash =
        find_gray_code_pairs_fxhash(&group1_indices, &group2_indices, &raw_encodings);

    let set_ref = pairs_to_set(&pairs_ref);
    let set_fxhash = pairs_to_set(&pairs_fxhash);

    assert_eq!(set_ref.len(), set_fxhash.len(), "Large: Different counts");
    assert_eq!(set_ref, set_fxhash, "Large: Pairs don't match");
}

#[test]
fn test_consistency_u64() {
    let mut rng = StdRng::seed_from_u64(789);

    let group1_encodings = generate_random_encodings_u64(&mut rng, 100, 16);
    let group2_encodings = generate_random_encodings_u64(&mut rng, 100, 17);

    let mut raw_encodings = group1_encodings.clone();
    raw_encodings.extend(group2_encodings);

    let group1_indices: Vec<usize> = (0..100).collect();
    let group2_indices: Vec<usize> = (100..200).collect();

    let pairs_ref = find_gray_code_pairs_ref(&group1_indices, &group2_indices, &raw_encodings);
    let pairs_fxhash =
        find_gray_code_pairs_fxhash(&group1_indices, &group2_indices, &raw_encodings);

    let set_ref = pairs_to_set(&pairs_ref);
    let set_fxhash = pairs_to_set(&pairs_fxhash);

    assert_eq!(set_ref, set_fxhash, "u64: Pairs don't match");
}

#[test]
fn test_consistency_u128() {
    let mut rng = StdRng::seed_from_u64(101112);

    let group1_encodings = generate_random_encodings_u128(&mut rng, 100, 24);
    let group2_encodings = generate_random_encodings_u128(&mut rng, 100, 25);

    let mut raw_encodings = group1_encodings.clone();
    raw_encodings.extend(group2_encodings);

    let group1_indices: Vec<usize> = (0..100).collect();
    let group2_indices: Vec<usize> = (100..200).collect();

    let pairs_ref = find_gray_code_pairs_ref(&group1_indices, &group2_indices, &raw_encodings);
    let pairs_fxhash =
        find_gray_code_pairs_fxhash(&group1_indices, &group2_indices, &raw_encodings);

    let set_ref = pairs_to_set(&pairs_ref);
    let set_fxhash = pairs_to_set(&pairs_fxhash);

    assert_eq!(set_ref, set_fxhash, "u128: Pairs don't match");
}

#[test]
fn test_consistency_sparse_data() {
    let mut rng = StdRng::seed_from_u64(999);

    let group1_encodings = generate_random_encodings_u32(&mut rng, 150, 3);
    let group2_encodings = generate_random_encodings_u32(&mut rng, 150, 4);

    let mut raw_encodings = group1_encodings.clone();
    raw_encodings.extend(group2_encodings);

    let group1_indices: Vec<usize> = (0..150).collect();
    let group2_indices: Vec<usize> = (150..300).collect();

    let pairs_ref = find_gray_code_pairs_ref(&group1_indices, &group2_indices, &raw_encodings);
    let pairs_fxhash =
        find_gray_code_pairs_fxhash(&group1_indices, &group2_indices, &raw_encodings);

    let set_ref = pairs_to_set(&pairs_ref);
    let set_fxhash = pairs_to_set(&pairs_fxhash);

    assert_eq!(set_ref, set_fxhash, "Sparse: Pairs don't match");
}

#[test]
fn test_consistency_dense_data() {
    let mut rng = StdRng::seed_from_u64(888);

    let group1_encodings = generate_random_encodings_u32(&mut rng, 150, 28);
    let group2_encodings = generate_random_encodings_u32(&mut rng, 150, 29);

    let mut raw_encodings = group1_encodings.clone();
    raw_encodings.extend(group2_encodings);

    let group1_indices: Vec<usize> = (0..150).collect();
    let group2_indices: Vec<usize> = (150..300).collect();

    let pairs_ref = find_gray_code_pairs_ref(&group1_indices, &group2_indices, &raw_encodings);
    let pairs_fxhash =
        find_gray_code_pairs_fxhash(&group1_indices, &group2_indices, &raw_encodings);

    let set_ref = pairs_to_set(&pairs_ref);
    let set_fxhash = pairs_to_set(&pairs_fxhash);

    assert_eq!(set_ref, set_fxhash, "Dense: Pairs don't match");
}

#[test]
fn test_consistency_empty_groups() {
    let raw_encodings: Vec<u32> = vec![1, 2, 3, 4, 5];
    let group1_indices: Vec<usize> = vec![];
    let group2_indices: Vec<usize> = vec![0, 1, 2];

    let pairs_ref = find_gray_code_pairs_ref(&group1_indices, &group2_indices, &raw_encodings);
    let pairs_fxhash =
        find_gray_code_pairs_fxhash(&group1_indices, &group2_indices, &raw_encodings);

    assert_eq!(pairs_ref.len(), pairs_fxhash.len());
    assert_eq!(pairs_ref.len(), 0, "Empty group1 should produce no pairs");
}

#[test]
fn test_consistency_no_matches() {
    // Group1 and group2 have no gray code pairs
    let raw_encodings: Vec<u32> = vec![
        0b0000_0011, // 2 bits set
        0b0000_0101, // 2 bits set
        0b1111_0000, // 4 bits set (far from group1)
        0b1111_1100, // 6 bits set (far from group1)
    ];

    let group1_indices = vec![0, 1];
    let group2_indices = vec![2, 3];

    let pairs_ref = find_gray_code_pairs_ref(&group1_indices, &group2_indices, &raw_encodings);
    let pairs_fxhash =
        find_gray_code_pairs_fxhash(&group1_indices, &group2_indices, &raw_encodings);

    assert_eq!(pairs_ref.len(), pairs_fxhash.len());
    assert_eq!(pairs_ref.len(), 0, "No gray code pairs should exist");
}

#[test]
fn test_consistency_all_match() {
    // Every element in group1 matches with some in group2
    let raw_encodings: Vec<u32> = vec![
        0b0000_0001, // 1 bit
        0b0000_0010, // 1 bit
        0b0000_0011, // 2 bits - differs from [0] by bit 1, from [1] by bit 0
        0b0000_0110, // 2 bits - differs from [1] by bit 2
    ];

    let group1_indices = vec![0, 1];
    let group2_indices = vec![2, 3];

    let pairs_ref = find_gray_code_pairs_ref(&group1_indices, &group2_indices, &raw_encodings);
    let pairs_fxhash =
        find_gray_code_pairs_fxhash(&group1_indices, &group2_indices, &raw_encodings);

    let set_ref = pairs_to_set(&pairs_ref);
    let set_fxhash = pairs_to_set(&pairs_fxhash);

    assert_eq!(set_ref, set_fxhash);
    assert!(
        pairs_ref.len() >= 2,
        "Should find at least 2 gray code pairs"
    );
}

#[test]
fn test_consistency_multiple_seeds() {
    // Test with multiple random seeds to catch edge cases
    for seed in [1, 42, 123, 456, 789, 1000, 9999] {
        let mut rng = StdRng::seed_from_u64(seed);

        let group1_encodings = generate_random_encodings_u32(&mut rng, 50, 8);
        let group2_encodings = generate_random_encodings_u32(&mut rng, 50, 9);

        let mut raw_encodings = group1_encodings.clone();
        raw_encodings.extend(group2_encodings);

        let group1_indices: Vec<usize> = (0..50).collect();
        let group2_indices: Vec<usize> = (50..100).collect();

        let pairs_ref = find_gray_code_pairs_ref(&group1_indices, &group2_indices, &raw_encodings);
        let pairs_fxhash =
            find_gray_code_pairs_fxhash(&group1_indices, &group2_indices, &raw_encodings);

        let set_ref = pairs_to_set(&pairs_ref);
        let set_fxhash = pairs_to_set(&pairs_fxhash);

        assert_eq!(set_ref, set_fxhash, "Seed {}: Pairs don't match", seed);
    }
}

#[test]
fn test_consistency_varying_bit_patterns() {
    let mut rng = StdRng::seed_from_u64(12345);

    // Test with different bit densities
    for (bits_g1, bits_g2) in [(2, 3), (8, 9), (15, 16), (20, 21), (28, 29)] {
        let group1_encodings = generate_random_encodings_u32(&mut rng, 50, bits_g1);
        let group2_encodings = generate_random_encodings_u32(&mut rng, 50, bits_g2);

        let mut raw_encodings = group1_encodings.clone();
        raw_encodings.extend(group2_encodings);

        let group1_indices: Vec<usize> = (0..50).collect();
        let group2_indices: Vec<usize> = (50..100).collect();

        let pairs_ref = find_gray_code_pairs_ref(&group1_indices, &group2_indices, &raw_encodings);
        let pairs_fxhash =
            find_gray_code_pairs_fxhash(&group1_indices, &group2_indices, &raw_encodings);

        let set_ref = pairs_to_set(&pairs_ref);
        let set_fxhash = pairs_to_set(&pairs_fxhash);

        assert_eq!(
            set_ref, set_fxhash,
            "Bits {}/{}: Pairs don't match",
            bits_g1, bits_g2
        );
    }
}

#[test]
fn test_consistency_u128_large_values() {
    let mut rng = StdRng::seed_from_u64(54321);

    // Use values that are > u64::MAX to ensure u128 is properly tested
    let mut group1_encodings = generate_random_encodings_u128(&mut rng, 50, 40);
    let mut group2_encodings = generate_random_encodings_u128(&mut rng, 50, 41);

    // Force some values to use high bits
    for encoding in &mut group1_encodings {
        *encoding |= 1u128 << 100; // Set bit 100
    }
    for encoding in &mut group2_encodings {
        *encoding |= 1u128 << 100; // Set bit 100
    }

    let mut raw_encodings = group1_encodings.clone();
    raw_encodings.extend(group2_encodings);

    let group1_indices: Vec<usize> = (0..50).collect();
    let group2_indices: Vec<usize> = (50..100).collect();

    let pairs_ref = find_gray_code_pairs_ref(&group1_indices, &group2_indices, &raw_encodings);
    let pairs_fxhash =
        find_gray_code_pairs_fxhash(&group1_indices, &group2_indices, &raw_encodings);

    let set_ref = pairs_to_set(&pairs_ref);
    let set_fxhash = pairs_to_set(&pairs_fxhash);

    assert_eq!(set_ref, set_fxhash, "u128 large values: Pairs don't match");
}
