// Random CNF generation test: 500 conjunctions, 16 bits, 8 disjunctions per conjunction

use qm_agent::cnf_dnf::{self, OptimizedFor};
use qm_agent::qm::Enc16;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::time::Instant;

fn main() {
    let mut rng = StdRng::seed_from_u64(42);
    const N_BITS: usize = 16;
    const N_CONJUNCTIONS: usize = 500;
    const N_DISJUNCTIONS: usize = 8;

    let mut cnf: Vec<u64> = Vec::new();
    for _ in 0..N_CONJUNCTIONS {
        let mut conjunction = 0u64;
        for _ in 0..N_DISJUNCTIONS {
            let r = rng.random_range(0..N_BITS);
            conjunction |= 1u64 << r;
        }
        cnf.push(conjunction);
    }

    println!("CNF = {}", cnf_dnf::cnf_to_string(&cnf));
    {
        let start = Instant::now();
        let dnf = cnf_dnf::cnf_to_dnf::<Enc16>(&cnf, N_BITS, OptimizedFor::X64 ).unwrap();
        println!("Runtime: Enc16,X64: {:?}", start.elapsed());
        println!("DNF size = {}", dnf.len());
    }
    {
        let start = Instant::now();
        let dnf = cnf_dnf::cnf_to_dnf::<Enc16>(&cnf, N_BITS, OptimizedFor::Avx512_16bits ).unwrap();
        println!("Runtime: Enc16,Avx512_16bits: {:?}", start.elapsed());
        println!("DNF size = {}", dnf.len());
    }
    {
        let start = Instant::now();
        let dnf = cnf_dnf::cnf_to_dnf::<Enc16>(&cnf, N_BITS, OptimizedFor::Avx512_32bits ).unwrap();
        println!("Runtime: Enc16,Avx512_32bits: {:?}", start.elapsed());
        println!("DNF size = {}", dnf.len());
    }
    {
        let start = Instant::now();
        let dnf = cnf_dnf::cnf_to_dnf::<Enc16>(&cnf, N_BITS, OptimizedFor::Avx512_64bits ).unwrap();
        println!("Runtime: Enc16,Avx512_64bits: {:?}", start.elapsed());
        println!("DNF size = {}", dnf.len());
    }
}
