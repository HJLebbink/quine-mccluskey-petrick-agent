//! QMSolver: High-level solver interface for Quine-McCluskey minimization

use super::encoding::{BitOps, MintermEncoding};
use super::implicant::{BitState, Implicant};
use super::min_cubes::{find_prime_implicants, populate_covered_minterms_u64, prime_cubes_to_implicants, TruthTable};
use super::petricks_method::PetricksMethod;
use super::qm_result::QMResult;
use super::quine_mccluskey::QuineMcCluskey;

/// High-level solver for Quine-McCluskey Boolean minimization
pub struct QMSolver<E: MintermEncoding> {
    variables: usize,
    minterms: Vec<E::Value>,
    dont_cares: Vec<E::Value>,
    variable_names: Vec<String>,
}

impl<E: MintermEncoding> QMSolver<E> {
    pub fn new(variables: usize) -> Self {
        //TODO if variables > 26, should we generate names like AA, BB, ...?
        let variable_names = (0..variables)
            .map(|i| ((b'A' + i as u8) as char).to_string())
            .collect();

        Self::with_variable_names(variables, variable_names)
    }

    pub fn with_variable_names(variables: usize, names: Vec<String>) -> Self {
        Self {
            variables,
            minterms:  Vec::with_capacity(0),
            dont_cares:  Vec::with_capacity(0),
            variable_names: names,
        }
    }

    pub fn set_minterms(&mut self, minterms: Vec<E::Value>) {
        self.minterms = minterms;
    }

    pub fn set_dont_cares(&mut self, dont_cares: Vec<E::Value>) {
        self.dont_cares = dont_cares;
    }

    pub fn solve(&self) -> QMResult {
        let mut qm = QuineMcCluskey::<E>::new(self.variables);
        qm.set_minterms(self.minterms.clone());
        qm.set_dont_cares(self.dont_cares.clone());

        let prime_implicants = qm.find_prime_implicants();
        let essential_pis = qm.find_essential_prime_implicants();

        let petricks = PetricksMethod::<E>::new(&prime_implicants, &self.minterms);
        let minimal_cover = petricks.find_minimal_cover();

        let minimized_expression = self.format_expression(&minimal_cover);

        QMResult {
            minimized_expression,
            prime_implicants: self.format_implicants(&prime_implicants),
            essential_prime_implicants: self.format_implicants(&essential_pis),
            solution_steps: qm.get_solution_steps().to_vec(),
            cost_original: self.calculate_original_cost(),
            cost_minimized: minimal_cover.len() * 2,
        }
    }

    fn format_expression(&self, implicants: &[Implicant<E>]) -> String {
        if implicants.is_empty() {
            return "0".to_string();
        }

        implicants.iter()
            .map(|imp| self.format_single_implicant(imp))
            .collect::<Vec<_>>()
            .join(" + ")
    }

    fn format_single_implicant(&self, implicant: &Implicant<E>) -> String {
        let mut result = String::new();
        for i in 0..self.variables {
            match implicant.get_bit(i) {
                BitState::Zero => result.push_str(&format!("{}'", self.variable_names[i])),
                BitState::One => result.push_str(&self.variable_names[i]),
                BitState::DontCare => {},
            }
        }
        if result.is_empty() {
            "1".to_string()
        } else {
            result
        }
    }

    fn format_implicants(&self, implicants: &[Implicant<E>]) -> Vec<String> {
        implicants.iter()
            .map(|imp| self.format_single_implicant(imp))
            .collect()
    }

    fn calculate_original_cost(&self) -> usize {
        self.minterms.len() * self.variables
    }

    // -----------------------------------------------------------------------
    // Min-cubes solver using condition-combination PI generation
    // -----------------------------------------------------------------------

    /// Solve using the min-cubes prime implicant generation algorithm.
    /// Produces an identical `QMResult` as `solve()` but via condition
    /// combination enumeration instead of Hamming-distance-1 merging.
    pub fn solve_min_cubes(&self) -> QMResult {
        // 1. Build truth table from minterms + dont-cares
        let n_conds = self.variables;
        let minterm_bits: Vec<u64> = self.minterms.iter().map(|m| m.to_u64() as u64).collect();
        let dc_bits: Vec<u64> = self.dont_cares.iter().map(|m| m.to_u64() as u64).collect();

        let tt = TruthTable::from_minterms(n_conds, &minterm_bits, &dc_bits)
            .expect("invalid truth table parameters");

        // 2. Find all prime implicants via min-cubes
        let cubies = find_prime_implicants(&tt, n_conds);

        // 3. Convert to Implicant<E>
        let mut pis = prime_cubes_to_implicants(&cubies, n_conds);

        // 4. Populate covered_minterms for Petrick's method and essential PI detection
        let all_true: Vec<E::Value> = self.minterms.iter().chain(self.dont_cares.iter()).cloned().collect();
        populate_covered_minterms_u64(&mut pis, &all_true, n_conds);

        // 5. Find essential prime implicants
        let essential_pis = find_essential_pis(&pis, &self.minterms);

        // 6. Petrick's method for minimal cover
        let petricks = PetricksMethod::<E>::new(&pis, &self.minterms);
        let minimal_cover = petricks.find_minimal_cover();

        // 7. Format result
        let minimized_expression = self.format_expression(&minimal_cover);

        let steps = vec![
            format!("Step 1: Built truth table with {} positive, {} negative rows", tt.pos_rows, tt.neg_rows),
            format!("Step 2: Found {} prime implicants via min-cubes", pis.len()),
            format!("Step 3: Identified {} essential prime implicants", essential_pis.len()),
            format!("Step 4: Petrick's method selected {} PIs for minimal cover", minimal_cover.len()),
        ];

        QMResult {
            minimized_expression,
            prime_implicants: self.format_implicants(&pis),
            essential_prime_implicants: self.format_implicants(&essential_pis),
            solution_steps: steps,
            cost_original: self.calculate_original_cost(),
            cost_minimized: minimal_cover.len() * 2,
        }
    }
}

/// Find essential prime implicants (those uniquely covering at least one minterm)
fn find_essential_pis<E: MintermEncoding>(pis: &[Implicant<E>], minterms: &[E::Value]) -> Vec<Implicant<E>> {
    // Build coverage map: minterm -> list of PIs that cover it
    let mut coverage_map: std::collections::HashMap<E::Value, Vec<usize>> =
        std::collections::HashMap::new();
    for minterm in minterms {
        for (pi_idx, pi) in pis.iter().enumerate() {
            if pi.covers_minterm(*minterm) {
                coverage_map.entry(*minterm).or_default().push(pi_idx);
            }
        }
    }

    let mut essential_indices = std::collections::HashSet::new();
    for (_minterm, covering_pis) in &coverage_map {
        if covering_pis.len() == 1 {
            essential_indices.insert(covering_pis[0]);
        }
    }

    pis.iter()
        .enumerate()
        .filter_map(|(idx, pi)| {
            if essential_indices.contains(&idx) {
                Some(pi.clone())
            } else {
                None
            }
        })
        .collect()
}
