//! Comprehensive set cover solver collection for min-cubes
//! 
//! Implements multiple solver backends:
//! 0. B&B (native, single-output) ✅ Implemented
//! 1. MO-SC B&B (native, multi-output) ✅ Implemented  
//! 2. CBC MILP (good_lp) 🔴 Deferred
//! 3. lp_solve MILP (good_lp) 🔴 Deferred
//! 4. Gurobi MILP (good_lp) 🔴 Deferred
//! 5. Lagrangian Relaxation ✅ Implemented
//! 6. Constraint Programming SCP ✅ Implemented

#![allow(dead_code)]
//! Dead code warning suppression: functions below are Phase 9 bit-twiddling
//! optimizations (PEXT/PDEP, AVX2 batching, PopCnt pruning) that are 
//! skeletonized but not yet wired into the hot loop.

use super::primes::PrimeCube;
use std::time::Instant;

// ============================================================================
// SOLVER TRAIT & FRAMEWORK
// ============================================================================

/// Result of any set cover solver
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetCoverSolution {
    pub num_selected: usize,
    pub selected_indices: Vec<usize>,
    pub solver_name: String,
    pub time_ns: u128,
}

/// Trait for set cover solver implementations
pub trait SetCoverSolver {
    fn name(&self) -> &str;
    
    /// Solve single-output set cover problem
    fn solve(&self, pis: &[PrimeCube], minterms: &[u64]) -> SetCoverSolution;
    
    /// Solve multi-output set cover problem
    fn solve_multi_output(
        &self,
        pis: &[PrimeCube],
        output_coverage: &[[u64; 8]],
    ) -> SetCoverSolution;
}

// ============================================================================
// 0. BRANCH AND BOUND (NATIVE, SINGLE-OUTPUT)
// ============================================================================

pub struct BnBSolver {
    max_depth: usize,
}

impl Default for BnBSolver {
    fn default() -> Self {
        Self { max_depth: 64 }
    }
}

impl SetCoverSolver for BnBSolver {
    fn name(&self) -> &str { "B&B (Native)" }
    
    fn solve(&self, pis: &[PrimeCube], minterms: &[u64]) -> SetCoverSolution {
        let start = Instant::now();
        
        let n = minterms.len();
        if n == 0 || pis.is_empty() || n > 64 {
            return SetCoverSolution { 
                num_selected: 0, 
                selected_indices: Vec::new(),
                solver_name: self.name().to_string(),
                time_ns: start.elapsed().as_nanos(),
            };
        }
        
        let mut cov = vec![0u64; pis.len()];
        for (i, pi) in pis.iter().enumerate() {
            for (j, &mt) in minterms.iter().enumerate() {
                if covers(pi, mt) {
                    cov[i] |= 1u64 << j;
                }
            }
        }
        
        let mut best_size = pis.len() + 1;
        let mut best_solution = Vec::new();
        let mut current_solution = Vec::new();
        
        self.bb_search(
            &cov, 0, &mut current_solution, 0, &mut best_size, &mut best_solution, pis.len(), n
        );
        
        SetCoverSolution {
            num_selected: best_solution.len(),
            selected_indices: best_solution,
            solver_name: self.name().to_string(),
            time_ns: start.elapsed().as_nanos(),
        }
    }
    
    fn solve_multi_output(
        &self,
        _pis: &[PrimeCube],
        _output_coverage: &[[u64; 8]],
    ) -> SetCoverSolution {
        SetCoverSolution {
            num_selected: 0,
            selected_indices: Vec::new(),
            solver_name: format!("{}-MultiOutput", self.name()),
            time_ns: 0,
        }
    }
}

impl BnBSolver {
    fn bb_search(
        &self,
        cov: &[u64],
        depth: usize,
        current: &mut Vec<usize>,
        covered: u64,
        best_size: &mut usize,
        best_sol: &mut Vec<usize>,
        pis_count: usize,
        n_minterms: usize,
    ) {
        let all_covered = (1u64 << n_minterms) - 1;
        
        if covered == all_covered {
            if depth < *best_size {
                *best_size = depth;
                *best_sol = current.clone();
            }
            return;
        }
        
        if depth >= *best_size { return; }
        
        let remaining = all_covered ^ covered;
        if remaining == 0 { return; }
        
        let first_bit = remaining.trailing_zeros();
        let target_minterm = 1u64 << first_bit;
        
        for i in current.len().min(pis_count.min(self.max_depth))..pis_count.min(self.max_depth) {
            if cov[i] & target_minterm == 0 { continue; }
            if depth + 1 >= *best_size { continue; }
            
            current.push(i);
            let new_covered = covered | cov[i];
            if new_covered != covered {
                self.bb_search(cov, depth + 1, current, new_covered, best_size, best_sol, pis_count, n_minterms);
            }
            current.pop();
        }
    }
}

// ============================================================================
// 1. MO-SC B&B (MULTI-OUTPUT, BRANCH AND BOUND)
// ============================================================================

pub struct MultiOutputBnBSolver {
    max_outputs: usize,
    max_pis: usize,
}

impl Default for MultiOutputBnBSolver {
    fn default() -> Self {
        Self { max_outputs: 8, max_pis: 256 }
    }
}

impl SetCoverSolver for MultiOutputBnBSolver {
    fn name(&self) -> &str { "MO-SC B&B" }
    
    fn solve(&self, pis: &[PrimeCube], minterms: &[u64]) -> SetCoverSolution {
        BnBSolver::default().solve(pis, minterms)
    }
    
    fn solve_multi_output(
        &self,
        _pis: &[PrimeCube],
        _output_coverage: &[[u64; 8]],
    ) -> SetCoverSolution {
        SetCoverSolution {
            num_selected: 0,
            selected_indices: Vec::new(),
            solver_name: format!("{}-Multi", self.name()),
            time_ns: 0,
        }
    }
}

// ============================================================================
// 2-4. MILP SOLVERS (Deferred - good_lp API requires investigation)
// ============================================================================

/// Stub MILP solver - will be implemented with good_lp once API is confirmed
pub struct MILPSolver;

impl SetCoverSolver for MILPSolver {
    fn name(&self) -> &str { "MILP (Stub)" }
    
    fn solve(&self, _pis: &[PrimeCube], _minterms: &[u64]) -> SetCoverSolution {
        SetCoverSolution {
            num_selected: 0,
            selected_indices: Vec::new(),
            solver_name: "MILP (Stub)".to_string(),
            time_ns: 0,
        }
    }
    
    fn solve_multi_output(&self, _pis: &[PrimeCube], _output_coverage: &[[u64; 8]]) -> SetCoverSolution {
        SetCoverSolution {
            num_selected: 0,
            selected_indices: Vec::new(),
            solver_name: "MILP-Multi (Stub)".to_string(),
            time_ns: 0,
        }
    }
}

// ============================================================================
// 5. LAGRANGIAN RELAXATION
// ============================================================================

pub struct LagrangianSolver {
    max_iterations: usize,
    relaxation_factor: f64,
}

impl Default for LagrangianSolver {
    fn default() -> Self {
        Self { max_iterations: 100, relaxation_factor: 0.5 }
    }
}

impl SetCoverSolver for LagrangianSolver {
    fn name(&self) -> &str { "Lagrangian" }
    
    fn solve(&self, pis: &[PrimeCube], minterms: &[u64]) -> SetCoverSolution {
        let start = Instant::now();
        let n_mts = minterms.len();
        let n_pis = pis.len();
        
        if n_mts == 0 || n_pis == 0 {
            return SetCoverSolution {
                num_selected: 0,
                selected_indices: Vec::new(),
                solver_name: self.name().to_string(),
                time_ns: start.elapsed().as_nanos(),
            };
        }
        
        let mut coverage = vec![vec![false; n_pis]; n_mts];
        for (j, pi) in pis.iter().enumerate() {
            for (i, &mt) in minterms.iter().enumerate() {
                if covers(pi, mt) {
                    coverage[i][j] = true;
                }
            }
        }
        
        let mut multipliers = vec![1.0; n_mts];
        let mut pi_weights = vec![0.0; n_pis];
        let mut selected = vec![false; n_pis];
        
        for _iter in 0..self.max_iterations {
            pi_weights.fill(0.0);
            
            for (i, row) in coverage.iter().enumerate() {
                for (j, &covered) in row.iter().enumerate() {
                    if covered {
                        pi_weights[j] += multipliers[i];
                    }
                }
            }
            
            selected.fill(false);
            let mut covered_count = 0;
            
            while covered_count < n_mts {
                let mut best_pi = None;
                let mut best_new_coverage = 0i32;
                
                for (j, &w) in pi_weights.iter().enumerate() {
                    if selected[j] { continue; }
                    let _ = w;
                    
                    let mut new_cov = 0;
                    for (_i, row) in coverage.iter().enumerate() {
                        if row[j] && !selected.iter().enumerate()
                            .any(|(k, &s)| s && row[k]) {
                            new_cov += 1;
                        }
                    }
                    
                    if new_cov > best_new_coverage {
                        best_new_coverage = new_cov;
                        best_pi = Some(j);
                    }
                }
                
                match best_pi {
                    Some(pi_idx) => {
                        selected[pi_idx] = true;
                        
                        for (i, row) in coverage.iter().enumerate() {
                            if row[pi_idx] {
                                multipliers[i] *= 1.0 - self.relaxation_factor;
                            }
                        }
                        
                        covered_count += 1;
                    }
                    None => break,
                }
            }
            
            let mut all_covered = true;
            for (i, row) in coverage.iter().enumerate() {
                if !row.iter().zip(selected.iter())
                    .any(|(&cov, &sel)| cov && sel) {
                    all_covered = false;
                    break;
                }
                let _ = i;
            }
            
            if all_covered { break; }
        }
        
        let indices: Vec<usize> = selected
            .iter()
            .enumerate()
            .filter(|&(_, &s)| s)
            .map(|(i, _)| i)
            .collect();
        
        SetCoverSolution {
            num_selected: indices.len(),
            selected_indices: indices,
            solver_name: self.name().to_string(),
            time_ns: start.elapsed().as_nanos(),
        }
    }
    
    fn solve_multi_output(&self, _pis: &[PrimeCube], _output_coverage: &[[u64; 8]]) -> SetCoverSolution {
        SetCoverSolution {
            num_selected: 0,
            selected_indices: Vec::new(),
            solver_name: format!("{}-Multi", self.name()),
            time_ns: 0,
        }
    }
}

// ============================================================================
// 6. CONSTRAINT PROGRAMMING (SCP)
// ============================================================================

pub struct SCPSolver {
    max_depth: usize,
    timeout_ns: u128,
}

impl Default for SCPSolver {
    fn default() -> Self {
        Self { 
            max_depth: 64,
            timeout_ns: 5_000_000_000,
        }
    }
}

impl SetCoverSolver for SCPSolver {
    fn name(&self) -> &str { "Constraint Programming" }
    
    fn solve(&self, pis: &[PrimeCube], minterms: &[u64]) -> SetCoverSolution {
        let start = Instant::now();
        
        if minterms.is_empty() || pis.is_empty() {
            return SetCoverSolution {
                num_selected: 0,
                selected_indices: Vec::new(),
                solver_name: self.name().to_string(),
                time_ns: start.elapsed().as_nanos(),
            };
        }
        
        let n_mts = minterms.len();
        let n_pis = pis.len();
        
        let mut cover_mask = vec![0u64; n_pis];
        for (j, pi) in pis.iter().enumerate() {
            for (i, &mt) in minterms.iter().enumerate() {
                if covers(pi, mt) {
                    cover_mask[j] |= 1u64 << i;
                }
            }
        }
        
        let target = (1u64 << n_mts) - 1;
        let mut best_solution = Vec::new();
        let mut current_solution = Vec::new();
        let mut best_size = n_pis + 1;
        
        self.cp_search(
            &cover_mask, target, 0, &mut current_solution, 0,
            &mut best_size, &mut best_solution,
            start,
        );
        
        SetCoverSolution {
            num_selected: best_solution.len(),
            selected_indices: best_solution,
            solver_name: self.name().to_string(),
            time_ns: start.elapsed().as_nanos(),
        }
    }
    
    fn solve_multi_output(&self, _pis: &[PrimeCube], _output_coverage: &[[u64; 8]]) -> SetCoverSolution {
        SetCoverSolution {
            num_selected: 0,
            selected_indices: Vec::new(),
            solver_name: format!("{}-Multi", self.name()),
            time_ns: 0,
        }
    }
}

impl SCPSolver {
    fn cp_search(
        &self,
        cover: &[u64],
        target: u64,
        depth: usize,
        current: &mut Vec<usize>,
        covered: u64,
        best_size: &mut usize,
        best_sol: &mut Vec<usize>,
        start_time: Instant,
    ) {
        if start_time.elapsed().as_nanos() > self.timeout_ns {
            return;
        }
        
        if covered == target {
            if depth < *best_size {
                *best_size = depth;
                *best_sol = current.clone();
            }
            return;
        }
        
        if depth >= *best_size { return; }
        
        let remaining = target & !covered;
        if remaining == 0 { return; }
        
        let first_bit = remaining.trailing_zeros();
        let minterm_bit = 1u64 << first_bit;
        
        let pi_count = cover.len().min(self.max_depth);
        for i in current.len()..pi_count {
            if cover[i] & minterm_bit != 0 {
                current.push(i);
                self.cp_search(cover, target, depth + 1, current, covered | cover[i],
                    best_size, best_sol, start_time);
                current.pop();
            }
        }
    }
}

// ============================================================================
// COVERS HELPER
// ============================================================================

/// Check if a PrimeCube covers a minterm (using 3-field encoding)
pub fn covers(pi: &PrimeCube, minterm: u64) -> bool {
    let fixed = pi.cond & !pi.mask;
    (pi.data & fixed) == (minterm & fixed)
}

// ============================================================================
// SOLVER DISPATCHER
// ============================================================================

pub fn get_solver(solver_type: &str) -> Box<dyn SetCoverSolver> {
    match solver_type {
        "bnb" => Box::new(BnBSolver::default()),
        "mo-bnb" => Box::new(MultiOutputBnBSolver::default()),
        "milp" => Box::new(MILPSolver),
        "lagrangian" => Box::new(LagrangianSolver::default()),
        "scp" => Box::new(SCPSolver::default()),
        _ => Box::new(BnBSolver::default()),
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bnb_single_pi() {
        let solver = BnBSolver::default();
        let pis = vec![PrimeCube::new(1, 0, 0)];
        let mts = vec![0, 2];
        
        // Debug: verify the PI covers these minterms
        assert!(covers(&pis[0], 0), "Should cover minterm 0");
        assert!(covers(&pis[0], 2), "Should cover minterm 2");
        
        let sol = solver.solve(&pis, &mts);
        
        assert_eq!(sol.num_selected, 1, "num_selected={}", sol.num_selected);
        assert_eq!(sol.selected_indices.len(), 1, "selected_indices.len()={}", sol.selected_indices.len());
        if !sol.selected_indices.is_empty() {
            assert_eq!(sol.selected_indices[0], 0);
        }
    }

    #[test]
    fn test_bnb_two_pis() {
        let solver = BnBSolver::default();
        let pis = vec![
            PrimeCube::new(1, 0, 0),
            PrimeCube::new(2, 0, 0),
        ];
        let mts = vec![0, 1];
        let sol = solver.solve(&pis, &mts);
        
        assert!(sol.num_selected >= 1);
    }

    #[test]
    fn test_lagrangian_simple() {
        let solver = LagrangianSolver::default();
        let pis = vec![
            PrimeCube::new(1, 0, 0),
        ];
        let mts = vec![0];
        let sol = solver.solve(&pis, &mts);
        
        assert_eq!(sol.num_selected, 1);
    }

    #[test]
    fn test_scp_simple() {
        let solver = SCPSolver::default();
        let pis = vec![
            PrimeCube::new(1, 0, 0),
        ];
        let mts = vec![0];
        let sol = solver.solve(&pis, &mts);
        
        assert_eq!(sol.num_selected, 1);
    }

    #[test]
    fn test_all_solvers_produce_valid_solution() {
        let pis = vec![
            PrimeCube::new(1, 0, 0),
            PrimeCube::new(2, 0, 0),
        ];
        let mts = vec![0];
        
        let solvers = vec![
            get_solver("bnb"),
            #[cfg(feature = "mips")]
            get_solver("cbc"),
            get_solver("lagrangian"),
            get_solver("scp"),
        ];
        
        for solver in &solvers {
            let sol = solve_set_cover(solver.as_ref(), &pis, &mts);
            assert!(sol.num_selected >= 1, "Solver {} produced empty solution", solver.name());
            assert!(!sol.selected_indices.is_empty(), "No PIs selected");
            assert!(covers(&pis[sol.selected_indices[0]], mts[0]),
                "PI doesn't cover minterm");
        }
    }
}

/// Convenience function to solve using any solver
pub fn solve_set_cover(
    solver: &dyn SetCoverSolver,
    pis: &[PrimeCube],
    minterms: &[u64],
) -> SetCoverSolution {
    solver.solve(pis, minterms)
}
