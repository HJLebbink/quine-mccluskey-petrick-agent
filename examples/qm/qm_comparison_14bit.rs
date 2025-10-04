// 14-bit Comparison Example
//
// Creates a Boolean function that outputs 1 when first 7-bit number > second 7-bit number
// This is a large example with 16384 possible inputs
//
// WARNING: This example takes significant time to run (10-15 seconds or more)
// It demonstrates the algorithm's capability on large practical problems

use qm_agent::{QMSolver, Enc32};
use std::collections::HashSet;

fn main() {
    println!("=== Quine-McCluskey Example: 14-bit Comparison (i1 > i2) ===\n");
    println!("Function: output = 1 when (7-bit i1) > (7-bit i2)");
    println!("WARNING: This is a large example and will take 10-15+ seconds");
    println!();

    let variables = 14;
    let mut minterms = Vec::new();
    let mut seen = HashSet::new();

    // Generate truth table: compare two 7-bit numbers
    for i1 in 0..=0b0111_1111 {
        for i2 in 0..=0b0111_1111 {
            let input = (i1 << 7) | i2;
            let output = if i1 > i2 { 1 } else { 0 };

            if output == 1 && !seen.contains(&input) {
                minterms.push(input);
                seen.insert(input);
            }
        }
    }

    println!("Generated {} minterms from 16384 possible inputs", minterms.len());
    println!("Number of variables: {}", variables);
    println!();

    let mut solver = QMSolver::<Enc32>::new(variables);
    let minterms_u64: Vec<u64> = minterms.iter().map(|&x| x as u64).collect();
    solver.set_minterms(&minterms_u64);

    println!("Starting minimization...");
    println!("(This will take approximately 10-15 seconds)");

    let start = std::time::Instant::now();
    let result = solver.solve();
    let elapsed = start.elapsed();

    println!();
    println!("Minimization completed in {:.2?}", elapsed);
    println!();

    println!("Results:");
    println!("  Prime Implicants: {}", result.prime_implicants.len());
    println!("  Essential Prime Implicants: {}", result.essential_prime_implicants.len());
    println!("  Expression terms: {}", result.minimized_expression.matches('+').count() + 1);
    println!();

    println!("First 10 prime implicants:");
    for (i, pi) in result.prime_implicants.iter().take(10).enumerate() {
        println!("  {}: {}", i, pi);
    }
    if result.prime_implicants.len() > 10 {
        println!("  ... and {} more", result.prime_implicants.len() - 10);
    }
    println!();

    println!("This demonstrates QM algorithm on large practical comparison circuits");
}
