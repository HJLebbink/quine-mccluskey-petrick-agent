//! Test validation of optimization level vs number of variables

use qm_agent::cnf_dnf::{self, OptimizedFor};
use qm_agent::qm::Enc16;

fn main() {
    let cnf = vec![0b1010u64, 0b1100u64];

    println!("=== Testing Optimization Level Validation ===\n");

    // This should work: 8 bits with Avx512_8bits
    println!("Test 1: 8 variables with Avx512_8bits (should work)");
    let result = cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::Avx512_8bits}>(&cnf, 8);
    println!("  Result: {} terms\n", result.len());

    // This should fail: 16 bits with Avx512_8bits (max 8)
    println!("Test 2: 16 variables with Avx512_8bits (should fail)");
    let result = cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::Avx512_8bits}>(&cnf, 16);
    println!("  Result: {} terms (empty = validation failed)\n", result.len());

    // This should work: 16 bits with Avx512_16bits
    println!("Test 3: 16 variables with Avx512_16bits (should work)");
    let result = cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::Avx512_16bits}>(&cnf, 16);
    println!("  Result: {} terms\n", result.len());

    // This should work: AutoDetect always works (selects appropriate level)
    println!("Test 4: 16 variables with AutoDetect (should work)");
    let result = cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::AutoDetect}>(&cnf, 16);
    println!("  Result: {} terms\n", result.len());

    println!("=== Validation tests complete ===");
}
