// CNF =  (1|2) & (3|4)
// DNF =  (1&3) | (2&3) | (1&4) | (2&4)

use qm_agent::cnf_dnf::{self, OptimizedFor};
use qm_agent::qm::Enc16;
use std::time::Instant;

fn main() {

    const N_BITS: usize = 8;


    let cnf: Vec<u64> = vec![
        (1 << 1) | (1 << 2),
        (1 << 3) | (1 << 4),
    ];

    println!("observed CNF: {}", cnf_dnf::cnf_to_string(&cnf));
    println!("expected CNF: (1|2) & (3|4)");

    println!("Using Encoding16 (auto-selects optimal SIMD strategy)");

    let start = Instant::now();
    let dnf = cnf_dnf::cnf_to_dnf::<Enc16>(&cnf, N_BITS, OptimizedFor::AutoDetect).unwrap();
    let duration = start.elapsed();

    println!("observed DNF: {}", cnf_dnf::dnf_to_string(&dnf));
    println!("expected DNF: (1&3) | (2&3) | (1&4) | (2&4)");
    println!("Runtime: {duration:?}");
}
