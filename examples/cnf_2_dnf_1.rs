// CNF =  (1|2) & (1|3) & (3|4) & (2|5) & (4|6) & (5|6)
// DNF =  (1&4&5) | (2&3&4&5) | (2&3&6) | (1&2&4&6) | (1&3&5&6)
//
// Answer according to wolfram:
// abdf acef ade bcde bcf
// 145 1246 1356 2345 236

use qm_agent::cnf_dnf::{self, OptimizedFor};
use qm_agent::qm::Enc16;
use std::time::Instant;

fn main() {
    let cnf: Vec<u64> = vec![
        (1 << 1) | (1 << 2),
        (1 << 3) | (1 << 4),
        (1 << 1) | (1 << 3),
        (1 << 5) | (1 << 6),
        (1 << 2) | (1 << 5),
        (1 << 4) | (1 << 6),
    ];

    println!("CNF = {}", cnf_dnf::cnf_to_string(&cnf));

    let start = Instant::now();
    let dnf = cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::AutoDetect}>(&cnf, 8);
    let duration = start.elapsed();

    println!("DNF = {}", cnf_dnf::dnf_to_string(&dnf));
    println!("Runtime: {duration:?}");
}
