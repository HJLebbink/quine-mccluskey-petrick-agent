// Example that needs Petrick's method (no primary essential prime implicants)
//
// Truth table for 4-bit input (ABCD):
//  0: 0000 -> 1
//  2: 0010 -> 1
//  3: 0011 -> 1
//  4: 0100 -> 1
//  5: 0101 -> 1
//  6: 0110 -> 1
//  7: 0111 -> 1
//  8: 1000 -> 1
//  9: 1001 -> 1
// 10: 1010 -> 1
// 11: 1011 -> 1
// 12: 1100 -> 1
// 13: 1101 -> 1
//
// Expected result: 0XX0 0X1X X10X 10XX
// Which means: (A'D') + (A'C) + (BC') + (AB')

use qm_agent::{QMSolver, Enc32};

fn main() {
    println!("=== Quine-McCluskey Example: Needs Petrick's Method ===\n");

    let minterms = vec![0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
    let variables = 4;

    println!("Input minterms: {:?}", minterms);
    println!("Number of variables: {}", variables);
    println!();

    let mut solver = QMSolver::<Enc32>::new(variables);
    let minterms_u64: Vec<u64> = minterms.iter().map(|&x| x as u64).collect();
    solver.set_minterms(&minterms_u64);

    let result = solver.solve();

    println!("Prime Implicants:");
    for (i, pi) in result.prime_implicants.iter().enumerate() {
        println!("  {}: {}", i, pi);
    }
    println!();

    println!("Essential Prime Implicants:");
    for (i, epi) in result.essential_prime_implicants.iter().enumerate() {
        println!("  {}: {}", i, epi);
    }
    println!();

    println!("Minimized Expression:");
    println!("  {}", result.minimized_expression);
    println!();

    println!("Expected (from Rust): 0XX0 0X1X X10X 10XX");
    println!("Which translates to: (A'D') + (A'C) + (BC') + (AB')");
    println!();

    println!("Solution steps:");
    for (i, step) in result.solution_steps.iter().enumerate() {
        println!("  Step {}: {}", i + 1, step);
    }
}
