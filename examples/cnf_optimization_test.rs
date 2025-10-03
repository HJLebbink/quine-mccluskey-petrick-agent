// Example demonstrating explicit optimization control in CNF to DNF conversion
//
// This example shows how to:
// 1. Use auto-detection (default behavior)
// 2. Force specific optimization levels (X64, AVX2, AVX512)
// 3. Compare results to ensure correctness across optimization levels

use qm_agent::cnf_dnf::{self, OptimizedFor};
use qm_agent::qm::Enc64;

fn main() {
    println!("CNF to DNF Optimization Testing Example");
    println!("========================================\n");

    // Example CNF: (A ∨ B) ∧ (A ∨ C) ∧ (B ∨ C)
    // Represented as bit vectors: A=bit0, B=bit1, C=bit2
    let cnf = vec![
        0b011u64,  // A ∨ B
        0b101u64,  // A ∨ C
        0b110u64,  // B ∨ C
    ];

    println!("Input CNF: {}", cnf_dnf::cnf_to_string(&cnf));
    println!("Expected DNF: (A&C) | (B&C) | (A&B)\n");

    // Test 1: Auto-detection (default)
    println!("Test 1: Auto-detection (AutoDetect)");
    let dnf_auto = cnf_dnf::convert_cnf_to_dnf::<Enc64, {OptimizedFor::AutoDetect}>(&cnf, 3);
    println!("  Result: {} terms", dnf_auto.len());
    println!("  DNF: {}\n", cnf_dnf::dnf_to_string(&dnf_auto));

    // Test 2: Force X64 scalar
    println!("Test 2: Force X64 scalar");
    let dnf_x64 = cnf_dnf::convert_cnf_to_dnf::<Enc64, {OptimizedFor::X64}>(
        &cnf,
        3,
    );
    println!("  Result: {} terms", dnf_x64.len());
    println!("  DNF: {}\n", cnf_dnf::dnf_to_string(&dnf_x64));

    // Test 3: Force AVX2 (if available)
    println!("Test 3: Force AVX2_64bits");
    let dnf_avx2 = cnf_dnf::convert_cnf_to_dnf::<Enc64, {OptimizedFor::Avx2_64bits}>(
        &cnf,
        3,
    );
    println!("  Result: {} terms", dnf_avx2.len());
    println!("  DNF: {}\n", cnf_dnf::dnf_to_string(&dnf_avx2));

    // Test 4: Force AVX512 (if available)
    println!("Test 4: Force AVX512_64bits");
    let dnf_avx512 = cnf_dnf::convert_cnf_to_dnf::<Enc64, {OptimizedFor::Avx512_64bits}>(
        &cnf,
        3,
    );
    println!("  Result: {} terms", dnf_avx512.len());
    println!("  DNF: {}\n", cnf_dnf::dnf_to_string(&dnf_avx512));

    // Verify all produce the same result
    println!("Verification:");
    assert_eq!(dnf_auto, dnf_x64, "Auto vs X64 mismatch!");
    assert_eq!(dnf_x64, dnf_avx2, "X64 vs AVX2 mismatch!");
    assert_eq!(dnf_avx2, dnf_avx512, "AVX2 vs AVX512 mismatch!");
    println!("  ✓ All optimization levels produce identical results\n");

    // Test with minimal DNF and early pruning
    println!("Testing minimal DNF with explicit optimization:");

    let cnf_complex = vec![
        0b1010u64,  // More complex CNF
        0b1100u64,
        0b0110u64,
    ];

    println!("Input CNF: {}", cnf_dnf::cnf_to_string(&cnf_complex));

    // Test with early pruning enabled
    let dnf_minimal_auto = cnf_dnf::convert_cnf_to_dnf_minimal::<Enc64, {OptimizedFor::AutoDetect}>(
        &cnf_complex,
        4,
        true,
    );
    println!("  Auto-detect with pruning: {} terms", dnf_minimal_auto.len());

    let dnf_minimal_x64 = cnf_dnf::convert_cnf_to_dnf_minimal::<Enc64, {OptimizedFor::X64}>(
        &cnf_complex,
        4,
        true,
    );
    println!("  X64 with pruning: {} terms", dnf_minimal_x64.len());

    assert_eq!(
        dnf_minimal_auto, dnf_minimal_x64,
        "Minimal DNF mismatch!"
    );
    println!("  ✓ Minimal DNF results match\n");

    println!("All tests passed! ✓");
    println!("\nUsage notes:");
    println!("  - Use AutoDetect for automatic hardware detection (recommended)");
    println!("  - Use X64Opt to force scalar (for debugging)");
    println!("  - Use Avx512_64bitsOpt to force AVX512");
    println!("  - Use Avx2_64bitsOpt to force AVX2");
}
