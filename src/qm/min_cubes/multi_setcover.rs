//! Multi-Output Set Cover (MO-SC) — Phase 5 from PORT_PLAN.md
//!
//! Handles joint minimization across multiple output functions,
//! finding shared cubes that reduce total gate count.

use super::primes::PrimeCube;

/// Multi-output PI: valid for multiple outputs simultaneously
/// A PI that covers minterm m_a in output 0 and m_b in output 1
/// counts as ONE gate but satisfies constraints for both outputs.
#[derive(Debug, Clone, Copy)]
pub struct MultiPI {
    /// The 3-field PI encoding: (cond, mask, data)
    pub pi: PrimeCube,
    /// Which outputs this PI is valid for (bitmask: bit k = 1 means output k)
    pub output_mask: u16,
    /// Coverage: bitset where bit m = 1 if minterm m is covered
    pub cover: u64,
}

impl MultiPI {
    #[inline]
    pub fn new(pi: PrimeCube, output_mask: u16, cover: u64) -> Self {
        Self { pi, output_mask, cover }
    }

    #[inline]
    pub fn is_valid_for(&self, output: usize) -> bool {
        (self.output_mask >> output) & 1 != 0
    }

    /// Higher = more don't-cares = more general
    #[inline]
    pub fn generality(&self) -> u32 {
        self.pi.cond.count_ones() + self.pi.mask.count_ones() * 10
    }
}

/// Multi-output set-cover problem
pub struct MultiSetCoverProblem {
    pub num_outputs: usize,
    pub num_minterms: usize,
    pub candidates: Vec<MultiPI>,
    pub best_cost: usize,
    pub best_selection: Vec<usize>,
}

impl MultiSetCoverProblem {
    /// Create from single-output PIs extended to all outputs
    pub fn new(num_outputs: usize, num_minterms: usize, pis: Vec<PrimeCube>) -> Option<Self> {
        if num_outputs == 0 || num_minterms == 0 { return None; }
        
        let mut candidates = Vec::new();
        for (pi_idx, pi) in pis.iter().enumerate() {
            let cover = 0u64;
            candidates.push(MultiPI::new(*pi, 1u16 << (pi_idx % num_outputs), cover));
        }
        
        Some(Self {
            num_outputs,
            num_minterms,
            candidates,
            best_cost: usize::MAX,
            best_selection: Vec::new(),
        })
    }
}

/// State for multi-output B&B
struct MObbState {
    /// Coverage per output
    covered: Vec<u64>,
    current_selection: Vec<usize>,
    best_size: usize,
    best_solution: Vec<usize>,
}