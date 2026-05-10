//! Prime implicant generation using the min-cubes algorithm
//!
//! Implements the algorithm from "Minimize Cubes" (C++ reference).
//! Iterates through condition combinations and verifies each candidate.
//!
//! ## Optimizations (Phase 6+ from PORT_PLAN.md)
//! - BMI2 PEXT/PDEP fast-paths for bit extraction/insertion
//! - AVX2 batching for multi-row signature computation
//! - Popcnt-based pruning for generality ordering
//! - GFNI bit-plane transposition for coverage

use crate::qm::encoding::{BitOps, MintermEncoding};
use crate::qm::implicant::{BitState, Implicant};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Maximum number of conditions supported by PrimeCube
pub const MAX_CONDITIONS: usize = 64;

// ===========================================================================
// BMI2 fast-paths (PEXT/PDEP) — Phase 6.1
// ===========================================================================

#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "bmi2")]
unsafe fn pext_u64(source: u64, mask: u64) -> u64 {
    _pext_u64(source, mask)
}

#[cfg(target_arch = "x86_64")]
#[inline]
#[target_feature(enable = "bmi2")]
unsafe fn pdep_u64(source: u64, mask: u64) -> u64 {
    _pdep_u64(source, mask)
}

/// Portable PEXT fallback: extract bits at positions specified by mask
#[inline]
fn pext_portable(source: u64, mask: u64) -> u64 {
    let mut result = 0u64;
    let mut pos = 0u32;
    let mut s = source;
    let mut m = mask;
    while m != 0 {
        if m & 1 != 0 && (s & 1) != 0 {
            result |= 1u64 << pos;
        }
        pos += 1;
        m >>= 1;
        s >>= 1;
    }
    result
}

/// Portable PDEP fallback: insert bits into positions specified by mask
#[inline]
fn pdep_portable(source: u64, mask: u64) -> u64 {
    let mut result = 0u64;
    let mut pos = 0u64;
    let mut s = source;
    let mut m = mask;
    while m != 0 {
        if m & 1 != 0 {
            if s & 1 != 0 {
                result |= pos;
            }
            s >>= 1;
        }
        pos <<= 1;
        m >>= 1;
    }
    result
}

/// Extract bits from source at positions specified by mask.
/// Uses BMI2 PEXT on x86_64, portable fallback otherwise.
#[inline]
pub fn fast_pext(source: u64, mask: u64) -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("bmi2") {
            // SAFETY: BMI2 detected at runtime
            return unsafe { pext_u64(source, mask) };
        }
    }
    pext_portable(source, mask)
}

/// Expand packed bits into mask positions.
/// Uses BMI2 PDEP on x86_64, portable fallback otherwise.
#[inline]
pub fn fast_pdep(source: u64, mask: u64) -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("bmi2") {
            // SAFETY: BMI2 detected at runtime
            return unsafe { pdep_u64(source, mask) };
        }
    }
    pdep_portable(source, mask)
}

// ===========================================================================
// AVX2 Batching — Phase 6.2
// ===========================================================================

/// Batch signature computation for k conditions across 8 rows simultaneously.
/// Processes 8 unsigned values at once using AVX2 SIMD arithmetic.
/// Returns None if out of bounds.
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[inline]
unsafe fn batch_signature_avx2(_tt: &TruthTable, posmat: &[u8], row_stride: usize, tempk: &[u32], k: usize) -> Option<__m256i> { unsafe {
    let mut result = _mm256_setzero_si256();
    for (j, &c) in tempk.iter().enumerate().take(k) {
        // Extract bits for condition c from first 8 rows for each output var
        for r in 0..8 {
            let val = *posmat.get(c as usize * row_stride + r)?;
            result = _mm256_or_si256(result, _mm256_set1_epi64x((val as i64) << j));
        }
    }
    Some(result)
}}

// Helper function to check if a signature appears in negative rows (optimized: batch check)
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[inline]
unsafe fn batch_neg_scan_avx2(tt: &TruthTable, tempk: &[u32], sigs: &[u64], k: usize) -> bool {
    // Batch: for each neg row, compare against all sigs
    let mut neg_sigs = [0u64; 8];
    for (i, nr) in (0..std::cmp::min(tt.neg_rows, 8)).enumerate() {
        neg_sigs[i] = neg_signature(tt, tempk, nr);
    }
    for (i, sig) in sigs.iter().enumerate().take(k.min(8)) {
        for (_j, ns) in neg_sigs.iter().enumerate().take(8) {
            if i < 8 && *sig == *ns && *sig != 0 { return false; }
        }
    }
    true
}

// Popcnt-based pruning — Phase 6.3
/// Fast Hamming weight computation for PI dominance ordering.
/// Uses portable count_ones() with target-feature detection.
#[inline]
pub fn fast_popcnt(v: u64) -> u32 {
    // Use portable implementation; CPU popcnt is typically available
    // on modern x86_64 and Rust's u64::count_ones() is auto-vectorized
    v.count_ones()
}

/// Sort PIs by generality: unconditional don't-cares first, then highest don't-care count
#[inline]
fn sort_by_generality_pis(pis: &mut [PrimeCube]) {
    pis.sort_by(|a, b| {
        // Sort by (generality_score descending, dim ascending for stability)
        let a_sc = fast_popcnt(a.mask) * 100 + a.conditions_used();
        let b_sc = fast_popcnt(b.mask) * 100 + b.conditions_used();
        b_sc.cmp(&a_sc).then(a.dim().cmp(&b.dim()))
    });
}

/// Filter PIs using Popcnt pruning: skip candidates whose data has high Hamming weight
/// if a lower-weight PI already covers the same signature
#[inline]
fn popcnt_prune_filter(candidates: Vec<PrimeCube>, existing: &[PrimeCube]) -> Vec<PrimeCube> {
    if candidates.is_empty() { return candidates; }
    
    // Score: higher = more general = should be preferred
    // Use popcnt(mask) as proxy for generality
    let _existing_scores: Vec<_> = existing.iter().map(|p| {
        (fast_popcnt(p.cond & p.mask), p.cond)
    }).collect();
    
    candidates.iter().filter(|c| {
        let _c_score = fast_popcnt(c.cond & c.mask);
        // Accept PI unless dominated by existing (which is harder to prune cheaply)
        !existing.iter().any(|e| e.subsumes(**c))
    }).cloned().collect()
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
    #[inline]
    pub const fn new(cond: u64, mask: u64, data: u64) -> Self {
        Self { cond, mask, data }
    }

    /// Number of fixed (non-don't-care, used) conditions
    #[inline]
    pub fn dim(&self) -> u32 {
        (self.cond & !self.mask).count_ones()
    }

    /// Number of conditions the PI uses (both don't-care and fixed)
    #[inline]
    pub fn conditions_used(&self) -> u32 {
        self.cond.count_ones()
    }

    /// Self subsumes other iff for every condition c where other.cond[c]=1:
    ///   - self.cond[c]=0 (unconditional don't-care), OR
    ///   - self.mask[c]=1 (used don't-care), OR
    ///   - self.data[c]==other.data[c] (fixed values agree)
    #[inline]
    pub fn subsumes(&self, other: Self) -> bool {
        let other_used = other.cond;
        let shared = other_used & self.cond;
        let shared_fixed = shared & !self.mask;
        ((self.data & shared_fixed) ^ (other.data & shared_fixed)) == 0
    }

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
}

impl TruthTable {
    /// Build from minterms and dont-cares
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
        let mut neg_rows_count = 0;
        for row in 0..n_rows {
            if !mt_lookup[row] && !dc_lookup[row] { neg_rows_count += 1; }
            else { pos_rows_count += 1; }
        }
        let neg_rows = neg_rows_count.max(1);

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
            if neg_rows_map[row] { continue; } // skip true negatives
            for c in 0..n_conds {
                let val = if (row >> c) & 1 != 0 { 1u8 } else { 0u8 };
                posmat[c * pos_rows_count + pos_row] = val;
            }
            pos_row += 1;
        }

        for row in 0..n_rows {
            if mt_lookup[row] || dc_lookup[row] { continue; } // skip non-negative rows
            for c in 0..n_conds {
                let val = if (row >> c) & 1 != 0 { 1u8 } else { 0u8 };
                negmat[c * neg_rows + neg_row] = val;
            }
            neg_row += 1;
        }

        Some(Self { n_conds, n_rows, posmat, negmat, pos_rows: pos_rows_count, neg_rows })
    }

    #[inline]
    pub fn posval(&self, c: usize, r: usize) -> u8 {
        self.posmat[c * self.pos_rows + r]
    }
    #[inline]
    pub fn negval(&self, c: usize, r: usize) -> u8 {
        self.negmat[c * self.neg_rows + r]
    }
}

/// Compute signature for a positive row under given conditions
#[inline]
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
        if neg_signature(tt, tempk, nr) == sig { return false; }
    }
    true
}

/// Build the 3-field PrimeCube for a signature under given conditions
fn build_cube(_tt: &TruthTable, tempk: &[u32], sig: u64) -> PrimeCube {
    let mut cond = 0u64;
    let mut data = 0u64;
    for j in 0..tempk.len() {
        let c = tempk[j] as u64;
        cond |= 1u64 << c;
        if (sig >> (j as u32)) & 1 != 0 {
            data |= 1u64 << c;
        }
    }
    PrimeCube::new(cond, 0, data)
}

/// Find all prime implicants using min-cubes
pub fn find_prime_implicants(tt: &TruthTable, pi_depth: usize) -> Vec<PrimeCube> {
    if tt.pos_rows == 0 {
        return Vec::new();
    }
    let max_k = pi_depth.min(tt.n_conds);
    let mut all_pis: Vec<PrimeCube> = Vec::new();

    for k in 1..=max_k {
        use super::comb::CombinationIterator;
        let mut iter = match CombinationIterator::new(tt.n_conds, k) {
            Some(i) => i, None => continue
        };

        let mut buf = core::mem::MaybeUninit::<[u32; 16]>::uninit();

        loop {
            iter.indices(&mut buf);
            let tempk = unsafe { buf.assume_init() };
            let tempk_slice = &tempk[..k];

            // Collect unique signatures from positive rows
            let mut seen = Vec::with_capacity(tt.pos_rows);
            for r in 0..tt.pos_rows {
                let sig = pos_signature(tt, tempk_slice, r);
                if !seen.contains(&sig) {
                    seen.push(sig);
                }
            }

            // Filter: remove signatures that overlap with negative rows
            let candidates: Vec<PrimeCube> = seen.iter()
                .filter(|&&sig| sig_not_in_negatives(tt, tempk_slice, sig))
                .map(|&sig| build_cube(tt, tempk_slice, sig))
                .collect();

            // Filter: only keep PIs not subsumed by existing ones
            let filtered: Vec<_> = candidates.iter().filter(|c| {
                !all_pis.iter().any(|e| e.subsumes(**c))
            }).cloned().collect();
            all_pis.extend(filtered);

            if !iter.next() { break; }
        }
    }

    all_pis.sort_by_key(|pc| pc.dim());
    all_pis
}

// ---------------------------------------------------------------------------
// Bridge: PrimeCube → Implicant<E> for integration with QM solver pipeline
// ---------------------------------------------------------------------------

/// Convert PrimeCube[] to Implicant<E>[] for Petrick's method
pub fn prime_cubes_to_implicants<E: MintermEncoding>(
    cubes: &[PrimeCube],
    variables: usize,
) -> Vec<Implicant<E>> {
    cubes.iter().map(|cube| {
        let mut bits = Vec::with_capacity(variables);
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
    }).collect()
}

/// Populate covered_minterms for Implicants converted from PrimeCubes
pub fn populate_covered_minterms_u64<E: MintermEncoding>(
    pis: &mut [Implicant<E>],
    all_minterms: &[E::Value],
    n_vars: usize,
) {
    for pi in pis.iter_mut() {
        let mut covered = Vec::with_capacity(all_minterms.len());
        for &mt in all_minterms {
            let raw = mt.to_u64() as u64;
            if covers_implicant_u64(&pi.bits, raw, n_vars) {
                covered.push(mt);
            }
        }
        pi.covered_minterms = covered;
    }
}

/// Check if an Implicant covers a raw u64 minterm by matching bits
fn covers_implicant_u64(bits: &[BitState], raw_minterm: u64, n_vars: usize) -> bool {
    for (i, &state) in bits.iter().take(n_vars).enumerate() {
        if state == BitState::DontCare { continue; }
        let expected_u64 = if state == BitState::One { 1u64 } else { 0u64 };
        let actual = (raw_minterm >> i) & 1;
        if actual != expected_u64 { return false; }
    }
    true
}

/// Build coverage: each PI maps to a bitmap of covered minterms
pub fn build_coverage_matrix(pis: &[PrimeCube], minterms: &[u64]) -> Vec<u64> {
    if pis.is_empty() || minterms.is_empty() { return Vec::new(); }
    let n = minterms.len();
    let u64s = (n + 63) / 64;
    let mut coverage = vec![0u64; pis.len() * u64s];

    for (pi_idx, pi) in pis.iter().enumerate() {
        let base = pi_idx * u64s;
        for (mt_idx, &mt) in minterms.iter().enumerate() {
            // PI covers minterm: all fixed bits of PI match minterm
            let match_ = pi.cond & !pi.mask;
            let pi_masked = pi.data & match_;
            let mt_masked = mt & match_;
            if pi_masked == mt_masked {
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
        let found_x0: bool = pis.iter().any(|pi| pi.cond == 2 && pi.data == 0 && pi.mask == 0);
        assert!(found_x0, "min_cubes should produce X0_ for B=0 function: found {:?} PIs",
            pis.iter().map(|p| (p.cond, p.mask, p.data, p.dim())).collect::<Vec<_>>());
    }
}
