// Krook dataset from QCA (Qualitative Comparative Analysis)
//
// 5-bit input: ES QU WS WM LP
// This is a real-world dataset used in social science research
//
// Input variables:
// - ES: Economic Strength
// - QU: Quality
// - WS: Workforce Skills
// - WM: Workforce Motivation
// - LP: Labor Productivity

use qm_agent::QMSolver;

fn main() {
    println!("=== Quine-McCluskey Example: Krook QCA Dataset ===\n");
    println!("5 variables: ES QU WS WM LP");
    println!("(Economic Strength, Quality, Workforce Skills, Workforce Motivation, Labor Productivity)");
    println!();

    // Binary representation: ES QU WS WM LP
    let minterms = vec![
        0b00011, // 3
        0b01011, // 11
        0b10100, // 20
        0b10111, // 23
        0b11001, // 25
        0b11010, // 26
        0b11011, // 27
        0b11100, // 28
        0b11111, // 31
    ];

    let variables = 5;

    println!("Input minterms: {:?}", minterms);
    println!("Decimal: [3, 11, 20, 23, 25, 26, 27, 28, 31]");
    println!("Number of variables: {}", variables);
    println!();

    println!("Truth table:");
    println!("  ES QU WS WM LP | Output");
    println!("  ---------------|-------");
    for i in 0..32 {
        let output = if minterms.contains(&i) { "1" } else { "0" };
        println!("  {:05b}          |   {}", i, output);
    }
    println!();

    let mut solver = QMSolver::new(variables);
    solver.set_minterms(&minterms);

    let result = solver.solve();

    println!("Prime Implicants ({}):", result.prime_implicants.len());
    for (i, pi) in result.prime_implicants.iter().enumerate() {
        println!("  {}: {}", i, pi);
    }
    println!();

    println!("Essential Prime Implicants ({}):", result.essential_prime_implicants.len());
    for (i, epi) in result.essential_prime_implicants.iter().enumerate() {
        println!("  {}: {}", i, epi);
    }
    println!();

    println!("Minimized Boolean Expression:");
    println!("  {}", result.minimized_expression);
    println!();

    println!("This dataset is from QCA (Qualitative Comparative Analysis)");
    println!("Used in social science research for analyzing causal complexity");
}
