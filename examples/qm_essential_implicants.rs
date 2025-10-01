// Example with primary essential prime implicants
// (Petrick's method is not needed)
//
// Truth table for 4-bit input (ABCD):
//  0: 0000 -> 1
//  2: 0010 -> 1
//  5: 0101 -> 1
//  6: 0110 -> 1
//  7: 0111 -> 1
//  8: 1000 -> 1
// 10: 1010 -> 1
// 12: 1100 -> 1
// 13: 1101 -> 1
// 14: 1110 -> 1
// 15: 1111 -> 1
//
// Expected result: X0X0 X1X1 XX10 1XX0
// Which means: (B'D') + (BD) + (CD') + (AD')

use qm_agent::QMSolver;

fn main() {
    println!("=== Quine-McCluskey Example: Has Essential Prime Implicants ===\n");

    let minterms = vec![0, 2, 5, 6, 7, 8, 10, 12, 13, 14, 15];
    let variables = 4;

    println!("Input minterms: {:?}", minterms);
    println!("Number of variables: {}", variables);
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

    println!("Expected (from Rust): X0X0 X1X1 XX10 1XX0");
    println!("Which translates to: (B'D') + (BD) + (CD') + (AD')");
    println!();

    println!("Verified with:");
    println!("  - https://www.mathematik.uni-marburg.de/~thormae/lectures/ti1/code/qmc/");
    println!("  - http://www.32x8.com/var4.html");
    println!("  - https://ictlab.kz/extra/Kmap/");
}
