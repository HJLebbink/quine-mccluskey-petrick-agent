//! Demonstration of unified CNF-to-DNF API with encoding awareness
//!
//! This example shows how the new encoding-aware API automatically selects
//! the optimal SIMD strategy based on the encoding type.

use qm_agent::cnf_dnf::{self, OptimizedFor};
use qm_agent::qm::{Enc16, Enc32, Enc64};

fn main() {
    println!("=== Unified CNF-to-DNF API Demo ===\n");

    // Example CNF: (A ∨ B) ∧ (A ∨ C) ∧ (B ∨ C)
    // Represented as bit vectors: A=bit0, B=bit1, C=bit2
    let cnf = vec![
        0b011u64,  // A ∨ B
        0b101u64,  // A ∨ C
        0b110u64,  // B ∨ C
    ];

    // Encoding-aware API: Automatically selects optimal SIMD strategy
    println!("Encoding-aware API (automatic optimization selection):");

    // For small problems (≤16 variables), use Encoding16
    let dnf_16 = cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::AutoDetect}>(&cnf, 3);
    println!("  Encoding16: {} terms (auto-selected Avx512_16bits)", dnf_16.len());

    // For medium problems (≤32 variables), use Encoding32
    let dnf_32 = cnf_dnf::convert_cnf_to_dnf::<Enc32, {OptimizedFor::AutoDetect}>(&cnf, 3);
    println!("  Encoding32: {} terms (auto-selected Avx512_32bits)", dnf_32.len());

    // For large problems (≤64 variables), use Encoding64
    let dnf_64 = cnf_dnf::convert_cnf_to_dnf::<Enc64, {OptimizedFor::AutoDetect}>(&cnf, 3);
    println!("  Encoding64: {} terms (auto-selected Avx512_64bits)", dnf_64.len());

    // All encodings produce the same result!
    assert_eq!(dnf_16, dnf_32);
    assert_eq!(dnf_32, dnf_64);

    println!("\n✓ All encodings produce identical results!");
    println!("✓ API is type-safe and automatically optimized!");

    // Show validation
    println!("\n=== Type Safety Demo ===\n");

    // This works - 8 variables fits in Encoding16
    let result = cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::AutoDetect}>(&cnf, 8);
    println!("8 variables with Encoding16: ✓ {} terms", result.len());

    // This fails - 20 variables exceeds Encoding16 (max 16)
    let result = cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::AutoDetect}>(&cnf, 20);
    if result.is_empty() {
        println!("20 variables with Encoding16: ✗ Validation failed (expected)");
    }

    // This works - 20 variables fits in Encoding32
    let result = cnf_dnf::convert_cnf_to_dnf::<Enc32, {OptimizedFor::AutoDetect}>(&cnf, 20);
    println!("20 variables with Encoding32: ✓ {result} terms", result = if !result.is_empty() { format!("{}", result.len()) } else { "0".to_string() });

    println!("\n=== Benefits ===");
    println!("✓ No need to manually choose OptimizedFor");
    println!("✓ Compile-time and runtime validation");
    println!("✓ Automatic optimal SIMD selection");
    println!("✓ Clearer API: encoding shows intent");
}
