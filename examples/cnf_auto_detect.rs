// Example demonstrating automatic hardware detection for CNF to DNF conversion
//
// This example shows how to use OptimizedFor::detect_best() to automatically
// select the best SIMD instruction set available on the current hardware.

use qm_agent::cnf_dnf::{self, OptimizedFor};

fn main() {
    // Define a CNF formula: (A + B) ∧ (A + C) ∧ (B + C)
    // CNF encoding: each bit represents a variable in a disjunction (OR clause)
    // Bit 0 = A, Bit 1 = B, Bit 2 = C
    let cnf: Vec<u64> = vec![
        0b011, // A + B
        0b101, // A + C
        0b110, // B + C
    ];
    let n_variables = 3;

    println!("CNF formula: (A + B) ∧ (A + C) ∧ (B + C)");
    println!();

    // Automatically detect the best optimization for this hardware
    let optimization = OptimizedFor::detect_best(n_variables);
    println!("Detected optimization: {}", optimization);
    println!("Maximum bits supported: {}", optimization.max_bits());
    println!();

    // Convert CNF to DNF using the auto-detected optimization
    let dnf = cnf_dnf::convert_cnf_to_dnf(&cnf, n_variables, optimization);

    println!("DNF result (minterms):");
    for term in &dnf {
        print!("  ");
        for bit in 0..n_variables {
            if (term >> bit) & 1 == 1 {
                print!("{}", ('A' as u8 + bit as u8) as char);
            }
        }
        println!();
    }
    println!();

    // Compare with explicit optimizations
    println!("Comparison with other optimization levels:");

    let optimizations = vec![
        OptimizedFor::X64,
        OptimizedFor::Avx2_64bits,
        OptimizedFor::Avx512_64bits,
        OptimizedFor::Avx512_32bits,
        OptimizedFor::Avx512_16bits,
        OptimizedFor::Avx512_8bits,
    ];

    for opt in optimizations {
        let dnf_test = cnf_dnf::convert_cnf_to_dnf(&cnf, n_variables, opt);
        println!("  {}: {} terms (max {} bits)",
                 opt, dnf_test.len(), opt.max_bits());
    }
    println!();

    println!("Note: All optimizations should produce the same result.");
    println!("The difference is in execution speed, not correctness.");
}
