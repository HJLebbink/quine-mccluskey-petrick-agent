//! Integration tests for SIMD coverage acceleration

use qm_agent::qm::encoding::Enc16;
use qm_agent::qm::implicant::{BitState, Implicant};
use qm_agent::qm::petricks_method::PetricksMethod;

#[test]
fn test_petricks_method_4bit_small() {
    // Small 4-bit problem: should use scalar (below SIMD threshold)
    // 0X1X covers 2,3,6,7  (bit0=X, bit1=1, bit2=X, bit3=0)
    let mut pi1 = Implicant::<Enc16>::from_minterm(0, 4);
    pi1.bits.clear();
    pi1.bits.extend([
        BitState::DontCare,
        BitState::One,
        BitState::DontCare,
        BitState::Zero,
    ]);
    pi1.covered_minterms = vec![2, 3, 6, 7].into_iter().collect();

    // X11X covers 3,7,11,15 (bit0=1, bit1=1, bit2=X, bit3=X)
    let mut pi2 = Implicant::<Enc16>::from_minterm(0, 4);
    pi2.bits.clear();
    pi2.bits.extend([
        BitState::One,
        BitState::One,
        BitState::DontCare,
        BitState::DontCare,
    ]);
    pi2.covered_minterms = vec![3, 7, 11, 15].into_iter().collect();

    let prime_implicants = vec![pi1, pi2];
    let minterms = vec![2, 3, 6, 7, 11, 15];

    let petricks = PetricksMethod::new(&prime_implicants, &minterms);
    let cover = petricks.find_minimal_cover();

    assert!(cover.len() >= 2);
}

#[test]
fn test_coverage_correctness_4bit() {
    // Create implicant 0X1X (covers 2,3,6,7)
    let mut pi = Implicant::<Enc16>::from_minterm(0, 4);
    pi.bits.clear();
    pi.bits.extend([
        BitState::DontCare,
        BitState::One,
        BitState::DontCare,
        BitState::Zero,
    ]);
    pi.covered_minterms = vec![2, 3, 6, 7].into_iter().collect();

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

    // 0X1X covers 2,3,6,7
    let mut pi1 = Implicant::<Enc16>::from_minterm(0, 4);
    pi1.bits.clear();
    pi1.bits.extend([
        BitState::DontCare,
        BitState::One,
        BitState::DontCare,
        BitState::Zero,
    ]);
    pi1.covered_minterms = vec![2, 3, 6, 7].into_iter().collect();

    // 11XX covers 12,13,14,15  (bit2=1, bit3=1)
    let mut pi2 = Implicant::<Enc16>::from_minterm(0, 4);
    pi2.bits.clear();
    pi2.bits.extend([
        BitState::DontCare,
        BitState::DontCare,
        BitState::One,
        BitState::One,
    ]);
    pi2.covered_minterms = vec![12, 13, 14, 15].into_iter().collect();

    let prime_implicants = vec![pi1, pi2];
    let minterms: Vec<u32> = (0..16).collect();

    let coverage_matrix =
        unsafe { simd_coverage::build_coverage_matrix_simd_4bit(&prime_implicants, &minterms) };

    assert!(coverage_matrix.get(0, 2), "pi1 should cover 2");
    assert!(coverage_matrix.get(0, 3), "pi1 should cover 3");
    assert!(coverage_matrix.get(0, 6), "pi1 should cover 6");
    assert!(coverage_matrix.get(0, 7), "pi1 should cover 7");
    assert!(!coverage_matrix.get(0, 0), "pi1 should not cover 0");
    assert!(!coverage_matrix.get(0, 1), "pi1 should not cover 1");

    assert!(coverage_matrix.get(1, 12), "pi2 should cover 12");
    assert!(coverage_matrix.get(1, 13), "pi2 should cover 13");
    assert!(coverage_matrix.get(1, 14), "pi2 should cover 14");
    assert!(coverage_matrix.get(1, 15), "pi2 should cover 15");
}

#[cfg(all(target_arch = "x86_64", feature = "simd"))]
#[test]
fn test_simd_threshold_detection() {
    use qm_agent::qm::simd_coverage;

    assert!(!simd_coverage::should_use_simd(100, 4));
    println!(
        "SIMD available for large 4-bit problem: {}",
        simd_coverage::should_use_simd(10000, 4)
    );
    println!(
        "SIMD available for large 5-bit problem: {}",
        simd_coverage::should_use_simd(10000, 5)
    );
    assert!(!simd_coverage::should_use_simd(10000, 6));
}
