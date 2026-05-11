use qm_agent::qm::primes::{TruthTable, find_prime_implicants as mc_find_pis};
use qm_agent::qm::primes_adaptive::find_prime_implicants_adaptive;

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
    println!("=== PI Verification ===\n");

    // Check: does the adaptive QM version cover ALL minterms the same as CCubes?
    let cases = vec![
        ("and_3", 3, vec![(1 << 3) - 1]),
        ("or_3", 3, vec![1, 2, 3, 4, 5, 6, 7]),
        ("xor_4", 4, vec![1, 2, 4, 7, 8, 11, 13, 14]),
        ("sparse_12", 12, gen_sparse(12, 128, 17)),
        ("dense_12_512", 12, gen_sparse(12, 512, 42)),
    ];

    let mut all_match = true;
    for (name, n, mt) in cases {
        let tt = TruthTable::from_minterms(n, &mt, &[]).unwrap();
        let mc_pis = mc_find_pis(&tt, n);
        let ad_pis = find_prime_implicants_adaptive(&tt);

        // Compute covered minterms for CCubes
        let mut mc_covered = Vec::new();
        for m in &mt {
            for pi in &mc_pis {
                let fixed = pi.cond & !pi.mask;
                if (m & fixed) == (pi.data & fixed) {
                    mc_covered.push(*m);
                    break;
                }
            }
        }

        // Compute covered minterms for Adaptive
        let mut ad_covered = Vec::new();
        for m in &mt {
            for pi in &ad_pis {
                let fixed = pi.cond & !pi.mask;
                if (m & fixed) == (pi.data & fixed) {
                    ad_covered.push(*m);
                    break;
                }
            }
        }

        mc_covered.sort();
        mc_covered.dedup();
        ad_covered.sort();
        ad_covered.dedup();

        let matches = mc_covered == ad_covered;
        if !matches {
            all_match = false;
            println!(
                "{:20} MISMATCH: mc={}mt covered vs ad={}mt covered",
                name,
                mc_covered.len(),
                ad_covered.len()
            );
            println!(
                "  CCubes PIs: {}, Adaptive PIs: {}",
                mc_pis.len(),
                ad_pis.len()
            );

            // Find missing minterms
            for m in &mt {
                let mc_cov = mc_pis.iter().any(|pi| {
                    let fixed = pi.cond & !pi.mask;
                    (m & fixed) == (pi.data & fixed)
                });
                let ad_cov = ad_pis.iter().any(|pi| {
                    let fixed = pi.cond & !pi.mask;
                    (m & fixed) == (pi.data & fixed)
                });
                if mc_cov && !ad_cov {
                    if mc_covered.len() < 50 {
                        println!("    missing minterm: {}", m);
                    }
                }
            }
        } else {
            println!(
                "{:20} OK: {} minterms covered by {} vs {} PIs",
                name,
                mc_covered.len(),
                mc_pis.len(),
                ad_pis.len()
            );
        }
    }

    if all_match {
        println!("\n=== All PI sets are coverage-equivalent ===");
    } else {
        println!("\n=== Some PI sets differ ===");
    }
}
