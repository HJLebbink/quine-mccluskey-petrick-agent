// CNF =  (1|2) & (3|4)
// DNF =  (1&3) | (2&3) | (1&4) | (2&4)

use qm_agent::cnf_dnf::{self, OptimizedFor};

fn main() {
    let cnf: Vec<u64> = vec![
        (1 << 1) | (1 << 2),
        (1 << 3) | (1 << 4),
    ];

    println!("observed CNF: {}", cnf_dnf::cnf_to_string(&cnf));
    println!("expected CNF: (1|2) & (3|4)");

    let dnf = cnf_dnf::convert_cnf_to_dnf(&cnf, 8, OptimizedFor::X64);

    println!("observed DNF: {}", cnf_dnf::dnf_to_string(&dnf));
    println!("expected DNF: (1&3) | (2&3) | (1&4) | (2&4)");
}
