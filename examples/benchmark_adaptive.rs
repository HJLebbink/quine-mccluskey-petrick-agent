use qm_agent::qm::primes::TruthTable;
use qm_agent::qm::primes_adaptive::find_prime_implicants_adaptive;
use std::time::Instant;

fn gen_sparse(n: u8, count: usize, seed: u64) -> Vec<u64> {
    let limit = 1u64 << n;
    let mut rng = seed;
    let mut taken = std::collections::HashSet::new();
    let mut result = Vec::with_capacity(count);
    while result.len() < count {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let val = rng % limit;
        if taken.insert(val) {
            result.push(val);
        }
    }
    result
}

fn main() {
    println!("=== Adaptive PI Generator Benchmark ===\n");

    // Test cases from our scaling study
    let cases: Vec<(&str, (u8, usize))> = vec![
        ("access_control", (12, 128)),
        ("dense_12var", (12, 512)),
        ("sparse_14", (14, 128)),
        ("sparse_16", (16, 256)),
        ("xor_4", (4, 8)), // dense: XOR has 8/16 = 50%
        ("and_3", (3, 7)), // dense: AND(3) = all true except 0
    ];

    for (name, (n, count)) in cases {
        let mt = gen_sparse(n, count, n as u64 * 1000);
        let tt = TruthTable::from_minterms(n as usize, &mt, &[]).unwrap();

        // Adaptive
        let start = Instant::now();
        let pis = find_prime_implicants_adaptive(&tt);
        let dur = start.elapsed();

        println!(
            "{:15} [n={}, {}mt]: {:>10} → {} PIs",
            name,
            n,
            count,
            format!("{:?}", dur),
            pis.len()
        );
    }
}
