// Very hard CNF problem found when generating popcnt_6_3
// 35 conjunctions with 60 variables

use std::time::Instant;
use qm_agent::cnf_dnf::{self, OptimizedFor};
use qm_agent::qm::Enc64;

fn main() {
    const N_BITS: usize = 64;

    let cnf: Vec<u64> = vec![
        1u64 << 0 | 1u64 << 1 | 1u64 << 2 | 1u64 << 3,
        1u64 << 4 | 1u64 << 5 | 1u64 << 6 | 1u64 << 7,
        1u64 << 3 | 1u64 << 7 | 1u64 << 11,
        1u64 << 8 | 1u64 << 9 | 1u64 << 10 | 1u64 << 11,
        1u64 << 12 | 1u64 << 13 | 1u64 << 14 | 1u64 << 15,
        1u64 << 2 | 1u64 << 15 | 1u64 << 19,
        1u64 << 16 | 1u64 << 17 | 1u64 << 18 | 1u64 << 19,
        1u64 << 10 | 1u64 << 18 | 1u64 << 22,
        1u64 << 6 | 1u64 << 14 | 1u64 << 23,
        1u64 << 20 | 1u64 << 21 | 1u64 << 22 | 1u64 << 23,
        1u64 << 24 | 1u64 << 25 | 1u64 << 26 | 1u64 << 27,
        1u64 << 1 | 1u64 << 27 | 1u64 << 31,
        1u64 << 28 | 1u64 << 29 | 1u64 << 30 | 1u64 << 31,
        1u64 << 9 | 1u64 << 30 | 1u64 << 34,
        1u64 << 5 | 1u64 << 26 | 1u64 << 35,
        1u64 << 32 | 1u64 << 33 | 1u64 << 34 | 1u64 << 35,
        1u64 << 21 | 1u64 << 33 | 1u64 << 37,
        1u64 << 17 | 1u64 << 29 | 1u64 << 38,
        1u64 << 13 | 1u64 << 25 | 1u64 << 39,
        1u64 << 36 | 1u64 << 37 | 1u64 << 38 | 1u64 << 39,
        1u64 << 40 | 1u64 << 41 | 1u64 << 42 | 1u64 << 43,
        1u64 << 0 | 1u64 << 43 | 1u64 << 47,
        1u64 << 44 | 1u64 << 45 | 1u64 << 46 | 1u64 << 47,
        1u64 << 8 | 1u64 << 46 | 1u64 << 50,
        1u64 << 4 | 1u64 << 42 | 1u64 << 51,
        1u64 << 48 | 1u64 << 49 | 1u64 << 50 | 1u64 << 51,
        1u64 << 20 | 1u64 << 49 | 1u64 << 53,
        1u64 << 16 | 1u64 << 45 | 1u64 << 54,
        1u64 << 12 | 1u64 << 41 | 1u64 << 55,
        1u64 << 52 | 1u64 << 53 | 1u64 << 54 | 1u64 << 55,
        1u64 << 36 | 1u64 << 52 | 1u64 << 56,
        1u64 << 32 | 1u64 << 48 | 1u64 << 57,
        1u64 << 28 | 1u64 << 44 | 1u64 << 58,
        1u64 << 24 | 1u64 << 40 | 1u64 << 59,
        1u64 << 56 | 1u64 << 57 | 1u64 << 58 | 1u64 << 59,
    ];
    
    println!("Computing minimal DNF (this may take a while)...");
    println!("CNF = {}", cnf_dnf::cnf_to_string(&cnf));

    {
        let start = Instant::now();
        let dnf = cnf_dnf::cnf_to_dnf_minimal::<Enc64>(&cnf, N_BITS, OptimizedFor::Avx512_64bits ).unwrap();
        println!("Runtime: Enc64,Avx512_64bits: {:?}", start.elapsed());
        println!("DNF size = {}", dnf.len());
    }
}
