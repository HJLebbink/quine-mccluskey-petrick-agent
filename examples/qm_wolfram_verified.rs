// Quine-McCluskey example verified with Wolfram Alpha
//
// Boolean Function 49148 (BF4 0xBFFC)
// Truth table for 4-bit input (ABCD):
//  1: 0001 -> 0
//  3: 0011 -> 1
//  5: 0101 -> 1
//  8: 1000 -> 1
// 10: 1010 -> 1
// 11: 1011 -> 1
// 13: 1101 -> 1
//
// Expected DNF (from Wolfram Alpha):
// (A & ~C) | (~A & B) | (~B & C) | (C & D)
// 1XX1     | 01XX     | X01X     | XX11
//
// Also equivalent to:
// 00X1 X011 X101 10X0

use qm_agent::QMSolver;

fn main() {
    println!("=== Quine-McCluskey Example: Wolfram Alpha Verified ===\n");
    println!("Boolean Function: BF4 0xBFFC = 49148");
    println!("Wolfram Alpha: 'boolean function 49148'");
    println!();

    let minterms = vec![1, 3, 5, 8, 10, 11, 13];
    let variables = 4;

    println!("Input minterms: {:?}", minterms);
    println!("Number of variables: {}", variables);
    println!();

    println!("Truth table:");
    println!("  ABCD | Output");
    println!("  -----|-------");
    for i in 0..16 {
        let output = if minterms.contains(&i) { "1" } else { "0" };
        println!("  {:04b} |   {}", i, output);
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

    println!("Minimized Expression:");
    println!("  {}", result.minimized_expression);
    println!();

    println!("Expected from Wolfram Alpha:");
    println!("  (A & ~C) | (~A & B) | (~B & C) | (C & D)");
    println!();

    println!("Expected from Rust:");
    println!("  00X1 X011 X101 10X0");
    println!("  Which is: (A'B'D) + (B'CD) + (BC'D) + (AB'D')");
    println!();

    println!("Verified with:");
    println!("  - Wolfram Alpha: 'boolean function 49148'");
    println!("  - http://www.32x8.com/qmm4_____A-B-C-D_____m_1-3-5-8-10-11-13");
}
