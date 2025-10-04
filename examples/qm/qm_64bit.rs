// Example demonstrating 64-bit minterm encoding with u128
// This allows working with up to 64 Boolean variables

use qm_agent::qm::random::generate_random_minterms;
use qm_agent::qm::{reduce_minterms, Enc64};

fn main() {

    // Example with 64-bit encoding using const parameter
    println!("\n\n64-bit Encoding Example with const parameter");
    println!("============================================\n");

    const NUM_VARIABLES: usize = 33;
    const NUM_MINTERMS: usize = 500000;
    const SEED: u64 = 42;

    println!("Generating random minterms with {NUM_VARIABLES} variables");
    let minterms: Vec<u128> = generate_random_minterms(NUM_VARIABLES, NUM_MINTERMS, SEED);

    println!("Generated {NUM_MINTERMS} random minterms");
    println!();

    println!("Reducing minterms with Encoding64...");
    let start_large = std::time::Instant::now();
    let reduced = reduce_minterms::<Enc64>(&minterms, false);
    let elapsed_large = start_large.elapsed();

    println!("Reduction completed in {elapsed_large:?}");
    println!("Reduced from {} to {} terms", minterms.len(), reduced.len());
    println!();
}
