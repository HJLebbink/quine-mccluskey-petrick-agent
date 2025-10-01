// Random CNF generation test: 500 conjunctions, 16 bits, 8 disjunctions per conjunction

use qm_agent::cnf_dnf::{self, OptimizedFor};
use rand::{rngs::StdRng, Rng, SeedableRng};

fn main() {
    let mut rng = StdRng::seed_from_u64(42);
    const N_BITS: usize = 16;
    const N_CONJUNCTIONS: usize = 500;
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

    let dnf = cnf_dnf::convert_cnf_to_dnf(&cnf, N_BITS, OptimizedFor::X64);

    println!("DNF size = {}", dnf.len());
    // Note: Full DNF not printed due to size
}
