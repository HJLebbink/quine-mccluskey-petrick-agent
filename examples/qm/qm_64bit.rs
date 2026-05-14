// Example demonstrating 64-bit minterm encoding with u128
// This allows working with up to 64 Boolean variables

use qm_agent::qm::random::generate_random_minterms;
use qm_agent::{QMSolver, Enc64};

fn main() {

    // Example with 64-bit encoding using const parameter
    println!("\n\n64-bit Encoding Example with const parameter");
    println!("============================================\n");

    const NUM_VARIABLES: usize = 33;
    const NUM_MINTERMS: usize = 70000;
    const SEED: u64 = 42;

    println!("Generating random minterms with {NUM_VARIABLES} variables");
    let minterms: Vec<u128> = generate_random_minterms(NUM_VARIABLES, NUM_MINTERMS, SEED);

    println!("Generated {NUM_MINTERMS} random minterms");
    println!("Solving with QMSolver...");
    let start = std::time::Instant::now();
    
    println!("Reducing minterms with Encoding64...");
    let mut solver = QMSolver::<Enc64>::new(NUM_VARIABLES);
    solver.set_logging(true);
    solver.set_method(qm_agent::qm::SolveMethod::QM);
    solver.set_minterms(minterms);
    
    let result = solver.solve();
    
    println!("Solved in {:?}", start.elapsed());
    println!("\nnumber of Prime Implicants: {}",  result.prime_implicants.len());
    println!("\nnumber of Essential Prime Implicants: {}", result.essential_prime_implicants.len());
}
