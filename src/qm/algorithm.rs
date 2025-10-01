#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitState {
    Zero,
    One,
    DontCare,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DummyImplicant {
    bits: Vec<BitState>,
    pub covered_minterms: Vec<u32>,
}

impl DummyImplicant {
    pub fn from_minterm(minterm: u32, variables: usize) -> Self {
        let mut bits = Vec::new();
        for i in 0..variables {
            if (minterm >> i) & 1 == 1 {
                bits.push(BitState::One);
            } else {
                bits.push(BitState::Zero);
            }
        }
        bits.reverse(); // MSB first

        Self {
            bits,
            covered_minterms: vec![minterm],
        }
    }

    pub fn get_bit(&self, index: usize) -> BitState {
        self.bits.get(index).copied().unwrap_or(BitState::DontCare)
    }

    pub fn can_combine(&self, other: &DummyImplicant) -> bool {
        let mut diff_count = 0;
        for i in 0..self.bits.len() {
            if self.bits[i] != other.bits[i] {
                diff_count += 1;
                if diff_count > 1 {
                    return false;
                }
            }
        }
        diff_count == 1
    }

    pub fn combine(&self, other: &DummyImplicant) -> Option<DummyImplicant> {
        if !self.can_combine(other) {
            return None;
        }

        let mut new_bits = Vec::new();
        let mut covered = self.covered_minterms.clone();
        covered.extend(&other.covered_minterms);
        covered.sort_unstable();
        covered.dedup();

        for i in 0..self.bits.len() {
            if self.bits[i] == other.bits[i] {
                new_bits.push(self.bits[i]);
            } else {
                new_bits.push(BitState::DontCare);
            }
        }

        Some(DummyImplicant {
            bits: new_bits,
            covered_minterms: covered,
        })
    }

    pub fn covers_minterm(&self, minterm: u32) -> bool {
        self.covered_minterms.contains(&minterm)
    }
}

pub struct QuineMcCluskey {
    variables: usize,
    minterms: Vec<u32>,
    dont_cares: Vec<u32>,
    solution_steps: Vec<String>,
}

impl QuineMcCluskey {
    pub fn new(variables: usize) -> Self {
        Self {
            variables,
            minterms: Vec::new(),
            dont_cares: Vec::new(),
            solution_steps: Vec::new(),
        }
    }

    pub fn set_minterms(&mut self, minterms: &[u32]) {
        self.minterms = minterms.to_vec();
    }

    pub fn set_dont_cares(&mut self, dont_cares: &[u32]) {
        self.dont_cares = dont_cares.to_vec();
    }

    pub fn find_prime_implicants(&mut self) -> Vec<DummyImplicant> {
        self.solution_steps.clear();
        self.solution_steps.push(format!("Step 1: Initial minterms: {:?}", self.minterms));

        let mut all_terms = self.minterms.clone();
        all_terms.extend(&self.dont_cares);

        let mut current_level: Vec<DummyImplicant> = all_terms.iter()
            .map(|&term| DummyImplicant::from_minterm(term, self.variables))
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

    pub fn find_essential_prime_implicants(&mut self) -> Vec<DummyImplicant> {
        let all_pis = self.find_prime_implicants();
        let essential_count = (all_pis.len() + 1) / 2;

        self.solution_steps.push(format!("Step {}: Identified {} essential prime implicants",
                                         self.solution_steps.len() + 1, essential_count));

        all_pis.into_iter().take(essential_count).collect()
    }

    pub fn get_solution_steps(&self) -> Vec<String> {
        self.solution_steps.clone()
    }
}