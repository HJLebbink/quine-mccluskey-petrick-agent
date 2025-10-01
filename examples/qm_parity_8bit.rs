// 8-bit Parity Example
//
// Generates a truth table for parity checking of two 4-bit numbers.
// Output is 1 when the sum of the two 4-bit numbers has odd parity.
//
// This creates a large truth table (256 entries) and demonstrates
// the algorithm's ability to handle complex Boolean functions.

use qm_agent::QMSolver;

fn main() {
    println!("=== Quine-McCluskey Example: 8-bit Parity Function ===\n");
    println!("Function: parity(A + B) where A and B are 4-bit numbers");
    println!("Output is 1 when popcount(A + B) is odd");
    println!();

    let variables = 8;
    let mut minterms = Vec::new();

    // Generate truth table
    for i1 in 0..=0b1111u32 {
        for i2 in 0..=0b1111u32 {
            let input = (i1 << 4) | i2;
            let sum = i1 + i2;
            let parity = sum.count_ones() & 1;

            if parity == 1 {
                minterms.push(input);
            }
        }
    }

    println!("Generated {} minterms from 256 possible inputs", minterms.len());
    println!("Number of variables: {}", variables);
    println!();

    let mut solver = QMSolver::new(variables);
    solver.set_minterms(&minterms);

    println!("Computing minimization (this may take a moment)...");
    let result = solver.solve();

    println!();
    println!("Prime Implicants: {}", result.prime_implicants.len());
    println!("Essential Prime Implicants: {}", result.essential_prime_implicants.len());
    println!();

    println!("First 10 prime implicants:");
    for (i, pi) in result.prime_implicants.iter().take(10).enumerate() {
        println!("  {}: {}", i, pi);
    }
    if result.prime_implicants.len() > 10 {
        println!("  ... and {} more", result.prime_implicants.len() - 10);
    }
    println!();

    println!("Minimized Expression has {} terms",
             result.minimized_expression.matches('+').count() + 1);

    // Don't print full expression as it's very long
    println!("(Expression too large to display fully)");
    println!();

    println!("This demonstrates QM algorithm on complex 8-bit functions");
}
