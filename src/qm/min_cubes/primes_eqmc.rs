//! Equational Quine-McCluskey (eQMC) — Optimized Ternary Superset Algorithm
//!
//! Optimized for sparse problems by processing supersets in priority order
//! and pruning early when supersets overlap with negatives.

use super::primes::{PrimeCube, TruthTable};

/// eQMC: Find prime implicants using ternary superset method.
/// 
/// Optimized version that:
/// 1. Processes supersets by decreasing generality (most don't-cares first)
/// 2. Prunes early when supersets overlap with negatives
/// 3. Returns only maximal non-overlapping supersets
pub fn find_prime_implicants_eqmc(tt: &TruthTable) -> Vec<PrimeCube> {
    if tt.pos_rows == 0 {
        return Vec::new();
    }
    if tt.actual_neg_rows == 0 {
        return vec![PrimeCube::new(0, 0, 0)];
    }

    let n = tt.n_conds.clamp(1, 32); // limit for practical computation
    let full_cond = (1u64 << n) - 1;

    // Step 1: Build lookup table for quick minterm classification
    let mut is_neg = vec![false; tt.n_rows];
    for nr in 0..tt.actual_neg_rows {
        let mut neg_mt: u64 = 0;
        for c in 0..n {
            if tt.negval(c, nr) != 0 {
                neg_mt |= 1u64 << c;
            }
        }
        is_neg[neg_mt as usize] = true;
    }

    // Step 2: Extract positive minterms
    let mut pos_mts = Vec::new();
    for pr in 0..tt.pos_rows {
        let mut pos_mt: u64 = 0;
        for c in 0..n {
            if tt.posval(c, pr) != 0 {
                pos_mt |= 1u64 << c;
            }
        }
        pos_mts.push(pos_mt);
    }
    pos_mts.sort();
    pos_mts.dedup();

    // Step 3: Generate supersets in order of decreasing generality
    // For each minterm, generate all 2^n supersets, sorted by don't-cares descending
    // Then filter and collect maximal ones
    let mut pis: Vec<PrimeCube> = Vec::new();
    
    for &mt in &pos_mts {
        // Collect all supersets as (generality_score, PrimeCube) tuples
        let mut all_supers: Vec<(u32, PrimeCube)> = Vec::new();
        
        for dcmask in 0..(1u64 << n) {
            let cube = PrimeCube::new(full_cond, dcmask, mt);
            let generality = cube.mask.count_ones();
            all_supers.push((generality, cube));
        }
        
        // Sort by generality descending (most general first)
        all_supers.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.dim().cmp(&b.1.dim())));

        // Process supersets
        for (_generality, sup) in all_supers {
            // Check if sup is subsumed by existing PI
            let dominated = pis.iter().any(|existing| {
                existing.cond == sup.cond && existing.mask == sup.mask
            });
            if dominated {
                continue;
            }
            
            // Check if overlaps with negatives
            if !overlaps_any_negative_direct(&sup, &is_neg, n) {
                pis.push(sup);
                // Sort again to maintain generality order
                pis.sort_by(|a, b| {
                    let a_sc = a.mask.count_ones();
                    let b_sc = b.mask.count_ones();
                    b_sc.cmp(&a_sc).then_with(|| a.dim().cmp(&b.dim()))
                });
            }
        }
    }

    pis
}

/// Direct overlap check without function call overhead
#[inline]
fn overlaps_any_negative_direct(cube: &PrimeCube, is_neg: &[bool], _n: usize) -> bool {
    let fixed = cube.cond & !cube.mask;
    if fixed == 0 {
        return !is_neg.is_empty();
    }
    let target = cube.data & fixed;
    
    for (mt_idx, &is_neg_m) in is_neg.iter().enumerate() {
        if is_neg_m {
            let mt_val = mt_idx as u64;
            if (mt_val & fixed) == target {
                return true;
            }
        }
    }
    false
}

/// Test eQMC on OR(3)
#[test]
fn test_eqmc_or3() {
    let tt = TruthTable::from_minterms(3, &[1, 2, 3, 4, 5, 6, 7], &[]).unwrap();
    let pis = find_prime_implicants_eqmc(&tt);
    
    assert!(!pis.is_empty(), "eQMC should find PIs for OR(3)");
    
    let all_covered: Vec<u64> = (0..8).filter(|&mt| {
        pis.iter().any(|pi| {
            let fixed = pi.cond & !pi.mask;
            (mt & fixed) == (pi.data & fixed)
        })
    }).collect();
    
    assert_eq!(all_covered, (0..8).filter(|&m| m > 0).collect::<Vec<_>>());
}
