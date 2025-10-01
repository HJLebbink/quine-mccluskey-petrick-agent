// 12-bit Random Example
//
// Generates random minterms with a fixed seed for reproducibility
// Tests the algorithm on arbitrary Boolean functions
//
// This example uses 550 random minterms (25 * 22) from 4096 possible inputs

use qm_agent::QMSolver;
use rand::{rngs::StdRng, Rng, SeedableRng};

fn main() {
    println!("=== Quine-McCluskey Example: 12-bit Random Function ===\n");

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
}
