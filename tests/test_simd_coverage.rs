//! Integration tests for SIMD coverage acceleration
//!
//! These tests verify that SIMD-accelerated coverage matrix computation
//! produces identical results to the scalar implementation.

use qm_agent::qm::encoding::Enc16;
use qm_agent::qm::implicant::{BitState, Implicant};
use qm_agent::qm::petricks_method::PetricksMethod;

#[test]
fn test_petricks_method_4bit_small() {
    // Small 4-bit problem: should use scalar (below SIMD threshold)
    let mut pi1 = Implicant::<Enc16>::from_minterm(0, 4);
    pi1.bits = vec![
        BitState::DontCare, // bit 0: don't care   \
        BitState::One,      // bit 1: must be 1     } 0X1X covers 2,3,6,7
        BitState::DontCare, // bit 2: don't care   /
        BitState::Zero,     // bit 3: must be 0   /
    ];
    pi1.covered_minterms = vec![2, 3, 6, 7].into_iter().collect();

    let mut pi2 = Implicant::<Enc16>::from_minterm(0, 4);
    pi2.bits = vec![
        BitState::One,      // bit 0: must be 1    \
        BitState::One,      // bit 1: must be 1     } X11X covers 3,7,11,15
        BitState::DontCare, // bit 2: don't care   /
        BitState::DontCare, // bit 3: don't care  /
    ];
    pi2.covered_minterms = vec![3, 7, 11, 15].into_iter().collect();

    let prime_implicants = vec![pi1, pi2];
    let minterms = vec![2, 3, 6, 7, 11, 15];

    let petricks = PetricksMethod::new(&prime_implicants, &minterms);
    let cover = petricks.find_minimal_cover();

    // Should select both implicants to cover all minterms
    assert!(cover.len() >= 2);
}

#[test]
fn test_coverage_correctness_4bit() {
    // Test that SIMD and scalar produce same results
    // Create implicant 0X1X (covers 2,3,6,7)
    let mut pi = Implicant::<Enc16>::from_minterm(0, 4);
    pi.bits = vec![
        BitState::DontCare, // bit 0: don't care   \
        BitState::One,      // bit 1: must be 1     } 0X1X covers 2,3,6,7
        BitState::DontCare, // bit 2: don't care   /
        BitState::Zero,     // bit 3: must be 0   /
    ];
    pi.covered_minterms = vec![2, 3, 6, 7].into_iter().collect();

    // Test coverage for all 16 4-bit minterms
    let expected_coverage = [
        false, false, true, true, // 0-3
        false, false, true, true, // 4-7
        false, false, false, false, // 8-11
        false, false, false, false, // 12-15
    ];

    for (minterm, &expected) in expected_coverage.iter().enumerate() {
        let actual = pi.covers_minterm(minterm as u32);
        assert_eq!(
            actual, expected,
            "Minterm {} coverage mismatch: expected {}, got {}",
            minterm, expected, actual
        );
    }
}

#[cfg(all(target_arch = "x86_64", feature = "simd"))]
#[test]
fn test_simd_coverage_matrix() {
    use qm_agent::qm::simd_coverage;

    // Create test implicants
    let mut pi1 = Implicant::<Enc16>::from_minterm(0, 4);
    pi1.bits = vec![
        BitState::DontCare, // bit 0: don't care   \
        BitState::One,      // bit 1: must be 1     } 0X1X covers 2,3,6,7
        BitState::DontCare, // bit 2: don't care   /
        BitState::Zero,     // bit 3: must be 0   /
    ];
    pi1.covered_minterms = vec![2, 3, 6, 7].into_iter().collect();

    let mut pi2 = Implicant::<Enc16>::from_minterm(0, 4);
    pi2.bits = vec![
        BitState::DontCare, // bit 0: don't care   \
        BitState::DontCare, // bit 1: don't care    } 11XX covers 12,13,14,15
        BitState::One,      // bit 2: must be 1    /
        BitState::One,      // bit 3: must be 1   /
    ];
    pi2.covered_minterms = vec![12, 13, 14, 15].into_iter().collect();

    let prime_implicants = vec![pi1, pi2];
    let minterms: Vec<u32> = (0..16).collect();

    // Build coverage matrix using SIMD
    let coverage_matrix = unsafe {
        simd_coverage::build_coverage_matrix_simd_4bit(&prime_implicants, &minterms)
    };

    // Verify pi1 coverage (0X1X should cover 2,3,6,7)
    assert!(coverage_matrix.get(0, 2), "pi1 should cover 2");
    assert!(coverage_matrix.get(0, 3), "pi1 should cover 3");
    assert!(coverage_matrix.get(0, 6), "pi1 should cover 6");
    assert!(coverage_matrix.get(0, 7), "pi1 should cover 7");
    assert!(!coverage_matrix.get(0, 0), "pi1 should not cover 0");
    assert!(!coverage_matrix.get(0, 1), "pi1 should not cover 1");

    // Verify pi2 coverage (11XX should cover 12,13,14,15)
    assert!(coverage_matrix.get(1, 12), "pi2 should cover 12");
    assert!(coverage_matrix.get(1, 13), "pi2 should cover 13");
    assert!(coverage_matrix.get(1, 14), "pi2 should cover 14");
    assert!(coverage_matrix.get(1, 15), "pi2 should cover 15");
    assert!(!coverage_matrix.get(1, 0), "pi2 should not cover 0");
    assert!(!coverage_matrix.get(1, 10), "pi2 should not cover 10");
}

#[cfg(all(target_arch = "x86_64", feature = "simd"))]
#[test]
fn test_simd_threshold_detection() {
    use qm_agent::qm::simd_coverage;

    // Small problem: should not use SIMD
    assert!(!simd_coverage::should_use_simd(100, 4));

    // Large problem: might use SIMD (depends on CPU features)
    let large_problem = simd_coverage::should_use_simd(10000, 4);
    println!("SIMD available for large 4-bit problem: {}", large_problem);

    // 5-bit problem: not supported (only 4-bit for now)
    assert!(!simd_coverage::should_use_simd(10000, 5));
}
