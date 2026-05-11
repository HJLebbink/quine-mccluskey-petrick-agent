//! Prime implicant generation using the min-cubes algorithm
//!
//! Implements the algorithm from "Minimize Cubes" (C++ reference).
//! Iterates through condition combinations and verifies each candidate.

use crate::qm::encoding::{BitOps, MintermEncoding};
use crate::qm::implicant::{BitState, Implicant};
use smallvec::smallvec;

/// Maximum number of conditions supported by PrimeCube
pub const MAX_CONDITIONS: usize = 64;

// ===========================================================================
// PopCnt-based pruning utilities
// ===========================================================================

/// Fast PopCnt: count don't-care bits (mask popcnt) as generality score
#[inline]
pub fn fast_popcnt(v: u64) -> u64 {
    (v as u64).count_ones() as u64
}

/// Check if candidate PI is definitely subsumed by existing PIs (PopCnt heuristic)
#[inline]
fn popcnt_maybe_subsumed(candidate: &PrimeCube, existing: &[PrimeCube]) -> bool {
    let c_generality = fast_popcnt(candidate.mask >> 1);
    for e in existing {
        let e_generality = fast_popcnt(e.mask >> 1);
        if e_generality >= c_generality {
            if e.subsumes(*candidate) {
                return true;
            }
        }
    }
    false
}

/// Sort prime implicants by generality: unconditional don't-cares first, then highest don't-care count.
///
/// Primes with more don't-care conditions (higher dimension reduction) are placed
/// first, which optimizes the set cover solver's greedy selection.
#[inline]
pub fn sort_by_generality_pis(pis: &mut [PrimeCube]) {
    pis.sort_by(|a, b| {
        a.dim()
            .cmp(&b.dim())
            .then(fast_popcnt(b.mask).cmp(&fast_popcnt(a.mask)))
    });
}

/// Prime implicant with three bitfields to distinguish **used** from **don't-care**.
///
/// Each condition position can be in one of four states:
///   - cond=0: unconditional don't-care (PI doesn't use this condition)
///   - cond=1, mask=1: used but value is don't-care (matches 0 and 1)
///   - cond=1, mask=0, data=1: used, must be true (1)
///   - cond=1, mask=0, data=0: used, must be false (0)
///
/// **Subsumption** (self subsumes other): for every condition where other.cond=1:
///   self.cond=0 (unconditional don't-care) OR self.mask=1 (don't-care value) OR (self.data==other.data)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimeCube {
    /// Condition usage: 1=PI cares about this condition, 0=unconditional don't-care
    pub cond: u64,
    /// Don't-care mask (among used conditions): 1=don't-care, 0=fixed
    pub mask: u64,
    /// Fixed values: 1=true, 0=false (where cond=1, mask=0)
    pub data: u64,
}

impl PrimeCube {
    /// Create a new prime cube.
    ///
    /// # Parameters
    /// - `cond`: Bitmask indicating which conditions the cube cares about (1 = cares)
    /// - `mask`: Among cared-about conditions, which are don't-cares (1 = don't-care)
    /// - `data`: Fixed values for conditions where cond=1 and mask=0
    #[inline]
    pub const fn new(cond: u64, mask: u64, data: u64) -> Self {
        Self { cond, mask, data }
    }

    /// Number of fixed (non-don't-care) conditions in this cube.
    ///
    /// A higher dimension means a more specific (less general) implicant.
    #[inline]
    pub fn dim(&self) -> u32 {
        (self.cond & !self.mask).count_ones()
    }

    /// Total number of conditions the cube uses (fixed + don't-care).
    #[inline]
    pub fn conditions_used(&self) -> u32 {
        self.cond.count_ones()
    }

    /// Check whether this cube subsumes another.
    ///
    /// A cube subsumes another when it covers a superset of minterms.
    /// For every condition where `other.cond = 1`:
    /// - `self.cond = 0` means this cube doesn't care unconditionally, OR
    /// - `self.mask = 1` means this cube has a don't-care value, OR
    /// - `self.data == other.data` means both have the same fixed value
    #[inline]
    pub fn subsumes(&self, other: Self) -> bool {
        let self_fixed = self.cond & !self.mask;
        let other_fixed = other.cond & !other.mask;
        let shared_fixed = self_fixed & other_fixed;
        if ((self.data & shared_fixed) ^ (other.data & shared_fixed)) != 0 {
            return false;
        }
        if self_fixed & !other_fixed != 0 {
            return false;
        }
        true
    }

    /// Check if this is a universal cube (unconditional don't-care on all conditions).
    ///
    /// A universal cube has `cond = 0`, meaning it matches every minterm.
    /// This represents the constant-1 function.
    #[inline]
    pub fn is_universal(&self) -> bool {
        self.cond == 0
    }
}

/// Truth table for min-cubes
#[derive(Debug, Clone)]
pub struct TruthTable {
    pub n_conds: usize,
    pub n_rows: usize,
    pub posmat: Vec<u8>,
    pub negmat: Vec<u8>,
    pub pos_rows: usize,
    pub neg_rows: usize,
    pub actual_neg_rows: usize,
}

impl TruthTable {
    /// Build a truth table from minterms and don't-care values.
    ///
    /// Minterms are positive rows that must be covered. Don't-cares are
    /// treated as positive rows to help merge prime implicants but are not
    /// checked for coverage validity. True negative rows are those that
    /// must never be covered by any prime implicant.
    ///
    /// Returns `None` if `n_conds` is 0 or exceeds `MAX_CONDITIONS`.
    pub fn from_minterms(n_conds: usize, minterms: &[u64], dont_cares: &[u64]) -> Option<Self> {
        if n_conds == 0 || n_conds > MAX_CONDITIONS {
            return None;
        }
        let n_rows = 1usize << n_conds;
        let mut mt_lookup = vec![false; n_rows];
        let mut dc_lookup = vec![false; n_rows];
        for &mt in minterms {
            if (mt as usize) < n_rows {
                mt_lookup[mt as usize] = true;
            }
        }
        for &dc in dont_cares {
            if (dc as usize) < n_rows {
                dc_lookup[dc as usize] = true;
            }
        }

        // Count rows needed
        // Pos rows: minterms + dont-cares (dont-cares help merge PIs, but coverage only checks minterms)
        // Neg rows: true negatives (must never be covered)
        let mut pos_rows_count = 0;
        let mut real_neg_count = 0usize;
        for row in 0..n_rows {
            if !mt_lookup[row] && !dc_lookup[row] {
                real_neg_count += 1;
            } else {
                pos_rows_count += 1;
            }
        }
        let neg_rows = real_neg_count.max(1);

        // Build neg_rows lookup for fast filtering
        let neg_rows_map: Vec<bool> = (0..n_rows)
            .map(|row| !mt_lookup[row] && !dc_lookup[row])
            .collect();

        let pos_size = n_conds * pos_rows_count;
        let neg_size = n_conds * neg_rows;
        let mut posmat = vec![0u8; pos_size];
        let mut negmat = vec![0u8; neg_size];
        let mut pos_row = 0usize;
        let mut neg_row = 0usize;

        for row in 0..n_rows {
            // Include minterms AND dont-cares in pos_rows (dont-cares help PI merging)
            if neg_rows_map[row] {
                continue;
            } // skip true negatives
            for c in 0..n_conds {
                let val = if (row >> c) & 1 != 0 { 1u8 } else { 0u8 };
                posmat[c * pos_rows_count + pos_row] = val;
            }
            pos_row += 1;
        }

        for row in 0..n_rows {
            if mt_lookup[row] || dc_lookup[row] {
                continue;
            } // skip non-negative rows
            for c in 0..n_conds {
                let val = if (row >> c) & 1 != 0 { 1u8 } else { 0u8 };
                negmat[c * neg_rows + neg_row] = val;
            }
            neg_row += 1;
        }

        Some(Self {
            n_conds,
            n_rows,
            posmat,
            negmat,
            pos_rows: pos_rows_count,
            neg_rows,
            actual_neg_rows: real_neg_count,
        })
    }

    /// Get the value of condition `c` at positive row `r`.
    #[inline]
    pub fn posval(&self, c: usize, r: usize) -> u8 {
        self.posmat[c * self.pos_rows + r]
    }
    /// Get the value of condition `c` at negative row (forbidden) `r`.
    #[inline]
    pub fn negval(&self, c: usize, r: usize) -> u8 {
        self.negmat[c * self.neg_rows + r]
    }
}

/// Compute signature for a positive row under given conditions (superseded by batch_signatures)
#[inline]
#[allow(dead_code)]
fn pos_signature(tt: &TruthTable, tempk: &[u32], row: usize) -> u64 {
    let mut sig = 0u64;
    for (j, &c) in tempk.iter().enumerate() {
        if tt.posval(c as usize, row) != 0 {
            sig |= 1u64 << j;
        }
    }
    sig
}

/// Compute signature for a negative row under given conditions
#[inline]
fn neg_signature(tt: &TruthTable, tempk: &[u32], row_idx: usize) -> u64 {
    let mut sig = 0u64;
    for (j, &c) in tempk.iter().enumerate() {
        if tt.negval(c as usize, row_idx) != 0 {
            sig |= 1u64 << j;
        }
    }
    sig
}

/// Check if a signature appears in any negative row
fn sig_not_in_negatives(tt: &TruthTable, tempk: &[u32], sig: u64) -> bool {
    for nr in 0..tt.neg_rows {
        if neg_signature(tt, tempk, nr) == sig {
            return false;
        }
    }
    true
}

/// Build the 3-field PrimeCube for a signature under given conditions
fn build_cube(_tt: &TruthTable, tempk: &[u32], sig: u64) -> PrimeCube {
    let mut cond = 0u64;
    let mut data = 0u64;
    for (j, &c) in tempk.iter().enumerate() {
        cond |= 1u64 << c;
        if (sig >> (j as u32)) & 1 != 0 {
            data |= 1u64 << c;
        }
    }
    PrimeCube::new(cond, 0, data)
}

/// Find all prime implicants using the scalar min-cubes algorithm.
///
/// Iterates through condition combinations of increasing size (k = 1..n),
/// builds candidate cubes from valid signatures, and filters out subsumed ones.
/// Returns cubes sorted by generality (most general first).
///
/// - Returns empty vector if there are no positive rows
/// - Returns a single universal PI `PrimeCube(0,0,0)` if there are no negative rows
pub fn find_prime_implicants(tt: &TruthTable, pi_depth: usize) -> Vec<PrimeCube> {
    if tt.pos_rows == 0 {
        return Vec::new();
    }
    // When there are zero actual negative rows (all-ones function),
    // a single universal PI subsumes everything - no conditions needed.
    if tt.actual_neg_rows == 0 {
        return vec![PrimeCube::new(0, 0, 0)];
    }
    let max_k = pi_depth.min(tt.n_conds);
    let mut all_pis: Vec<PrimeCube> = Vec::new();
    let _row_stride = tt.pos_rows;
    let mut seen_buf = Vec::with_capacity(tt.pos_rows);
    let mut valid_buf = Vec::with_capacity(tt.pos_rows);
    let mut buf = core::mem::MaybeUninit::<[u32; 16]>::uninit();

    for k in 1..=max_k {
        use super::comb::CombinationIterator;
        let mut iter = match CombinationIterator::new(tt.n_conds, k) {
            Some(i) => i,
            None => continue,
        };

        loop {
            iter.indices(&mut buf);
            let tempk = unsafe { buf.assume_init() };
            let tempk_slice = &tempk[..k];

            seen_buf.clear();
            valid_buf.clear();

            // Batch signature computation: process 8 positive rows at a time
            let mut row = 0usize;
            while row < tt.pos_rows {
                let batch_end = (row + 8).min(tt.pos_rows);
                let batch_size = batch_end - row;
                let mut results = [0u64; 8];
                for s in 0..batch_size {
                    let r = row + s;
                    let mut sig = 0u64;
                    for j in 0..k {
                        let c = tempk[j] as usize;
                        let val = tt.posval(c, r);
                        if val != 0 {
                            sig |= 1u64 << j;
                        }
                    }
                    results[s] = sig;
                }
                for s in 0..batch_size {
                    let sig = results[s];
                    if !seen_buf.contains(&sig) {
                        seen_buf.push(sig);
                    }
                }
                row = batch_end;
            }

            // Batch negative row check: check each unique signature against neg rows
            for &sig in &seen_buf {
                if sig_not_in_negatives(tt, tempk_slice, sig) {
                    valid_buf.push(sig);
                }
            }

            // Build PrimeCubes from valid signatures
            let mut candidates: Vec<PrimeCube> = Vec::with_capacity(valid_buf.len());
            for &sig in &valid_buf {
                candidates.push(build_cube(tt, tempk_slice, sig));
            }

            // PopCnt-based subsumption pre-filter
            let filtered: Vec<_> = candidates
                .into_iter()
                .filter(|c| {
                    if popcnt_maybe_subsumed(c, &all_pis) {
                        return false;
                    }
                    !all_pis.iter().any(|e| e.subsumes(*c))
                })
                .collect();

            all_pis.extend(filtered);

            if !iter.next() {
                break;
            }
        }
    }

    // Apply generality-based sorting for optimal set cover ordering
    sort_by_generality_pis(&mut all_pis);
    all_pis
}

// ---------------------------------------------------------------------------
// Bridge: PrimeCube → Implicant<E> for integration with QM solver pipeline
// ---------------------------------------------------------------------------

/// Convert PrimeCubes to Implicant<E>[] for Petrick's method.
///
/// Translates each cube's 3-field encoding (cond, mask, data) into a
/// vector of BitState values compatible with the QM solver pipeline.
pub fn prime_cubes_to_implicants<E: MintermEncoding>(
    cubes: &[PrimeCube],
    variables: usize,
) -> Vec<Implicant<E>> {
    cubes
        .iter()
        .map(|cube| {
            let mut bits = smallvec![];
            for i in (0..variables).rev() {
                let is_used = (cube.cond >> i) & 1 == 1;

                let state = if !is_used {
                    // Unconditional don't-care: any value of this variable is fine
                    BitState::DontCare
                } else if (cube.mask >> i) & 1 == 1 {
                    BitState::DontCare
                } else if (cube.data >> i) & 1 == 1 {
                    BitState::One
                } else {
                    BitState::Zero
                };
                bits.push(state);
            }
            Implicant {
                bits,
                covered_minterms: Vec::new(),
            }
        })
        .collect()
}

/// Populate the `covered_minterms` field for Implicants converted from PrimeCubes.
///
/// Checks each minterm against the implicant's bits to build the coverage
/// list. Must be called after `prime_cubes_to_implicants` because those
/// Implicants start with empty `covered_minterms`.
pub fn populate_covered_minterms_u64<E: MintermEncoding>(
    pis: &mut [Implicant<E>],
    all_minterms: &[E::Value],
    n_vars: usize,
) {
    for pi in pis.iter_mut() {
        let mut covered = Vec::with_capacity(all_minterms.len());
        for mt in all_minterms {
            let raw = mt.to_u64();
            if covers_implicant_u64(&pi.bits, raw, n_vars) {
                covered.push(*mt);
            }
        }
        pi.covered_minterms = covered;
    }
}

/// Check if an Implicant covers a raw u64 minterm by matching bits
fn covers_implicant_u64(bits: &[BitState], raw_minterm: u64, n_vars: usize) -> bool {
    for (i, &state) in bits.iter().take(n_vars).enumerate() {
        if state == BitState::DontCare {
            continue;
        }
        let expected_u64 = if state == BitState::One { 1u64 } else { 0u64 };
        let actual = (raw_minterm >> i) & 1;
        if actual != expected_u64 {
            return false;
        }
    }
    true
}

/// Build a bit-packed coverage matrix mapping each prime implicant to its covered minterms.
///
/// For each PI, checks if `(pi.data & fixed_mask) == (minterm & fixed_mask)` for
/// every minterm. Returns a Vec<u64> where each PI's coverage spans `div_ceil(m.len(), 64)` words.
pub fn build_coverage_matrix(pis: &[PrimeCube], minterms: &[u64]) -> Vec<u64> {
    if pis.is_empty() || minterms.is_empty() {
        return Vec::new();
    }
    let n = minterms.len();
    let u64s = n.div_ceil(64);
    let mut coverage = vec![0u64; pis.len() * u64s];

    for (pi_idx, pi) in pis.iter().enumerate() {
        let base = pi_idx * u64s;
        let match_mask = pi.cond & !pi.mask;
        let pi_masked = pi.data & match_mask;
        for (mt_idx, mt) in minterms.iter().enumerate() {
            if pi_masked == (*mt & match_mask) {
                coverage[base + mt_idx / 64] |= 1u64 << (mt_idx % 64);
            }
        }
    }
    coverage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_subsumes_all() {
        let univ = PrimeCube::new(0, 0, 0);
        let specific = PrimeCube::new(3, 0, 0); // uses cond 0,1 fixed=0
        assert!(univ.subsumes(specific));
    }

    #[test]
    fn test_dim() {
        let pc = PrimeCube::new(7, 1, 5); // cond 0,1,2; mask=1 (cond 0 is DC); data=5 (0b101)
        assert_eq!(pc.dim(), 2); // cond 1 fixed=1 (data 5=0b101, mask=001, fixed mask=7&~1=6, data&6=4(0b100), wait...
        // cond=0b111, mask=0b001, data=0b101
        // cond & !mask = 0b111 & 0b110 = 0b110 (cond 1 and 2 are fixed)
        assert_eq!(pc.dim(), 2);
    }

    #[test]
    fn test_subsumes_basic() {
        // X0 : cond=1, mask=0, data=0
        let x0 = PrimeCube::new(1, 0, 0);
        // 0X would be cond=2, mask=1, data=0 (but we don't create 0X from tempk)
        // Let's use: 0 = fixed cond 0, 1 = don't care cond 0 but we can't have that with mask=0
        // X0 subsumes nothing (it's specific)
        let a = PrimeCube::new(1, 0, 0); // same as x0
        assert!(x0.subsumes(a)); // same PI

        let b = PrimeCube::new(3, 0, 0); // cond 0,1 fixed=0
        // x0.subsumes(b)? b.cond=3, shared=3&1=1, shared_fixed=1&1=1
        // x0.data&1 = 0, b.data&1 = 0 → match → true
        assert!(x0.subsumes(b)); // X0 subsumes 00 (X is unconditional don't-care vs 0 is used)
    }

    #[test]
    fn test_truth_table_basic() {
        let tt = TruthTable::from_minterms(3, &[0, 1], &[]).unwrap();
        assert_eq!(tt.n_conds, 3);
        assert_eq!(tt.pos_rows, 2);
        assert_eq!(tt.neg_rows, 6); // 8 - 2 = 6
    }

    #[test]
    fn test_coverage_simple() {
        let pis = vec![
            PrimeCube::new(1, 0, 0), // cond 0, fixed 0
        ];
        let mts = vec![0, 1, 4, 5]; // minterm 0 has bit0=0, covers it
        let cov = build_coverage_matrix(&pis, &mts);
        // PI covers minterms where bit 0 is 0: 0 (0b000), 4 (0b100) → indices 0,2
        assert_eq!(cov.len(), 1); // 4 minterms → 1 u64
        assert_eq!(cov[0], 0b0101); // bits 0 and 2 set
    }

    #[test]
    fn test_min_cubes_3bit() {
        // f(A,B,C) = Σ(1, 7) → rows 001(1) and 111(7)
        // Only share condition 2 (C) = 1, but XX1 would cover rows 1,3,5,7
        // which overlaps with neg rows 3,5 → XX1 not valid
        // The PIs are just the individual minterms
        let tt = TruthTable::from_minterms(3, &[1, 7], &[]).unwrap();
        assert_eq!(tt.n_conds, 3);
        assert_eq!(tt.pos_rows, 2); // rows 1 and 7
        assert_eq!(tt.neg_rows, 6); // rows 0, 2, 3, 4, 5, 6

        let pis = find_prime_implicants(&tt, 3);
        // 001 and 111 should both be found (individual minterms)
        // 001: cond bits {0}=1, data=1 (C=1)
        // 111: cond bits {0,1,2}=111 at row 7, but sig 111 overlaps neg
        // For row 7: bits {0,1,2}=111=7; neg row 3(011) → bits {0,1,2}=011≠111, neg row 5(101)→101≠111, ok
        // Wait, neg rows include 3 (011). 3 has bits {0,1,2}=011=3. 3≠7 so no overlap.
        // So 111 should be valid.
        assert_eq!(pis.len(), 2);
    }

    #[test]
    fn test_min_cubes_with_dontcares() {
        // f(A,B,C) = Σ(0, 1, 4, 5) → B=0 covers all
        // This function: when B=0 → output=1, when B=1 → output=0
        // So the function is simply NOT B
        let tt = TruthTable::from_minterms(3, &[0, 1, 4, 5], &[]).unwrap();
        assert_eq!(tt.pos_rows, 4); // rows 0, 1, 4, 5
        assert_eq!(tt.neg_rows, 4); // rows 2, 3, 6, 7

        let pis = find_prime_implicants(&tt, 3);
        // X0_ (B=0): cond bits {1}=1, data=0 → covers rows 0,1,4,5
        // Check: neg rows 2(010)->B=1, 3(011)->B=1, 6(110)->B=1, 7(111)->B=1
        // All neg rows have B=1, so X0_ doesn't overlap → valid!
        let found_x0: bool = pis
            .iter()
            .any(|pi| pi.cond == 2 && pi.data == 0 && pi.mask == 0);
        assert!(
            found_x0,
            "min_cubes should produce X0_ for B=0 function: found {:?} PIs",
            pis.iter()
                .map(|p| (p.cond, p.mask, p.data, p.dim()))
                .collect::<Vec<_>>()
        );
    }

    // ==========================================================================
    // Phase 9: Bit-tricks unit tests
    // ==========================================================================

    #[test]
    fn test_popcnt_maybe_subsumed_with_universal() {
        let existing = vec![PrimeCube::new(0, 0, 0)];
        let candidate = PrimeCube::new(3, 0, 0);
        assert!(popcnt_maybe_subsumed(&candidate, &existing));
        let empty: Vec<PrimeCube> = vec![];
        assert!(!popcnt_maybe_subsumed(&candidate, &empty));
    }

    #[test]
    fn test_popcnt_maybe_subsumed_no_conflict() {
        let existing = vec![PrimeCube::new(2, 0, 0)];
        let candidate = PrimeCube::new(1, 0, 0);
        assert!(!existing[0].subsumes(candidate));
        assert!(!popcnt_maybe_subsumed(&candidate, &existing));
    }

    #[test]
    fn test_sort_by_generality_pis() {
        let mut pis = vec![
            PrimeCube::new(3, 0, 0),
            PrimeCube::new(7, 3, 1),
            PrimeCube::new(0, 0, 0),
        ];
        sort_by_generality_pis(&mut pis);
        assert!(pis[0].is_universal(), "universal PI should be first");
    }

    #[test]
    fn test_coverage_matrix_with_div_ceil() {
        // 65 minterms need 2 u64 slots (div_ceil(65, 64) = 2)
        // 2 PIs × 2 u64s = 4 total entries
        let pis = vec![PrimeCube::new(1, 0, 0), PrimeCube::new(2, 0, 0)];
        let mts: Vec<u64> = (0..65).collect();
        let cov = build_coverage_matrix(&pis, &mts);
        assert_eq!(cov.len(), 4);
        assert!(
            cov[0] != 0 || cov[1] != 0,
            "coverage should not be all zeros"
        );
    }

    #[test]
    fn test_coverage_matrix_edge_cases() {
        let pis = vec![PrimeCube::new(1, 0, 0)];
        let empty_mts: Vec<u64> = vec![];
        let cov = build_coverage_matrix(&pis, &empty_mts);
        assert!(
            cov.is_empty(),
            "empty minterms should return empty coverage"
        );
        let empty_pis: Vec<PrimeCube> = vec![];
        let cov2 = build_coverage_matrix(&empty_pis, &[42u64]);
        assert!(cov2.is_empty(), "empty pis should return empty coverage");
        let empty_both: Vec<u64> = vec![];
        let cov3 = build_coverage_matrix(&empty_pis, &empty_both);
        assert!(cov3.is_empty());
    }

    #[test]
    fn test_build_coverage_matrix_correctness() {
        // PI: cond=1 (bit 0), data=0 → covers minterms with bit 0 = 0
        // That's minterms {0, 2, 4, 6, ...}
        let pis = vec![PrimeCube::new(1, 0, 0)];
        let mts: Vec<u64> = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let cov = build_coverage_matrix(&pis, &mts);
        // Even indices (0,2,4,6) → bits 0,2,4,6 set
        assert_eq!(cov[0], 0b01010101);
    }

    #[test]
    fn test_build_cube_zero_sig() {
        let tt = TruthTable::from_minterms(3, &[0, 1], &[]).unwrap();
        let tempk = [0, 1];
        let cube = build_cube(&tt, &tempk, 0);
        // sig=0, tempk=[0,1] → cond has bits 0,1 set; data=0
        assert_eq!(cube.cond, 3);
        assert_eq!(cube.data, 0);
        assert_eq!(cube.mask, 0);
    }

    #[test]
    fn test_find_prime_implicants_empty_truth_table() {
        let tt = TruthTable::from_minterms(3, &[], &[]).unwrap();
        let pis = find_prime_implicants(&tt, 3);
        assert!(pis.is_empty());
    }

    #[test]
    fn test_find_prime_implicants_universal() {
        // All minterms: should produce universal PI (cond=0, no conditions needed)
        let tt = TruthTable::from_minterms(2, &[0, 1, 2, 3], &[]).unwrap();
        let pis = find_prime_implicants(&tt, 2);
        let has_universal: bool = pis.iter().any(|p| p.is_universal());
        assert!(has_universal, "all-ones function should have universal PI");
    }

    #[test]
    fn test_prime_cubes_to_implicants_conversion() {
        use crate::qm::implicant::BitState;
        let cubes = vec![
            PrimeCube::new(3, 0, 3), // cond=3(data=3) → A=1, B=1
            PrimeCube::new(1, 1, 0), // cond=1(mask=1) → A=X
        ];
        let pis = prime_cubes_to_implicants::<crate::qm::encoding::Enc16>(&cubes, 2);
        assert_eq!(pis.len(), 2);
        // First: A=One, B=One
        assert_eq!(pis[0].bits[0], BitState::One);
        assert_eq!(pis[0].bits[1], BitState::One);
        // Second: A=DontCare, B=DontCare (cond=0 for both bits)
        assert_eq!(pis[1].bits[0], BitState::DontCare);
        assert_eq!(pis[1].bits[1], BitState::DontCare);
    }

    #[test]
    fn test_coverage_matrix_large() {
        // Test with >64 minterms to verify multi-u64 coverage
        let pis = vec![PrimeCube::new(1, 0, 0)];
        let mts: Vec<u64> = (0..128).collect();
        let cov = build_coverage_matrix(&pis, &mts);
        assert_eq!(cov.len(), 2);
        // PI covers even minterms → check a few known ranges
        // Minterm 0,2,4,6 are in first u64 → bits 0,2,4,6 set = 0b01010101
        assert_eq!((cov[0] & 0xFF), 0b01010101);
    }
}
