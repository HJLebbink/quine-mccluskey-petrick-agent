//! QuineMcCluskey: Core implementation of the Quine-McCluskey algorithm

use super::encoding::{BitOps, MintermEncoding};
use super::implicant::Implicant;

/// Core Quine-McCluskey algorithm implementation
pub struct QuineMcCluskey<E: MintermEncoding> {
    variables: usize,
    mask: E::Value,  // Cached mask for extracting data bits
    minterms: Vec<E::Value>,
    dont_cares: Vec<E::Value>,
    solution_steps: Vec<String>,
}

impl<E: MintermEncoding> QuineMcCluskey<E> {
    pub fn new(variables: usize) -> Self {
        let mask = (E::Value::one() << variables) - E::Value::one();
        Self {
            variables,
            mask,
            minterms: Vec::with_capacity(0),
            dont_cares: Vec::with_capacity(0),
            solution_steps: Vec::with_capacity(0),
        }
    }

    pub fn set_minterms(&mut self, minterms: Vec<E::Value>) {
        self.minterms = minterms;
    }

    pub fn set_dont_cares(&mut self, dont_cares: Vec<E::Value>) {
        self.dont_cares = dont_cares;
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
            let msg = format!("Step {}: Processing {} implicants", level + 1, current_level.len());
            println!("{}", msg);
            self.solution_steps.push(msg);

            let mut next_level = Vec::new();
            let mut used = vec![false; current_level.len()];

            // Use Hamming weight grouping with fast raw encoding operations
            // Two implicants can only combine if they differ by exactly 1 bit
            // So we only need to compare groups[k] with groups[k+1]
            use std::collections::HashMap;
            let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();

            // Convert all implicants to raw encoding for fast operations
            let raw_encodings: Vec<E::Value> = current_level.iter()
                .map(|imp| imp.to_raw_encoding(self.variables))
                .collect();

            // Group by Hamming weight using fast popcount on raw encoding
            for (idx, &raw_value) in raw_encodings.iter().enumerate() {
                let data = raw_value & self.mask;
                let ones_count = data.count_ones() as usize;
                groups.entry(ones_count).or_insert_with(Vec::new).push(idx);
            }

            // Use HashMap for deduplication while tracking covered_minterms
            use std::collections::HashMap as DedupeMap;
            let mut next_level_map: DedupeMap<E::Value, Vec<E::Value>> = DedupeMap::new();

            // Only compare adjacent Hamming weight groups
            let max_weight = groups.keys().max().copied().unwrap_or(0);
            for weight in 0..max_weight {
                if let (Some(group1), Some(group2)) = (groups.get(&weight), groups.get(&(weight + 1))) {
                    // Use SIMD-optimized gray code pair finding
                    let pairs = E::find_gray_code_pairs(group1, group2, &raw_encodings);

                    for (i, j) in pairs {
                        used[i] = true;
                        used[j] = true;

                        let raw_i = raw_encodings[i];
                        let raw_j = raw_encodings[j];
                        let raw_combined = Implicant::<E>::replace_complements(raw_i, raw_j, self.variables);

                        let entry = next_level_map.entry(raw_combined).or_insert_with(Vec::new);
                        entry.extend(&current_level[i].covered_minterms);
                        entry.extend(&current_level[j].covered_minterms);
                    }
                }
            }

            // Convert back to Implicants with proper covered_minterms
            for (raw_value, mut covered) in next_level_map {
                covered.sort_unstable();
                covered.dedup();

                let mut combined_imp = Implicant::<E>::from_raw_encoding(raw_value, self.variables);
                combined_imp.covered_minterms = covered;

                next_level.push(combined_imp);
            }

            for (i, implicant) in current_level.into_iter().enumerate() {
                if !used[i] {
                    prime_implicants.push(implicant);
                }
            }

            current_level = next_level;
            level += 1;
        }

        self.solution_steps.push(format!("Found {} prime implicants", prime_implicants.len()));
        prime_implicants
    }

    /// Find essential prime implicants (those that uniquely cover certain minterms).
    ///
    /// Essential prime implicants are those that are the only ones covering
    /// specific minterms. These must be included in any minimal solution.
    pub fn find_essential_prime_implicants(&mut self) -> Vec<Implicant<E>> {
        let all_pis = self.find_prime_implicants();

        // Build a coverage map: minterm -> list of prime implicants that cover it
        let mut coverage_map: std::collections::HashMap<E::Value, Vec<usize>> =
            std::collections::HashMap::new();

        for minterm in &self.minterms {
            for (pi_idx, pi) in all_pis.iter().enumerate() {
                if pi.covers_minterm(*minterm) {
                    coverage_map.entry(*minterm).or_default().push(pi_idx);
                }
            }
        }

        // Find essential prime implicants: those that uniquely cover at least one minterm
        let mut essential_indices = std::collections::HashSet::new();
        for (_minterm, covering_pis) in &coverage_map {
            if covering_pis.len() == 1 {
                // This minterm is only covered by one PI, making it essential
                essential_indices.insert(covering_pis[0]);
            }
        }

        let essential_pis: Vec<Implicant<E>> = all_pis.iter()
            .enumerate()
            .filter_map(|(idx, pi)| {
                if essential_indices.contains(&idx) {
                    Some(pi.clone())
                } else {
                    None
                }
            })
            .collect();

        self.solution_steps.push(format!(
            "Step {}: Identified {} essential prime implicants (uniquely covering minterms)",
            self.solution_steps.len() + 1, essential_pis.len()
        ));

        essential_pis
    }

    pub fn get_solution_steps(&self) -> &[String] {
        &self.solution_steps
    }
}
