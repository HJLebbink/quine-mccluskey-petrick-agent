// CNF to DNF Consistency Tests
//
// Long-running randomized tests that verify consistency across:
// - Different encoding implementations (Enc16, Enc32, Enc64)
// - Different optimization modes (with/without early pruning)
//
// Run with: cargo test --test cnf_dnf_consistency_tests -- --ignored --nocapture

use qm_agent::cnf_dnf::{self, OptimizedFor};
use qm_agent::qm::{Enc16, Enc32, Enc64};
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
    let mut rng = StdRng::from_os_rng();
    const MAX_EXPERIMENTS: usize = 100_000;

    for experiment in 0..MAX_EXPERIMENTS {
        let time_begin = Instant::now();

        let n_variables = rng.random_range(1..=64);
        let n_conjunctions = rng.random_range(1..=10);
        let n_disjunctions = rng.random_range(1..=n_variables);

        print!(
            "experiment {experiment}: n_variables={n_variables}; conjunctions {n_conjunctions}; disjunctions {n_disjunctions}"
        );

        let mut cnf_64: Vec<u64> = Vec::new();

        for _ in 0..n_conjunctions {
            let mut conjunction = 0u64;
            for _ in 0..n_disjunctions {
                let r = rng.random_range(0..n_variables);
                conjunction |= 1u64 << r;
            }
            cnf_64.push(conjunction);
        }

        // Reference implementation: Encoding64 (supports all variable counts up to 64)
        let dnf_64_a = cnf_dnf::cnf_to_dnf::<Enc64>(&cnf_64, n_variables, OptimizedFor::X64).unwrap();


        // Test that different encodings produce identical results when compatible
        if n_variables <= 32 {
            let dnf_32 = cnf_dnf::cnf_to_dnf::<Enc32>(&cnf_64, n_variables, OptimizedFor::AutoDetect).unwrap();
            if !dnf_equal(&dnf_64_a, &dnf_32) {
                println!(" Experiment {experiment}: Enc64/X64 != Enc32/AutoDetect with {n_variables} variables");
                panic!("DNF mismatch: Enc64/X64 vs Enc32/AutoDetect with {n_variables} variables");
            }
        }

        if n_variables <= 16 {
            let dnf_16 = cnf_dnf::cnf_to_dnf::<Enc16>(&cnf_64, n_variables, OptimizedFor::AutoDetect).unwrap();
            if !dnf_equal(&dnf_64_a, &dnf_16) {
                println!(" Experiment {experiment}: Enc64/X64 != Enc16/AutoDetect with {n_variables} variables");
                panic!("DNF mismatch: Enc64/X64 vs Enc16/AutoDetect with {n_variables} variables");
            }
        }

        let elapsed = time_begin.elapsed();
        println!(" took {} seconds", elapsed.as_secs());

        // Progress report every 1000 experiments
        if (experiment + 1) % 1000 == 0 {
            println!("=== Completed {} / {MAX_EXPERIMENTS} experiments ===", experiment + 1);
        }
    }

    println!("✓ All {MAX_EXPERIMENTS} equality tests passed!");
}

#[test]
#[ignore] // Run explicitly with: cargo test equality_test_minimal -- --ignored --nocapture
fn equality_test_minimal() {
    let mut rng = StdRng::from_os_rng();
    const MAX_EXPERIMENTS: usize = 100_000;

    for experiment in 0..MAX_EXPERIMENTS {
        let time_begin = Instant::now();

        let n_variables = rng.random_range(1..=64);
        let n_conjunctions = rng.random_range(1..=10);
        let n_disjunctions = rng.random_range(1..=n_variables);

        print!("experiment {experiment}: n_variables={n_variables}; conjunctions {n_conjunctions}; disjunctions {n_disjunctions}");

        let mut cnf: Vec<u64> = Vec::new();

        for _ in 0..n_conjunctions {
            let mut conjunction = 0u64;
            for _ in 0..n_disjunctions {
                let r = rng.random_range(0..n_variables);
                conjunction |= 1u64 << r;
            }
            cnf.push(conjunction);
        }

        // Test that multiple runs produce consistent results
        // Use appropriate encoding based on n_variables
        let (dnf_a, dnf_b, encoding_name) = if n_variables <= 16 {
            (
                cnf_dnf::cnf_to_dnf_minimal::<Enc16>(
                    &cnf,
                    n_variables, OptimizedFor::AutoDetect).unwrap(),
                cnf_dnf::cnf_to_dnf_minimal_reference::<Enc16>(
                    &cnf,
                    n_variables, OptimizedFor::AutoDetect).unwrap(),
                "Enc16"
            )
        } else if n_variables <= 32 {
            (
                cnf_dnf::cnf_to_dnf_minimal::<Enc32>(
                    &cnf,
                    n_variables, OptimizedFor::AutoDetect).unwrap(),
                cnf_dnf::cnf_to_dnf_minimal_reference::<Enc32>(
                    &cnf,
                    n_variables, OptimizedFor::AutoDetect).unwrap(),
                "Enc32"
            )
        } else {
            (
                cnf_dnf::cnf_to_dnf_minimal::<Enc64>(
                    &cnf,
                    n_variables, OptimizedFor::AutoDetect).unwrap(),
                cnf_dnf::cnf_to_dnf_minimal_reference::<Enc64>(
                    &cnf,
                    n_variables, OptimizedFor::AutoDetect).unwrap(),
                "Enc64"
            )
        };

        if !dnf_equal(&dnf_a, &dnf_b) {
            println!(" Experiment {experiment}: Minimal DNF mismatch with {encoding_name} and {n_variables} variables");
            println!("DNF_A (run 1): {dnf_a:?}");
            println!("DNF_B (run 2): {dnf_b:?}");
            panic!("Minimal DNF mismatch: {encoding_name}/AutoDetect produced inconsistent results with {n_variables} variables");
        }

        let elapsed = time_begin.elapsed();
        println!(" took {} seconds", elapsed.as_secs());

        // Progress report every 1000 experiments
        if (experiment + 1) % 1000 == 0 {
            println!("=== Completed {} / {MAX_EXPERIMENTS} minimal tests ===", experiment + 1);
        }
    }

    println!("✓ All {MAX_EXPERIMENTS} minimal equality tests passed!");
}

#[test]
fn quick_equality_smoke_test_cnf_dnf() {
    // Quick smoke test (runs as part of the normal test suite)
    let mut rng = StdRng::seed_from_u64(42);

    const N_EXPERIMENTS: usize = 10;

    for _ in 0..N_EXPERIMENTS {

        for n_variables in [1, 2, 7, 8, 9, 15, 16, 17, 31, 32, 33] {
            let n_conjunctions = rng.random_range(1..=5);
            let n_disjunctions = rng.random_range(1..=n_variables);

            let mut cnf: Vec<u64> = Vec::new();
            for _ in 0..n_conjunctions {
                let mut conjunction = 0u64;
                for _ in 0..n_disjunctions {
                    let r = rng.random_range(0..n_variables);
                    conjunction |= 1u64 << r;
                }
                cnf.push(conjunction);
            }

            // Reference: Encoding64 (supports all variable counts)
            let dnf_64 = cnf_dnf::cnf_to_dnf::<Enc64>(&cnf, n_variables, OptimizedFor::X64).unwrap();

            if OptimizedFor::Avx512_64bits.is_supported() {
                let dnf_64_a = cnf_dnf::cnf_to_dnf::<Enc64>(&cnf, n_variables, OptimizedFor::Avx512_64bits).unwrap();
                assert!(
                    dnf_equal(&dnf_64, &dnf_64_a),
                    "DNF mismatch: Enc64/X64 vs Enc64/Avx512_64bits with {} variables",
                    n_variables
                );
            }

            // Test that different encodings produce identical results when compatible
            if n_variables <= 32 {
                if OptimizedFor::Avx512_64bits.is_supported() {
                    let dnf_x = cnf_dnf::cnf_to_dnf::<Enc32>(&cnf, n_variables, OptimizedFor::Avx512_64bits ).unwrap();
                    assert!(
                        dnf_equal(&dnf_64, &dnf_x),
                        "DNF mismatch: Enc64/X64 vs Enc32/Avx512_64bits with {} variables",
                        n_variables
                    );
                }
                if OptimizedFor::Avx512_32bits.is_supported() {
                    let dnf_x = cnf_dnf::cnf_to_dnf::<Enc32>(&cnf, n_variables, OptimizedFor::Avx512_32bits ).unwrap();
                    assert!(
                        dnf_equal(&dnf_64, &dnf_x),
                        "DNF mismatch: Enc64/X64 vs Enc32/Avx512_32bits with {} variables",
                        n_variables
                    );
                }
            }

            if n_variables <= 16 {
                if OptimizedFor::Avx512_64bits.is_supported() {
                    let dnf_x = cnf_dnf::cnf_to_dnf::<Enc16>(&cnf, n_variables, OptimizedFor::Avx512_64bits ).unwrap();
                    assert!(
                        dnf_equal(&dnf_64, &dnf_x),
                        "DNF mismatch: Enc64/X64 vs Enc16/Avx512_64bits with {} variables",
                        n_variables
                    );
                }
                if OptimizedFor::Avx512_32bits.is_supported() {
                    let dnf_x = cnf_dnf::cnf_to_dnf::<Enc16>(&cnf, n_variables, OptimizedFor::Avx512_32bits ).unwrap();
                    assert!(
                        dnf_equal(&dnf_64, &dnf_x),
                        "DNF mismatch: Enc64/X64 vs Enc16/Avx512_32bits with {} variables",
                        n_variables
                    );
                }
                if OptimizedFor::Avx512_16bits.is_supported() {
                    let dnf_x = cnf_dnf::cnf_to_dnf::<Enc16>(&cnf, n_variables, OptimizedFor::Avx512_16bits ).unwrap();
                    assert!(
                        dnf_equal(&dnf_64, &dnf_x),
                        "DNF mismatch: Enc64/X64 vs Enc16/Avx512_16bits with {} variables",
                        n_variables
                    );
                }
            }

            if n_variables <= 8 {
                if OptimizedFor::Avx512_8bits.is_supported() {
                    let dnf_x = cnf_dnf::cnf_to_dnf::<Enc16>(&cnf, n_variables, OptimizedFor::Avx512_8bits ).unwrap();
                    assert!(
                        dnf_equal(&dnf_64, &dnf_x),
                        "DNF mismatch: Enc64/X64 vs Enc16/Avx512_8bits with {} variables",
                        n_variables
                    );
                }
            }
        }
    }
}
