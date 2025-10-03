//! Check which SIMD optimization levels are supported on the current hardware
//!
//! This example demonstrates the `is_supported()` method to query hardware capabilities.

use qm_agent::cnf_dnf::OptimizedFor;

fn main() {
    println!("=== SIMD Hardware Support Check ===\n");

    let optimizations = [
        OptimizedFor::X64,
        OptimizedFor::Avx2_64bits,
        OptimizedFor::Avx512_8bits,
        OptimizedFor::Avx512_16bits,
        OptimizedFor::Avx512_32bits,
        OptimizedFor::Avx512_64bits,
    ];

    for opt in &optimizations {
        let supported = if opt.is_supported() { "✓" } else { "✗" };
        println!("{} {:25} (max {} bits)", supported, opt.to_string(), opt.max_bits());
    }

    println!("\n=== Auto-Detection ===");
    let auto_8 = OptimizedFor::detect_best(8);
    let auto_16 = OptimizedFor::detect_best(16);
    let auto_32 = OptimizedFor::detect_best(32);
    let auto_64 = OptimizedFor::detect_best(64);

    println!("Best for  8 variables: {}", auto_8.to_string());
    println!("Best for 16 variables: {}", auto_16.to_string());
    println!("Best for 32 variables: {}", auto_32.to_string());
    println!("Best for 64 variables: {}", auto_64.to_string());

    println!("\n=== Conditional Execution Example ===");
    if OptimizedFor::Avx512_64bits.is_supported() {
        println!("You can use AVX-512 optimizations for maximum performance!");
    } else if OptimizedFor::Avx2_64bits.is_supported() {
        println!("AVX2 is available - good performance for large problems.");
    } else {
        println!("Using scalar X64 implementation - still fast, but no SIMD acceleration.");
    }
}
