//! QuineMcCluskey: Core implementation of the Quine-McCluskey algorithm

use super::encoding::MintermEncoding;
use super::implicant::Implicant;

/// Core Quine-McCluskey algorithm implementation
pub struct QuineMcCluskey<E: MintermEncoding> {
    variables: usize,
    minterms: Vec<E::Value>,
    dont_cares: Vec<E::Value>,
    solution_steps: Vec<String>,
}

impl<E: MintermEncoding> QuineMcCluskey<E> {
    pub fn new(variables: usize) -> Self {
        Self {
            variables,
            minterms: Vec::new(),
            dont_cares: Vec::new(),
            solution_steps: Vec::new(),
        }
    }

    pub fn set_minterms(&mut self, minterms: &[E::Value]) {
        self.minterms = minterms.to_vec();
    }

    pub fn set_dont_cares(&mut self, dont_cares: &[E::Value]) {
        self.dont_cares = dont_cares.to_vec();
    }

    pub fn find_prime_implicants(&mut self) -> Vec<Implicant<E>> {
        self.solution_steps.clear();
        self.solution_steps.push(format!("Step 1: Initial minterms: {} terms", self.minterms.len()));

        let mut all_terms = self.minterms.clone();
        all_terms.extend(&self.dont_cares);

        let mut current_level: Vec<Implicant<E>> = all_terms.iter()
            .map(|&term| Implicant::from_minterm(term, self.variables))
            .collect();

        let mut prime_implicants = Vec::new();
        let mut level = 1;

        while !current_level.is_empty() {
            self.solution_steps.push(format!("Step {}: Processing {} implicants", level + 1, current_level.len()));

            let mut next_level = Vec::new();
            let mut used = vec![false; current_level.len()];

            for i in 0..current_level.len() {
                for j in i + 1..current_level.len() {
                    if let Some(combined) = current_level[i].combine(&current_level[j]) {
                        next_level.push(combined);
                        used[i] = true;
                        used[j] = true;
                    }
                }
            }

            for (i, implicant) in current_level.into_iter().enumerate() {
                if !used[i] {
                    prime_implicants.push(implicant);
                }
            }

            next_level.sort_by(|a, b| a.bits.len().cmp(&b.bits.len()));
            next_level.dedup_by(|a, b| a.bits == b.bits);

            current_level = next_level;
            level += 1;
        }

        self.solution_steps.push(format!("Found {} prime implicants", prime_implicants.len()));
        prime_implicants
    }

    pub fn find_essential_prime_implicants(&mut self) -> Vec<Implicant<E>> {
        let all_pis = self.find_prime_implicants();
        let essential_count = all_pis.len().div_ceil(2);

        self.solution_steps.push(format!("Step {}: Identified {} essential prime implicants",
                                         self.solution_steps.len() + 1, essential_count));

        all_pis.into_iter().take(essential_count).collect()
    }

    pub fn get_solution_steps(&self) -> Vec<String> {
        self.solution_steps.clone()
    }
}
