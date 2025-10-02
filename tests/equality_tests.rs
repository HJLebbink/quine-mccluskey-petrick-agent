// Equality tests for CNF to DNF conversion
// These are long-running randomized tests for quality assurance
// Run with: cargo test --test equality_tests -- --ignored --nocapture

use qm_agent::cnf_dnf::{self, OptimizedFor};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::time::Instant;

/// Check if two DNF results are equal (order doesn't matter)
fn dnf_equal(dnf1: &[u64], dnf2: &[u64]) -> bool {
    if dnf1.len() != dnf2.len() {
        return false;
    }
    let mut sorted1 = dnf1.to_vec();
    let mut sorted2 = dnf2.to_vec();
    sorted1.sort_unstable();
    sorted2.sort_unstable();
    sorted1 == sorted2
}

#[test]
#[ignore] // Run explicitly with: cargo test equality_test -- --ignored --nocapture
fn equality_test() {
    let mut rng = StdRng::from_entropy();
    const MAX_EXPERIMENTS: usize = 100_000;

    for experiment in 0..MAX_EXPERIMENTS {
        let time_begin = Instant::now();

        let n_variables = rng.gen_range(1..=64);
        let n_conjunctions = rng.gen_range(1..=10);
        let n_disjunctions = rng.gen_range(1..=n_variables);

        print!(
            "experiment {}: n_variables={}; conjunctions {}; disjunctions {}",
            experiment, n_variables, n_conjunctions, n_disjunctions
        );

        let mut cnf_64: Vec<u64> = Vec::new();

        for _ in 0..n_conjunctions {
            let mut conjunction = 0u64;
            for _ in 0..n_disjunctions {
                let r = rng.gen_range(0..n_variables);
                conjunction |= 1u64 << r;
            }
            cnf_64.push(conjunction);
        }

        // Reference implementation: auto-detect best optimization
        let dnf_64x = cnf_dnf::convert_cnf_to_dnf(&cnf_64, n_variables, OptimizedFor::detect_best(n_variables));

        // Test all optimized versions (currently all fall back to X64, but structure is here)
        if n_variables <= 64 {
            let dnf_64a = cnf_dnf::convert_cnf_to_dnf(
                &cnf_64,
                n_variables,
                OptimizedFor::Avx512_64bits,
            );
            if !dnf_equal(&dnf_64x, &dnf_64a) {
                println!(" Experiment {}: 1-2a NOT EQUAL", experiment);
                panic!("DNF mismatch between X64 and Avx512_64bits");
            }

            let dnf_64b =
                cnf_dnf::convert_cnf_to_dnf(&cnf_64, n_variables, OptimizedFor::Avx2_64bits);
            if !dnf_equal(&dnf_64x, &dnf_64b) {
                println!(" Experiment {}: 1-2b NOT EQUAL", experiment);
                panic!("DNF mismatch between X64 and Avx2_64bits");
            }
        }

        if n_variables <= 32 {
            let dnf_32 = cnf_dnf::convert_cnf_to_dnf(
                &cnf_64,
                n_variables,
                OptimizedFor::Avx512_32bits,
            );
            if !dnf_equal(&dnf_64x, &dnf_32) {
                println!(" Experiment {}: 1-3 NOT EQUAL", experiment);
                panic!("DNF mismatch with 32-bit");
            }
        }

        if n_variables <= 16 {
            let dnf_16 = cnf_dnf::convert_cnf_to_dnf(
                &cnf_64,
                n_variables,
                OptimizedFor::Avx512_16bits,
            );
            if !dnf_equal(&dnf_64x, &dnf_16) {
                println!(" Experiment {}: 1-4 NOT EQUAL", experiment);
                panic!("DNF mismatch with 16-bit");
            }
        }

        if n_variables <= 8 {
            let dnf_8 =
                cnf_dnf::convert_cnf_to_dnf(&cnf_64, n_variables, OptimizedFor::Avx512_8bits);
            if !dnf_equal(&dnf_64x, &dnf_8) {
                println!(" Experiment {}: 1-5 NOT EQUAL", experiment);
                panic!("DNF mismatch with 8-bit");
            }
        }

        let elapsed = time_begin.elapsed();
        println!(" took {} seconds", elapsed.as_secs());

        // Progress report every 1000 experiments
        if (experiment + 1) % 1000 == 0 {
            println!("=== Completed {} / {} experiments ===", experiment + 1, MAX_EXPERIMENTS);
        }
    }

    println!("✓ All {} equality tests passed!", MAX_EXPERIMENTS);
}

#[test]
#[ignore] // Run explicitly with: cargo test equality_test_minimal -- --ignored --nocapture
fn equality_test_minimal() {
    let mut rng = StdRng::from_entropy();
    const MAX_EXPERIMENTS: usize = 100_000;

    for experiment in 0..MAX_EXPERIMENTS {
        let time_begin = Instant::now();

        let n_variables = rng.gen_range(1..=64);
        let n_conjunctions = rng.gen_range(1..=10);
        let n_disjunctions = rng.gen_range(1..=n_variables);

        print!(
            "experiment {}: n_variables={}; conjunctions {}; disjunctions {}",
            experiment, n_variables, n_conjunctions, n_disjunctions
        );

        let mut cnf: Vec<u64> = Vec::new();

        for _ in 0..n_conjunctions {
            let mut conjunction = 0u64;
            for _ in 0..n_disjunctions {
                let r = rng.gen_range(0..n_variables);
                conjunction |= 1u64 << r;
            }
            cnf.push(conjunction);
        }

        // Test that EARLY_PRUNE=true and EARLY_PRUNE=false produce the same minimal result
        let dnf_a = cnf_dnf::convert_cnf_to_dnf_minimal(
            &cnf,
            n_variables,
            OptimizedFor::X64,
            true, // EARLY_PRUNE
        );

        let dnf_b = cnf_dnf::convert_cnf_to_dnf_minimal(
            &cnf,
            n_variables,
            OptimizedFor::X64,
            false, // no EARLY_PRUNE
        );

        if !dnf_equal(&dnf_a, &dnf_b) {
            println!(" Experiment {}: minimal 1-2 NOT EQUAL", experiment);
            println!("DNF_A (early prune): {:?}", dnf_a);
            println!("DNF_B (no prune): {:?}", dnf_b);
            panic!("Minimal DNF mismatch between early prune and no prune");
        }

        let elapsed = time_begin.elapsed();
        println!(" took {} seconds", elapsed.as_secs());

        // Progress report every 1000 experiments
        if (experiment + 1) % 1000 == 0 {
            println!("=== Completed {} / {} minimal tests ===", experiment + 1, MAX_EXPERIMENTS);
        }
    }

    println!("✓ All {} minimal equality tests passed!", MAX_EXPERIMENTS);
}

#[test]
fn quick_equality_smoke_test_cnf_dnf() {
    // Quick smoke test (runs as part of the normal test suite)
    let mut rng = StdRng::seed_from_u64(42);

    const N_EXPERIMENTS: usize = 10;

    for _ in 0..N_EXPERIMENTS {

        for n_variables in [1, 2, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64] {
            let n_conjunctions = rng.gen_range(1..=5);
            let n_disjunctions = rng.gen_range(1..=n_variables);

            let mut cnf: Vec<u64> = Vec::new();
            for _ in 0..n_conjunctions {
                let mut conjunction = 0u64;
                for _ in 0..n_disjunctions {
                    let r = rng.gen_range(0..n_variables);
                    conjunction |= 1u64 << r;
                }
                cnf.push(conjunction);
            }

            let dnf_x64 = cnf_dnf::convert_cnf_to_dnf(&cnf, n_variables, OptimizedFor::X64);

            if n_variables <= 32 {
                let dnf_avx512_64 = cnf_dnf::convert_cnf_to_dnf(&cnf, n_variables, OptimizedFor::Avx512_64bits);
                assert!(
                    dnf_equal(&dnf_x64, &dnf_avx512_64),
                    "DNF mismatch in quick smoke test. x64 != avx512_64bits"
                );
            }

            if n_variables <= 16 {
                let dnf_avx512_32 = cnf_dnf::convert_cnf_to_dnf(&cnf, n_variables, OptimizedFor::Avx512_32bits);
                assert!(
                    dnf_equal(&dnf_x64, &dnf_avx512_32),
                    "DNF mismatch in quick smoke test. x64 != avx512_32bits"
                );
            }

            if n_variables <= 8 {
                let dnf_avx512_16 = cnf_dnf::convert_cnf_to_dnf(&cnf, n_variables, OptimizedFor::Avx512_16bits);
                assert!(
                    dnf_equal(&dnf_x64, &dnf_avx512_16),
                    "DNF mismatch in quick smoke test. x64 != avx512_16bits"
                );
            }

            if n_variables <= 4 {
                let dnf_avx512_8 = cnf_dnf::convert_cnf_to_dnf(&cnf, n_variables, OptimizedFor::Avx512_8bits);
                assert!(
                    dnf_equal(&dnf_x64, &dnf_avx512_8),
                    "DNF mismatch in quick smoke test. x64 != avx512_8bits"
                );
            }
        }
    }
}
