// 14-bit Comparison Example
//
// Creates a Boolean function that outputs 1 when first 7-bit number > second 7-bit number
// This is a large example with 16384 possible inputs
//
// WARNING: This example takes significant time to run (10-15 seconds or more)
// It demonstrates the algorithm's capability on large practical problems

use qm_agent::qm::BitOps;
use qm_agent::{Enc16, Enc32, Enc64, MintermEncoding, QMSolver};
use std::collections::HashSet;

fn main() {
    println!("=== Quine-McCluskey Example: 14-bit Comparison (i1 > i2) ===\n");
    println!("Function: output = 1 when (7-bit i1) > (7-bit i2)");

    let logging_on = true;

    if true {
        let start = std::time::Instant::now();
        main_inner::<Enc16>(logging_on);
        println!("Minimization Enc16 completed in {:.2?}", start.elapsed());
    }
    if false {
        let start = std::time::Instant::now();
        main_inner::<Enc32>(logging_on);
        println!("Minimization Enc32 completed in {:.2?}", start.elapsed());
    }
    if false {
        let start = std::time::Instant::now();
        main_inner::<Enc64>(logging_on);
        println!("Minimization Enc64 completed in {:.2?}", start.elapsed());
    }
}

fn main_inner<E: MintermEncoding>(logging_on: bool) {
    const N_VARIABLES: usize = 14;

    let mut minterms: Vec<E::Value> = Vec::new();
    let mut seen = HashSet::new();
    let mut n_inputs = 0;

    // Generate truth table: compare two 7-bit numbers; just some inefficient code
    for i1 in 0..=0b0111_1111 {
        for i2 in 0..=0b0111_1111 {
            n_inputs += 1;
            let input: E::Value = (E::Value::from_u64(i1) << 7usize) | E::Value::from_u64(i2);
            let output: bool = i1 > i2;

            if output && !seen.contains(&input) {
                minterms.push(input);
                seen.insert(input);
            }
        }
    }

    if logging_on {
        println!("==========================");
        println!("Number of variables: {N_VARIABLES}; Generated {} minterms from {n_inputs} possible inputs", minterms.len());
        println!();
    }

    let mut solver = QMSolver::<E>::new(N_VARIABLES);
    solver.set_logging(logging_on);
    solver.set_minterms(minterms);

    if logging_on {
        println!("Starting minimization...");
    }
    let result = solver.solve();

    if logging_on {
        println!("Results:");
        println!("  Prime Implicants: {}", result.prime_implicants.len());
        println!("  Essential Prime Implicants: {}", result.essential_prime_implicants.len());
        println!("  Expression terms: {}", result.minimized_expression.matches('+').count() + 1);
        println!();

        println!("First 10 prime implicants:");
        for (i, pi) in result.prime_implicants.iter().take(10).enumerate() {
            println!("  {i}: {pi}");
        }
        if result.prime_implicants.len() > 10 {
            println!("  ... and {} more", result.prime_implicants.len() - 10);
        }
    }
}