// Simple 3-bit Quine-McCluskey example
//
// Truth table for 3-bit input (ABC):
//  0: 000 -> 0
//  1: 001 -> 0
//  2: 010 -> 0
//  3: 011 -> 1
//  4: 100 -> 1
//  5: 101 -> 1
//  6: 110 -> 1
//  7: 111 -> 1
//
// This represents the Boolean function: f(A,B,C) = Σ(3,4,5,6,7)
// Output is 1 when the decimal value > 2

use qm_agent::QMSolver;

fn main() {
    println!("=== Simple 3-bit Quine-McCluskey Example ===\n");

    println!("Boolean function: output = 1 when input > 2");
    println!();

    let minterms = vec![3, 4, 5, 6, 7];
    let variables = 3;

    println!("Input minterms: {:?}", minterms);
    println!("Number of variables: {}", variables);
    println!();

    println!("Truth table:");
    println!("  ABC | Output");
    println!("  ----|-------");
    for i in 0..8 {
        let output = if minterms.contains(&i) { "1" } else { "0" };
        println!("  {:03b} |   {}", i, output);
    }
    println!();

    let mut solver = QMSolver::new(variables);
    solver.set_minterms(&minterms);

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

    println!("Minimized Boolean Expression:");
    println!("  {}", result.minimized_expression);
    println!();

    println!("Solution process:");
    for (i, step) in result.solution_steps.iter().enumerate() {
        println!("  Step {}: {}", i + 1, step);
    }
}
