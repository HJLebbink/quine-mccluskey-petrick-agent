pub mod quine_mccluskey;
pub mod petricks_method;
pub mod utils;

pub use quine_mccluskey::{QuineMcCluskey, DummyImplicant, BitState};
pub use petricks_method::PetricksMethod;

#[derive(Debug, Clone)]
pub struct QMResult {
    pub minimized_expression: String,
    pub prime_implicants: Vec<String>,
    pub essential_prime_implicants: Vec<String>,
    pub solution_steps: Vec<String>,
    pub cost_original: usize,
    pub cost_minimized: usize,
}

pub struct QMSolver {
    variables: usize,
    minterms: Vec<u32>,
    dont_cares: Vec<u32>,
    variable_names: Vec<String>,
}

impl QMSolver {
    pub fn new(variables: usize) -> Self {
        let variable_names = (0..variables)
            .map(|i| ((b'A' + i as u8) as char).to_string())
            .collect();

        Self {
            variables,
            minterms: Vec::new(),
            dont_cares: Vec::new(),
            variable_names,
        }
    }

    pub fn with_variable_names(variables: usize, names: Vec<String>) -> Self {
        Self {
            variables,
            minterms: Vec::new(),
            dont_cares: Vec::new(),
            variable_names: names,
        }
    }

    pub fn set_minterms(&mut self, minterms: &[u32]) {
        self.minterms = minterms.to_vec();
    }

    pub fn set_dont_cares(&mut self, dont_cares: &[u32]) {
        self.dont_cares = dont_cares.to_vec();
    }

    pub fn solve(&self) -> QMResult {
        let mut qm = QuineMcCluskey::new(self.variables);
        qm.set_minterms(&self.minterms);
        qm.set_dont_cares(&self.dont_cares);

        let prime_implicants = qm.find_prime_implicants();
        let essential_pis = qm.find_essential_prime_implicants();

        let petricks = PetricksMethod::new(&prime_implicants, &self.minterms);
        let minimal_cover = petricks.find_minimal_cover();

        let minimized_expression = self.format_expression(&minimal_cover);
        let steps = qm.get_solution_steps();

        QMResult {
            minimized_expression,
            prime_implicants: self.format_implicants(&prime_implicants),
            essential_prime_implicants: self.format_implicants(&essential_pis),
            solution_steps: steps,
            cost_original: self.calculate_original_cost(),
            cost_minimized: minimal_cover.len() * 2,
        }
    }

    fn format_expression(&self, implicants: &[DummyImplicant]) -> String {
        if implicants.is_empty() {
            return "0".to_string();
        }

        implicants.iter()
            .map(|imp| self.format_single_implicant(imp))
            .collect::<Vec<_>>()
            .join(" + ")
    }

    fn format_single_implicant(&self, implicant: &DummyImplicant) -> String {
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

    fn format_implicants(&self, implicants: &[DummyImplicant]) -> Vec<String> {
        implicants.iter()
            .map(|imp| self.format_single_implicant(imp))
            .collect()
    }

    fn calculate_original_cost(&self) -> usize {
        self.minterms.len() * self.variables
    }
}