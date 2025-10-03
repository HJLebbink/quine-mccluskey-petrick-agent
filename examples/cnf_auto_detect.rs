// Example demonstrating encoding-aware API for CNF to DNF conversion
//
// This example shows how to use the encoding-aware API which automatically
// selects the optimal SIMD strategy based on the encoding type.

use qm_agent::cnf_dnf::{self, OptimizedFor};
use qm_agent::qm::{Enc16, Enc32, Enc64};

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

    println!("NEW ENCODING-AWARE API:");
    println!("The encoding type automatically selects the optimal SIMD strategy!");
    println!();

    // Convert using Encoding16 (for problems with ≤16 variables)
    let dnf_16 = cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::AutoDetect}>(&cnf, n_variables);
    println!("Encoding16: {} terms (auto-selected Avx512_16bits)", dnf_16.len());

    // Convert using Encoding32 (for problems with ≤32 variables)
    let dnf_32 = cnf_dnf::convert_cnf_to_dnf::<Enc32, {OptimizedFor::AutoDetect}>(&cnf, n_variables);
    println!("Encoding32: {} terms (auto-selected Avx512_32bits)", dnf_32.len());

    // Convert using Encoding64 (for problems with ≤64 variables)
    let dnf_64 = cnf_dnf::convert_cnf_to_dnf::<Enc64, {OptimizedFor::AutoDetect}>(&cnf, n_variables);
    println!("Encoding64: {} terms (auto-selected Avx512_64bits)", dnf_64.len());
    println!();

    println!("DNF result (minterms):");
    for term in &dnf_64 {
        print!("  ");
        for bit in 0..n_variables {
            if (term >> bit) & 1 == 1 {
                print!("{}", ('A' as u8 + bit as u8) as char);
            }
        }
        println!();
    }
    println!();

    println!("Benefits of the encoding-aware API:");
    println!("  ✓ No need to manually select optimization level");
    println!("  ✓ Type-safe: encoding validates variable count");
    println!("  ✓ Automatically uses best SIMD strategy for encoding");
    println!("  ✓ Simpler API with fewer parameters");
    println!();

    println!("Note: All encodings produce identical results for compatible variable counts.");
    println!("The difference is in execution speed and maximum variable count supported.");
}
