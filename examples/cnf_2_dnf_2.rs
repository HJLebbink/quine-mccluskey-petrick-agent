// CNF =  (A|B) & (A|C) & (B|E) & (C|D) & (D|F) & (E|F)
// DNF =  (A&B&D&F) | (A&C&E&F) | (A&D&E) | (B&C&D&E) | (B&C&F)

use qm_agent::cnf_dnf::{self, OptimizedFor};
use std::time::Instant;

fn main() {
    let cnf = vec![
        vec!["A".to_string(), "B".to_string()],
        vec!["C".to_string(), "D".to_string()],
        vec!["A".to_string(), "C".to_string()],
        vec!["E".to_string(), "F".to_string()],
        vec!["B".to_string(), "E".to_string()],
        vec!["D".to_string(), "F".to_string()],
    ];

    println!("CNF = {:?}", cnf);

    let start = Instant::now();
    let dnf = cnf_dnf::convert_cnf_to_dnf_with_names(&cnf, OptimizedFor::X64);
    let duration = start.elapsed();

    println!("DNF = {:?}", dnf);
    println!("Runtime: {:?}", duration);
}
