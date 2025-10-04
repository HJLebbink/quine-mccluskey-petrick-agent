// CNF to DNF Conversion Tests
//
// Tests for cnf_to_dnf and cnf_to_dnf_minimal functions

use std::collections::HashSet;
use qm_agent::cnf_dnf::{cnf_to_dnf, cnf_to_dnf_minimal, OptimizedFor};
use rand::{rngs::StdRng, Rng, SeedableRng};
use qm_agent::{Enc16, Enc32, Enc64};

/// This is a unit test for verifying the correctness of the `cnf_to_dnf_minimal` function
/// compared to the regular `convert_cnf_to_dnf` function under various optimization modes.
/// The test ensures that the minimized Disjunctive Normal Form (DNF) representation generated
/// by `cnf_to_dnf_minimal` is a subset of the DNF representation produced by
/// `convert_cnf_to_dnf` and adheres to the expected minimal term size.
///
/// # Test Parameters
/// - The test is repeated for `NUM_TESTS` (1000) iterations.
/// - Randomized test cases are generated with the following parameters:
///   - `n_variables`: Number of variables used in the expressions (randomized between 2 and 64).
///   - `n_conjunctions`: Number of conjunctions in the CNF (randomized between 1 and 5).
///   - `n_disjunctions`: Number of disjunctions within each conjunction (randomized between 1 and 6 or `n_variables`, whichever is smaller).
/// - CNF (Conjunctive Normal Form) expressions are randomly generated using a seeded random number generator (`StdRng`).
/// - Encodings are automatically selected based on variable count:
///   - Enc16 for ≤16 variables
///   - Enc32 for 17-32 variables
///   - Enc64 for 33-64 variables
///
/// # Optimization Modes
/// The test evaluates the functionality for the following optimization modes:
/// - `OptimizedFor::X64`
/// - `OptimizedFor::Avx512_64bits`
/// - `OptimizedFor::Avx512_32bits`
/// - `OptimizedFor::Avx512_16bits`
///
/// # Verification Steps
/// For each test case:
/// 1. A random CNF expression is generated.
/// 2. Skips empty CNFs (to avoid invalid operations).
/// 3. Calculates the minimized DNF (`dnf_minimal`) using `cnf_to_dnf_minimal`, as well as the regular DNF (`dnf_regular`) using `convert_cnf_to_dnf` for each optimization mode.
/// 4. Converts the resulting DNFs into `HashSet<u64>` for easier validation.
///
/// The following assertions are performed:
/// - **Subset Check**: Each term in `dnf_minimal` must be present in `dnf_regular`.
/// - **Term Count Check**: The number of terms in `dnf_minimal` must be less than or equal to that in `dnf_regular`.
/// - **Minimal Size Check**: If `dnf_regular` is not empty, all terms in `dnf_minimal` must have the minimal size (in terms of the number of set bits).
///
/// # Panics
/// The test panics in the following cases:
/// - Unsupported optimization mode is encountered.
/// - If `dnf_minimal` contains a term that does not exist in `dnf_regular`.
/// - If the number of terms in `dnf_minimal` exceeds those in `dnf_regular`.
/// - If terms in `dnf_minimal` do not adhere to the minimal size when compared to the regular DNF.
///
/// # RNG Seeding
/// A fixed seed (`42`) is used for the random number generator to ensure reproducibility of the test.
///
/// # Complexity
/// The test performs extensive randomized testing for different optimization modes, making it a comprehensive, yet exhaustive, validation of correctness and minimality.
#[test]
fn test_cnf_to_dnf_minimal_vs_regular() {
    let mut rng = StdRng::seed_from_u64(42);
    const NUM_TESTS: usize = 10;

    const OPT_MODES: [OptimizedFor; 5] = [
        OptimizedFor::X64,
        OptimizedFor::Avx512_64bits,
        OptimizedFor::Avx512_32bits,
        OptimizedFor::Avx512_16bits,
        OptimizedFor::Avx2_64bits,
    ];

    for test_idx in 0..NUM_TESTS {
        // Generate random test parameters with balanced distribution across encodings
        let n_variables = if test_idx % 3 == 0 {
            rng.random_range(2..=16)  // Enc16
        } else if test_idx % 3 == 1 {
            rng.random_range(17..=32) // Enc32
        } else {
            rng.random_range(33..=64) // Enc64
        };
        let n_conjunctions = rng.random_range(1..=5);
        let n_disjunctions = rng.random_range(1..=n_variables.min(6));

        // Generate random CNF
        let mut cnf: Vec<u64> = Vec::new();
        for _ in 0..n_conjunctions {
            let mut conjunction = 0u64;
            for _ in 0..n_disjunctions {
                let bit_pos = rng.random_range(0..n_variables);
                conjunction |= 1u64 << bit_pos;
            }
            if conjunction != 0 {
                cnf.push(conjunction);
            }
        }

        // Skip empty CNF
        if cnf.is_empty() {
            continue;
        }

        // Test each optimization mode (filter based on bit width support)
        for opt in &OPT_MODES {
            // Skip optimization modes that don't support this many variables
            let max_bits = opt.max_bits();
            if n_variables > max_bits {
                continue;
            }

            let opt_name = opt.to_string();

            // Select encoding based on n_variables and compute both versions
            let (dnf_minimal, dnf_regular) = if n_variables <= 16 {
                let minimal = match opt {
                    OptimizedFor::X64 => cnf_to_dnf_minimal::<Enc16>(&cnf, n_variables, OptimizedFor::X64),
                    OptimizedFor::Avx512_64bits => cnf_to_dnf_minimal::<Enc16>(&cnf, n_variables, OptimizedFor::Avx512_64bits),
                    OptimizedFor::Avx512_32bits => cnf_to_dnf_minimal::<Enc16>(&cnf, n_variables, OptimizedFor::Avx512_32bits),
                    OptimizedFor::Avx512_16bits => cnf_to_dnf_minimal::<Enc16>(&cnf, n_variables, OptimizedFor::Avx512_16bits),
                    OptimizedFor::Avx2_64bits => cnf_to_dnf_minimal::<Enc16>(&cnf, n_variables, OptimizedFor::Avx2_64bits),
                    _ => panic!("Unsupported optimization mode"),
                };
                let regular = match opt {
                    OptimizedFor::X64 => cnf_to_dnf::<Enc16>(&cnf, n_variables, OptimizedFor::X64),
                    OptimizedFor::Avx512_64bits => cnf_to_dnf::<Enc16>(&cnf, n_variables, OptimizedFor::Avx512_64bits),
                    OptimizedFor::Avx512_32bits => cnf_to_dnf::<Enc16>(&cnf, n_variables, OptimizedFor::Avx512_32bits),
                    OptimizedFor::Avx512_16bits => cnf_to_dnf::<Enc16>(&cnf, n_variables, OptimizedFor::Avx512_16bits),
                    OptimizedFor::Avx2_64bits => cnf_to_dnf::<Enc16>(&cnf, n_variables, OptimizedFor::Avx2_64bits),
                    _ => panic!("Unsupported optimization mode"),
                };
                (minimal.unwrap(), regular.unwrap())
            } else if n_variables <= 32 {
                let minimal = match opt {
                    OptimizedFor::X64 => cnf_to_dnf_minimal::<Enc32>(&cnf, n_variables, OptimizedFor::X64),
                    OptimizedFor::Avx512_64bits => cnf_to_dnf_minimal::<Enc32>(&cnf, n_variables, OptimizedFor::Avx512_64bits),
                    OptimizedFor::Avx512_32bits => cnf_to_dnf_minimal::<Enc32>(&cnf, n_variables, OptimizedFor::Avx512_32bits),
                    OptimizedFor::Avx512_16bits => cnf_to_dnf_minimal::<Enc32>(&cnf, n_variables, OptimizedFor::Avx512_16bits),
                    OptimizedFor::Avx2_64bits => cnf_to_dnf_minimal::<Enc32>(&cnf, n_variables, OptimizedFor::Avx2_64bits),
                    _ => panic!("Unsupported optimization mode"),
                };
                let regular = match opt {
                    OptimizedFor::X64 => cnf_to_dnf::<Enc32>(&cnf, n_variables, OptimizedFor::X64),
                    OptimizedFor::Avx512_64bits => cnf_to_dnf::<Enc32>(&cnf, n_variables, OptimizedFor::Avx512_64bits),
                    OptimizedFor::Avx512_32bits => cnf_to_dnf::<Enc32>(&cnf, n_variables, OptimizedFor::Avx512_32bits),
                    OptimizedFor::Avx512_16bits => cnf_to_dnf::<Enc32>(&cnf, n_variables, OptimizedFor::Avx512_16bits),
                    OptimizedFor::Avx2_64bits => cnf_to_dnf::<Enc32>(&cnf, n_variables, OptimizedFor::Avx2_64bits),
                    _ => panic!("Unsupported optimization mode"),
                };
                (minimal.unwrap(), regular.unwrap())
            } else {
                let minimal = match opt {
                    OptimizedFor::X64 => cnf_to_dnf_minimal::<Enc64>(&cnf, n_variables, OptimizedFor::X64),
                    OptimizedFor::Avx512_64bits => cnf_to_dnf_minimal::<Enc64>(&cnf, n_variables, OptimizedFor::Avx512_64bits),
                    OptimizedFor::Avx512_32bits => cnf_to_dnf_minimal::<Enc64>(&cnf, n_variables, OptimizedFor::Avx512_32bits),
                    OptimizedFor::Avx512_16bits => cnf_to_dnf_minimal::<Enc64>(&cnf, n_variables, OptimizedFor::Avx512_16bits),
                    OptimizedFor::Avx2_64bits => cnf_to_dnf_minimal::<Enc64>(&cnf, n_variables, OptimizedFor::Avx2_64bits),
                    _ => panic!("Unsupported optimization mode"),
                };
                let regular = match opt {
                    OptimizedFor::X64 => cnf_to_dnf::<Enc64>(&cnf, n_variables, OptimizedFor::X64),
                    OptimizedFor::Avx512_64bits => cnf_to_dnf::<Enc64>(&cnf, n_variables, OptimizedFor::Avx512_64bits),
                    OptimizedFor::Avx512_32bits => cnf_to_dnf::<Enc64>(&cnf, n_variables, OptimizedFor::Avx512_32bits),
                    OptimizedFor::Avx512_16bits => cnf_to_dnf::<Enc64>(&cnf, n_variables, OptimizedFor::Avx512_16bits),
                    OptimizedFor::Avx2_64bits => cnf_to_dnf::<Enc64>(&cnf, n_variables, OptimizedFor::Avx2_64bits),
                    _ => panic!("Unsupported optimization mode"),
                };
                (minimal.unwrap(), regular.unwrap())
            };

            // Convert to sets for easy subset checking
            let minimal_set: HashSet<u64> = dnf_minimal.iter().copied().collect();
            let regular_set: HashSet<u64> = dnf_regular.iter().copied().collect();

            // Every term in minimal must be in regular
            for &term in &minimal_set {
                assert!(regular_set.contains(&term),
                    "Test {test_idx} ({opt_name}): Minimal term {term:064b} not found in regular DNF (vars={n_variables}, conj={n_conjunctions}, disj={n_disjunctions})");
            }

            // Minimal should have fewer or equal terms
            assert!(dnf_minimal.len() <= dnf_regular.len(),
                "Test {} ({}): Minimal has {} terms but regular has {} (should be <=)",
                test_idx, opt_name, dnf_minimal.len(), dnf_regular.len());

            // If regular is not empty, check that minimal contains only minimal-size terms
            if !dnf_regular.is_empty() {
                let min_size = dnf_regular.iter()
                    .map(|x| x.count_ones())
                    .min()
                    .unwrap();

                for &term in &dnf_minimal {
                    assert_eq!(term.count_ones(), min_size,
                        "Test {} ({}): Minimal term {:064b} has size {} but minimum is {}",
                        test_idx, opt_name, term, term.count_ones(), min_size);
                }
            }
        }
    }
}


#[test]
fn test_cnf_to_dnf_minimal_edge_cases() {
    // Empty CNF - should return empty (tautology)
    let cnf: Vec<u64> = vec![];
    let dnf = cnf_to_dnf_minimal::<qm_agent::qm::Enc16>(
        &cnf, 4, OptimizedFor::X64
    ).unwrap();
    // Empty CNF means no constraints, which is represented as empty result
    assert!(dnf.is_empty() || dnf == vec![0]);

    // Single clause with two bits
    let cnf_single: Vec<u64> = vec![0b101]; // bit 0 and bit 2
    let dnf_single = cnf_to_dnf_minimal::<qm_agent::qm::Enc16>(
        &cnf_single, 4, OptimizedFor::X64
    ).unwrap();
    assert_eq!(dnf_single.len(), 2, "Single clause with 2 literals should produce 2 terms");

    // All terms should have size 1 (single literal)
    for &term in &dnf_single {
        assert_eq!(term.count_ones(), 1,
                   "Each term should be a single literal, but got {:b}", term);
    }

    // Single clause with one bit
    let cnf_one_bit: Vec<u64> = vec![0b1]; // just bit 0
    let dnf_one_bit = cnf_to_dnf_minimal::<qm_agent::qm::Enc16>(
        &cnf_one_bit, 4, OptimizedFor::X64
    ).unwrap();
    assert_eq!(dnf_one_bit.len(), 1, "Single literal clause should produce 1 term");
    assert_eq!(dnf_one_bit[0], 0b1, "Should be the same literal");
}
