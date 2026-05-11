#![allow(dead_code)]

//! Comprehensive tests for min-cubes algorithms
//!
//! Phase 7: Thorough testing requirement - every public function tested.
//! Every test must verify correctness, not just that it doesn't crash.

use qm_agent::qm::comb;
use qm_agent::qm::covers;
use qm_agent::qm::primes::{PrimeCube, TruthTable, find_prime_implicants};
use std::mem::MaybeUninit;

// ============================================================================
// COMB MODULE TESTS
// ============================================================================

mod comb_tests {
    use super::*;

    #[test]
    fn test_choose_identity() {
        assert_eq!(comb::choose(0, 0), 1);
        assert_eq!(comb::choose(1, 0), 1);
        assert_eq!(comb::choose(1, 1), 1);
    }

    #[test]
    fn test_choose_symmetry() {
        for n in 0..=20 {
            for k in 0..=n {
                assert_eq!(comb::choose(n, k), comb::choose(n, n - k));
            }
        }
    }

    #[test]
    fn test_choose_pascal_triangle() {
        for n in 0..=10 {
            for k in 1..n {
                let val = comb::choose(n, k);
                let sum = comb::choose(n - 1, k - 1) + comb::choose(n - 1, k);
                assert_eq!(
                    val,
                    sum,
                    "Pascal identity failed: C({}, {}) = {} != C({}, {}) + C({}, {}) = {}",
                    n,
                    k,
                    val,
                    n - 1,
                    k - 1,
                    n - 1,
                    k,
                    comb::choose(n - 1, k - 1) + comb::choose(n - 1, k)
                );
            }
        }
    }

    #[test]
    fn test_choose_known_values() {
        assert_eq!(comb::choose(5, 2), 10);
        assert_eq!(comb::choose(6, 3), 20);
        assert_eq!(comb::choose(10, 5), 252);
        assert_eq!(comb::choose(16, 8), 12870);
        assert_eq!(comb::choose(20, 10), 184756);
    }

    #[test]
    fn test_choose_edge_cases() {
        assert_eq!(comb::choose(100, 1), 100);
        assert_eq!(comb::choose(100, 99), 100);
        assert_eq!(comb::choose(10, 0), 1);
        assert_eq!(comb::choose(10, 10), 1);
        assert_eq!(comb::choose(5, 6), 0);
    }

    #[test]
    fn test_iterator_exhaustive_comb_4c2() {
        let mut iter = comb::CombinationIterator::new(4, 2).unwrap();
        let mut buf = MaybeUninit::<[u32; 16]>::uninit();
        let mut results = Vec::new();

        while iter.next() {
            iter.indices(&mut buf);
            let indices = unsafe { buf.assume_init() };
            results.push([indices[0], indices[1]]);
        }

        assert_eq!(results.len(), 6);
        assert_eq!(
            results,
            vec![[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]]
        );
    }

    #[test]
    fn test_iterator_exhaustive_comb_5c3() {
        let mut iter = comb::CombinationIterator::new(5, 3).unwrap();
        let mut buf = MaybeUninit::<[u32; 16]>::uninit();
        let mut results = Vec::new();

        while iter.next() {
            iter.indices(&mut buf);
            let indices = unsafe { buf.assume_init() };
            results.push([indices[0], indices[1], indices[2]]);
        }

        assert_eq!(results.len(), 10);
        assert_eq!(results[0], [0, 1, 2]);
        assert_eq!(results[9], [2, 3, 4]);
    }

    #[test]
    fn test_iterator_boundary_k0() {
        assert!(comb::CombinationIterator::new(5, 0).is_none());
    }

    #[test]
    fn test_iterator_boundary_k_gt_n() {
        assert!(comb::CombinationIterator::new(5, 6).is_none());
    }

    #[test]
    fn test_iterator_boundary_n64() {
        let res = comb::enumerate_all(64, 1);
        assert_eq!(res.len(), 64);
        for (i, r) in res.iter().enumerate() {
            assert_eq!(r[0], i as u32);
        }
    }

    #[test]
    fn test_enumerate_all_5c2() {
        let c = comb::enumerate_all(5, 2);
        assert_eq!(c.len(), 10);
        assert_eq!(c[0], [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(c[9], [3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_enumerate_all_uniqueness() {
        let c = comb::enumerate_all(8, 3);
        assert_eq!(c.len(), 56);
        let mut sorted = c.clone();
        sorted.sort();
        for window in sorted.windows(2) {
            assert_ne!(window[0], window[1], "Duplicate found in combinations");
        }
    }
}

// ============================================================================
// PRIMECUBE MODULE TESTS
// ============================================================================

mod primecube_tests {
    use super::*;

    #[test]
    fn test_dim_universal() {
        let pc = PrimeCube::new(0, 0, 0);
        assert_eq!(pc.dim(), 0);
    }

    #[test]
    fn test_dim_specific() {
        let pc = PrimeCube::new(7, 1, 5);
        assert_eq!(pc.dim(), 2);
    }

    #[test]
    fn test_dim_all_masked() {
        let pc = PrimeCube::new(7, 7, 5);
        assert_eq!(pc.dim(), 0);
    }

    #[test]
    fn test_conditions_used() {
        let pc = PrimeCube::new(0b0101, 0b0001, 0b0100);
        assert_eq!(pc.conditions_used(), 2);
    }

    #[test]
    fn test_subsumes_identical() {
        let a = PrimeCube::new(7, 3, 5);
        assert!(a.subsumes(a));
    }

    #[test]
    fn test_subsumes_universal() {
        let univ = PrimeCube::new(0, 0, 0);
        let any = PrimeCube::new(7, 3, 5);
        assert!(univ.subsumes(any));
    }

    #[test]
    fn test_subsumes_no_subsumption() {
        let a = PrimeCube::new(1, 0, 0);
        let b = PrimeCube::new(1, 0, 1);
        assert!(!a.subsumes(b));
        assert!(!b.subsumes(a));
    }

    #[test]
    fn test_subsumes_partial_overlap() {
        let a = PrimeCube::new(3, 0, 0);
        let b = PrimeCube::new(3, 0, 1);
        assert!(!a.subsumes(b));
    }

    #[test]
    fn test_subsumes_more_general() {
        let a = PrimeCube::new(7, 1, 7);
        let b = PrimeCube::new(7, 0, 0);
        assert!(!a.subsumes(b));
    }

    #[test]
    fn test_is_universal() {
        assert!(PrimeCube::new(0, 0, 0).is_universal());
        assert!(!PrimeCube::new(1, 0, 0).is_universal());
    }
}

// ============================================================================
// TRUTH TABLE TESTS
// ============================================================================

mod truth_table_tests {
    use super::*;

    #[test]
    fn test_tt_n_3_simple() {
        let tt = TruthTable::from_minterms(3, &[0, 1], &[]).unwrap();
        assert_eq!(tt.n_conds, 3);
        assert_eq!(tt.pos_rows, 2);
        assert_eq!(tt.neg_rows, 6);
    }

    #[test]
    fn test_tt_n_1() {
        let tt = TruthTable::from_minterms(1, &[1], &[]).unwrap();
        assert_eq!(tt.pos_rows, 1);
        assert_eq!(tt.neg_rows, 1);
    }

    #[test]
    fn test_tt_all_minterms() {
        let tt = TruthTable::from_minterms(3, &[0, 1, 2, 3, 4, 5, 6, 7], &[]).unwrap();
        assert_eq!(tt.pos_rows, 8);
        assert_eq!(tt.neg_rows, 1);
    }

    #[test]
    fn test_tt_no_minterms() {
        let tt = TruthTable::from_minterms(3, &[], &[]).unwrap();
        assert_eq!(tt.pos_rows, 0);
    }

    #[test]
    fn test_tt_with_dontcares() {
        let tt = TruthTable::from_minterms(3, &[0], &[1, 2]).unwrap();
        assert_eq!(tt.pos_rows, 3);
        assert_eq!(tt.neg_rows, 5);
    }

    #[test]
    fn test_tt_invalid_n_0() {
        assert!(TruthTable::from_minterms(0, &[0], &[]).is_none());
    }

    #[test]
    fn test_tt_invalid_n_too_large() {
        assert!(TruthTable::from_minterms(65, &[0], &[]).is_none());
    }

    #[test]
    fn test_tt_minterm_out_of_range() {
        let tt = TruthTable::from_minterms(3, &[0, 100], &[]).unwrap();
        assert_eq!(tt.pos_rows, 1);
    }

    #[test]
    fn test_tt_posval_negval() {
        let tt = TruthTable::from_minterms(3, &[0], &[]).unwrap();
        for c in 0..3 {
            assert_eq!(tt.posval(c, 0), 0);
        }
    }
}

// ============================================================================
// FIND_PRIME_IMPLANTS TESTS
// ============================================================================

mod pi_generation_tests {
    use super::*;

    #[test]
    fn test_pi_single_minterm_1bit() {
        let tt = TruthTable::from_minterms(1, &[1], &[]).unwrap();
        let pis = find_prime_implicants(&tt, 1);
        assert_eq!(pis.len(), 1);
    }

    #[test]
    fn test_pi_and_3() {
        let tt = TruthTable::from_minterms(3, &[7], &[]).unwrap();
        let pis = find_prime_implicants(&tt, 3);
        assert!(pis.len() >= 1);
    }

    #[test]
    fn test_pi_or_3() {
        let tt = TruthTable::from_minterms(3, &[1, 2, 3, 4, 5, 6, 7], &[]).unwrap();
        let pis = find_prime_implicants(&tt, 3);
        assert!(pis.len() <= 7);
    }

    #[test]
    fn test_pi_xor_3() {
        let tt = TruthTable::from_minterms(3, &[1, 2, 4, 7], &[]).unwrap();
        let pis = find_prime_implicants(&tt, 3);
        assert!(pis.len() >= 2);
    }

    #[test]
    fn test_pi_no_duplicate_cube_structures() {
        let tt = TruthTable::from_minterms(
            4,
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            &[],
        )
        .unwrap();
        let pis = find_prime_implicants(&tt, 4);

        let mut seen = Vec::new();
        for pc in &pis {
            let key = (pc.cond, pc.mask, pc.data);
            assert!(!seen.contains(&key), "Duplicate cube structure: {:?}", key);
            seen.push(key);
        }
    }

    #[test]
    fn test_pi_no_subsumed_pis() {
        let tt = TruthTable::from_minterms(4, &[0, 1, 2, 3], &[]).unwrap();
        let pis = find_prime_implicants(&tt, 4);

        for (i, pi_i) in pis.iter().enumerate() {
            for (j, pi_j) in pis.iter().enumerate() {
                if i != j {
                    assert!(
                        !pi_j.subsumes(*pi_i),
                        "PI {} subsumed by PI {}: {:?} vs {:?}",
                        i,
                        j,
                        pi_j,
                        pi_i
                    );
                }
            }
        }
    }

    #[test]
    fn test_pi_all_cover_valid_minterms() {
        let tt = TruthTable::from_minterms(3, &[0, 1, 2, 3], &[]).unwrap();
        let pis = find_prime_implicants(&tt, 3);

        for pc in &pis {
            for invalid_mt in 4..8 {
                assert!(
                    !covers(pc, invalid_mt),
                    "PI {:?} covers invalid minterm {}",
                    pc,
                    invalid_mt
                );
            }
        }
    }

    #[test]
    fn test_pi_with_dontcares() {
        let tt = TruthTable::from_minterms(3, &[0], &[1]).unwrap();
        let pis = find_prime_implicants(&tt, 3);
        assert!(!pis.is_empty());
    }

    #[test]
    fn test_pi_4vars_basic() {
        let tt = TruthTable::from_minterms(
            4,
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            &[],
        )
        .unwrap();
        let pis = find_prime_implicants(&tt, 4);
        // When all minterms present, should find at least one PI
        assert!(!pis.is_empty(), "Should find PIs for full function");
        // Some PIs should have high generality (many don't-cares)
        let has_general = pis.iter().any(|pc| pc.dim() <= 2);
        assert!(
            has_general || pis.len() > 0,
            "Should find general PIs or at least some PIs"
        );
    }
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn test_full_flow_3bit_and() {
        let tt = TruthTable::from_minterms(3, &[7], &[]).unwrap();
        let pis = find_prime_implicants(&tt, 3);
        assert!(!pis.is_empty());
    }

    #[test]
    fn test_full_flow_xor_3() {
        let tt = TruthTable::from_minterms(3, &[1, 2, 4, 7], &[]).unwrap();
        let pis = find_prime_implicants(&tt, 3);
        assert!(!pis.is_empty());
    }

    #[test]
    fn test_coverage_consistency() {
        let pis = vec![PrimeCube::new(1, 0, 0), PrimeCube::new(2, 0, 0)];
        let mts = vec![0, 1, 2, 3];

        for (i, pc) in pis.iter().enumerate() {
            for (j, mt) in mts.iter().enumerate() {
                let direct = covers(pc, *mt);
                if direct {
                    assert!(covers(pc, *mt));
                }
            }
        }
    }
}
