//! QMSolver: High-level solver interface for Quine-McCluskey minimization

use super::encoding::MintermEncoding;
use super::implicant::{BitState, Implicant};
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
}
