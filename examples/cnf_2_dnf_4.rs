// Random CNF generation test: 20 conjunctions, 32 bits, 8 disjunctions per conjunction

use qm_agent::cnf_dnf::{self, OptimizedFor};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::time::Instant;

fn main() {
    let mut rng = StdRng::seed_from_u64(42);
    const N_BITS: usize = 32;
    const N_CONJUNCTIONS: usize = 20;
    const N_DISJUNCTIONS: usize = 8;

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

    let start = Instant::now();
    let dnf = cnf_dnf::convert_cnf_to_dnf(&cnf, N_BITS, OptimizedFor::X64);
    let duration = start.elapsed();

    println!("DNF = {}", cnf_dnf::dnf_to_string(&dnf));
    println!("Runtime: {:?}", duration);
}
