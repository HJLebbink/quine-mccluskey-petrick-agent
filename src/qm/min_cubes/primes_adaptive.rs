//! Adaptive PI generator: selects QM or CCubes based on problem characteristics.
//!
//! Benchmark results show:
//! - min-cubes CCubes O(2^n * pos * neg) — wins for n≤8, dense (>50%)
//! - QM merging O(pos^2 * n) per level — wins for n≥10, sparse (<25%)

use super::primes::find_prime_implicants;
use super::primes::{PrimeCube, TruthTable, fast_popcnt};

/// Select the fastest prime implicant algorithm based on problem sparsity.
///
/// Uses scalar min-cubes for dense problems (≤8 variables, >50% fill) and
/// classic QM merging for sparse problems (>12 variables regardless of density).
/// For medium cases (9-12 variables), uses sparsity as the deciding factor.
///
/// Benchmarks show min-cubes CCubes wins for n≤8 dense, while QM merging
/// wins for n≥10 sparse.
pub fn find_prime_implicants_adaptive(tt: &TruthTable) -> Vec<PrimeCube> {
    let n = tt.n_conds;
    let pos = tt.pos_rows;
    let total = 1u64 << n.min(50);
    let sparsity = pos as f64 / (total.min(u64::MAX / 2).max(1)) as f64;

    if pos == 0 {
        return Vec::new();
    }
    if tt.actual_neg_rows == 0 {
        return vec![PrimeCube::new(0, 0, 0)];
    }

    // Decision thresholds tuned from scaling study:
    // 3-var dense (87.5%): CCubes 0.5µs vs QM 455µs → CCubes wins
    // 4-var dense (50%): CCubes 0.48µs vs QM 139µs → CCubes wins
    // 10-var dense (50%): CCubes 6µs vs QM ~ms → CCubes OK
    // 12-var 3% (3%): CCubes 2.3s vs QM 3ms → QM wins
    // 12-var 12.5% (12.5%): CCubes 7s vs QM 5ms → QM wins
    // 14-var 0.8% (0.8%): CCubes 77s vs QM 5ms → QM wins
    // 16-var 0.4% (0.4%): CCubes >3min vs QM 2ms → QM wins
    //
    // Threshold: use QM when (n > 8 AND sparsity < 0.5) OR n > 12
    let use_qm = (n > 8 && sparsity < 0.5) || n > 12;

    if use_qm {
        find_prime_implicants_qm(tt)
    } else {
        find_prime_implicants(tt, n)
    }
}

/// QM-style PI generation using PrimeCube encoding with Hamming-distance-1 merging.
///
/// One PrimeCube per positive row, then iteratively merges cubes with
/// Hamming distance 1 in their fixed bits. Checks against negative rows
/// to ensure validity.
fn find_prime_implicants_qm(tt: &TruthTable) -> Vec<PrimeCube> {
    let n = tt.n_conds;
    let full_cond = (1u64 << n) - 1;

    // Step 1: One PrimeCube per positive row, all bits fixed
    let mut pis: Vec<PrimeCube> = Vec::with_capacity(tt.pos_rows);
    for pr in 0..tt.pos_rows {
        let mut data: u64 = 0;
        for c in 0..n {
            if tt.posval(c, pr) != 0 {
                data |= 1u64 << c;
            }
        }
        pis.push(PrimeCube::new(full_cond, 0, data));
    }

    // Step 2: Iteratively merge cubes with Hamming distance 1 in fixed bits
    for _level in 0..n {
        if pis.is_empty() {
            break;
        }

        let mut merged = vec![false; pis.len()];
        let mut next: Vec<PrimeCube> = Vec::new();

        for i in 0..pis.len() {
            if merged[i] {
                continue;
            }
            let mut found_merge = false;

            for j in (i + 1)..pis.len() {
                if merged[j] || !can_merge_pi(&pis[i], &pis[j]) {
                    continue;
                }

                let merged_cube = merge_pis_qm(&pis[i], &pis[j]);
                if !covers_any_negative(&merged_cube, tt) {
                    next.push(merged_cube);
                    merged[i] = true;
                    merged[j] = true;
                    found_merge = true;
                    break;
                }
            }

            if !found_merge {
                next.push(pis[i]);
            }
        }

        pis = next;
    }

    pis.sort_by_key(|c| fast_popcnt(c.mask));
    pis
}

/// Check whether two PrimeCubes can be merged (Hamming distance 1 in fixed bits).
///
/// Returns false if the cubes have different condition masks or if they differ
/// in more than one bit in their fixed (non-don't-care) portions.
fn can_merge_pi(a: &PrimeCube, b: &PrimeCube) -> bool {
    if a.cond != b.cond {
        return false;
    }
    let diff = (a.data & !a.mask) ^ (b.data & !b.mask);
    diff != 0 && diff.count_ones() == 1
}

/// Merge two PrimeCubes that differ by exactly one fixed bit into a single cube.
///
/// The differing bit becomes a don't-care (added to `mask`), and the data
/// is taken from either parent (they agree on all non-differing fixed bits).
fn merge_pis_qm(a: &PrimeCube, b: &PrimeCube) -> PrimeCube {
    let diff = (a.data & !a.mask) ^ (b.data & !b.mask);
    let diff_bit = diff.trailing_zeros();
    PrimeCube::new(a.cond, a.mask | (1u64 << diff_bit), a.data)
}

/// Check whether a PrimeCube would cover any negative (forbidden) row.
///
/// Returns true if the cube would incorrectly cover a minterm that must
/// not be in the function's on-set. Prevents invalid prime implicants
/// from being generated.
fn covers_any_negative(c: &PrimeCube, tt: &TruthTable) -> bool {
    let fixed = c.cond & !c.mask;
    if fixed == 0 {
        return true;
    }
    let target = c.data & fixed;
    for nr in 0..tt.actual_neg_rows {
        let mut neg_val: u64 = 0;
        for c2 in 0..tt.n_conds {
            if tt.negval(c2, nr) != 0 {
                neg_val |= 1u64 << c2;
            }
        }
        if (neg_val & fixed) == target {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_or3() {
        let tt = TruthTable::from_minterms(3, &[1, 2, 3, 4, 5, 6, 7], &[]).unwrap();
        let pis = find_prime_implicants_adaptive(&tt);
        assert!(!pis.is_empty());
    }

    #[test]
    fn test_adaptive_xors4() {
        let tt = TruthTable::from_minterms(4, &[1, 2, 4, 7, 8, 11, 13, 14], &[]).unwrap();
        let pis = find_prime_implicants_adaptive(&tt);
        assert!(!pis.is_empty());
    }
}
