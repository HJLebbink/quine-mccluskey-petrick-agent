// Example demonstrating 64-bit minterm encoding with u128
// This allows working with up to 64 Boolean variables

use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use qm_agent::qm::{reduce_minterms, Encoding64};
use qm_agent::QMSolver;

fn main() {
    // Example with 40 variables (impossible with u64 encoding)
    println!("64-bit Encoding Example with 40 variables");
    println!("==========================================\n");

    const VARIABLES: usize = 12;
    const SEED: u64 = 42;
    const NUM_MINTERMS: usize = 25 * 22; // 550 minterms

    println!("Generating random Boolean function with seed {}", SEED);
    println!("Variables: {}", VARIABLES);
    println!("Random minterms: {}", NUM_MINTERMS);
    println!();

    let mut rng = StdRng::seed_from_u64(SEED);
    let mut minterms = Vec::new();

    // Generate random minterms
    for _ in 0..NUM_MINTERMS {
        let input = rng.gen_range(0..(1 << VARIABLES));
        if !minterms.contains(&input) {
            minterms.push(input);
        }
    }

    println!("Generated {} unique minterms from {} possible inputs",
             minterms.len(), 1 << VARIABLES);
    println!();

    let mut solver = QMSolver::new(VARIABLES);
    solver.set_minterms(&minterms);

    println!("Computing minimization...");
    let start = std::time::Instant::now();
    let result = solver.solve();
    let elapsed = start.elapsed();

    println!();
    println!("Minimization completed in {:.2?}", elapsed);
    println!();

    let initial_terms = minterms.len();
    let final_terms = result.prime_implicants.len();
    let reduction = initial_terms - final_terms;

    println!("Results:");
    println!("  Initial minterms: {}", initial_terms);
    println!("  Prime Implicants: {}", final_terms);
    println!("  Essential Prime Implicants: {}", result.essential_prime_implicants.len());
    println!("  Reduction: {} terms ({:.1}%)", reduction,
             (reduction as f64 / initial_terms as f64) * 100.0);
    println!();

    println!("First 20 prime implicants:");
    for (i, pi) in result.prime_implicants.iter().take(20).enumerate() {
        println!("  {}: {}", i, pi);
    }
    if result.prime_implicants.len() > 20 {
        println!("  ... and {} more", result.prime_implicants.len() - 20);
    }
    println!();

    println!("Minimized expression has {} terms",
             result.minimized_expression.matches('+').count() + 1);
    println!();

    println!("This demonstrates QM algorithm on random Boolean functions");
    println!("Seed {} ensures reproducible results", SEED);


    // Minterms using u128 to support larger variable counts
    let minterms: Vec<u128> = vec![
        0b00000000_00000000_00000000_00000001u128, // minterm 1
        0b00000000_00000000_00000000_00000011u128, // minterm 3
        0b00000000_00000000_00000000_00000111u128, // minterm 7
        0b00000000_00000000_00000000_00001111u128, // minterm 15
    ];

    println!("Input minterms (first 8 bits): {:?}",
        minterms.iter().map(|&m| (m & 0xFF) as u8).collect::<Vec<_>>());

    let result = reduce_minterms::<Encoding64>(&minterms, false);

    println!("\nReduced minterms count: {}", result.len());
    println!("First few reduced minterms:");
    for (i, &minterm) in result.iter().take(5).enumerate() {
        println!("  {}: 0x{:016X}", i, minterm);
    }

    // Smaller example with 8 variables for clarity
    println!("\n\n64-bit Encoding Example with 8 variables");
    println!("=========================================\n");

    let small_minterms: Vec<u128> = vec![1, 3, 7, 15];

    println!("Input minterms: {:?}", small_minterms);

    let result = reduce_minterms::<Encoding64>(&small_minterms, false);

    println!("Reduced minterms: {:?}", result);
    println!("Count: {}", result.len());
}
