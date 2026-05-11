use qm_agent::qm::primes::{TruthTable, find_prime_implicants as mc_find_pis};
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
    let cases = vec![
        ("and_3", 3, vec![(1 << 3) - 1]),
        ("or_3", 3, vec![1, 2, 3, 4, 5, 6, 7]),
        ("xor_4", 4, vec![1, 2, 4, 7, 8, 11, 13, 14]),
        ("access_12", 12, gen_sparse(12, 128, 17)),
        ("dense_12", 12, gen_sparse(12, 512, 42)),
        ("sparse_14", 14, gen_sparse(14, 128, 99)),
        ("sparse_16", 16, gen_sparse(16, 256, 42)),
    ];

    println!(
        "{:15} {:>8} {:>10} {:>10} {:>10} {:>10}",
        "Problem", "n", "CCubes", "Adaptive", "Ratio", "Verdict"
    );
    println!("{}", "-".repeat(65));

    for (name, n, mt) in cases {
        let tt = TruthTable::from_minterms(n as usize, &mt, &[]).unwrap();
        let limit = 1u64 << n;
        let density = mt.len() as f64 / limit as f64 * 100.0;

        // Time CCubes
        let t0 = Instant::now();
        let mc_pis = mc_find_pis(&tt, n as usize);
        let mc_dur = t0.elapsed();

        // Time Adaptive
        let t1 = Instant::now();
        let ad_pis = find_prime_implicants_adaptive(&tt);
        let ad_dur = t1.elapsed();

        // Verify same PI count
        let verdict = if mc_pis.len() == ad_pis.len() {
            "OK"
        } else {
            "PI MISMATCH"
        };

        let ratio = if mc_dur.as_nanos() > 0 && ad_dur.as_nanos() > 0 {
            ad_dur.as_secs_f64() / mc_dur.as_secs_f64()
        } else {
            0.0
        };

        println!(
            "{:15} {:2} {:.1}% {:>8} {:>10} {:>10} {}",
            name,
            n,
            density,
            format!("{:?}", mc_dur),
            format!("{:?}", ad_dur),
            if ratio >= 1.0 {
                format!("{:.0}x", ratio)
            } else {
                format!("{:.0}x", 1.0 / ratio)
            },
            verdict
        );
    }
}
