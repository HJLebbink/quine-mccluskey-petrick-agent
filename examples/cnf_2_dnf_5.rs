// Random CNF generation test with minimal DNF: 10 conjunctions, 64 bits, 8 disjunctions per conjunction

use qm_agent::cnf_dnf::{self, OptimizedFor};
use rand::{rngs::StdRng, Rng, SeedableRng};

fn main() {
    let mut rng = StdRng::seed_from_u64(42);
    const N_BITS: usize = 64;
    const N_CONJUNCTIONS: usize = 10;
    const N_DISJUNCTIONS: usize = 8;
    const EARLY_PRUNE: bool = true;

    let mut cnf: Vec<u64> = Vec::new();
    for _ in 0..N_CONJUNCTIONS {
        let mut conjunction = 0u64;
        for _ in 0..N_DISJUNCTIONS {
            let r = rng.gen_range(0..N_BITS);
            conjunction |= 1u64 << r;
        }
        cnf.push(conjunction);
    }

    println!("CNF = {}", cnf_dnf::cnf_to_string(&cnf));

    let min_dnf = cnf_dnf::convert_cnf_to_dnf_minimal(&cnf, N_BITS, OptimizedFor::X64, EARLY_PRUNE);

    println!("DNF_min = {}", cnf_dnf::dnf_to_string(&min_dnf));
}
