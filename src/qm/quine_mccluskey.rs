//! QuineMcCluskey: Core implementation of the Quine-McCluskey algorithm

use std::arch::asm;
use super::encoding::{BitOps, MintermEncoding};
use super::implicant::Implicant;


pub fn int3() {
    unsafe { asm!("int3"); }
}


/// Core Quine-McCluskey algorithm implementation
pub struct QuineMcCluskey<E: MintermEncoding> {
    variables: usize,
    mask: E::Value, // Cached mask for extracting data bits
    minterms: Vec<E::Value>,
    dont_cares: Vec<E::Value>,
    solution_steps: Vec<String>,
    logging_on: bool,
}

impl<E: MintermEncoding> QuineMcCluskey<E> {
    /// Create a new Quine-McCluskey solver with the given number of variables.
    ///
    /// The mask for extracting data bits from minterms is computed as
    /// `(1 << variables) - 1` to support the encoding type's value width.
    pub fn new(variables: usize) -> Self {
        let mask = (E::Value::one() << variables) - E::Value::one();
        Self {
            variables,
            mask,
            minterms: Vec::with_capacity(0),
            dont_cares: Vec::with_capacity(0),
            solution_steps: Vec::with_capacity(0),
            logging_on: false,
        }
    }

    pub fn set_logging_on(&mut self, logging_on: bool) {
        self.logging_on = logging_on;
    }

    /// Set the minterms that must be covered by the prime implicants.
    ///
    /// Any previous minterms are replaced by this new set.
    pub fn set_minterms(&mut self, minterms: Vec<E::Value>) {
        self.minterms = minterms;
    }

    /// Set the don't-care minterms that may be covered but are not required.
    ///
    /// Don't-cares help the algorithm find larger prime implicants by allowing
    /// more combinations, but they do not need to be covered in the final solution.
    pub fn set_dont_cares(&mut self, dont_cares: Vec<E::Value>) {
        self.dont_cares = dont_cares;
    }

    /// Find all prime implicants using Hamming-distance-1 iterative merging.
    ///
    /// Groups implicants by Hamming weight and compare adjacent groups to find
    /// pairs that differ by exactly one bit, combining them into larger implicants.
    /// Repeats until no more combinations are possible.
    ///
    /// Clears the solution steps before starting and logs each processing level.
    fn find_prime_implicants(&mut self) -> Vec<Implicant<E>> {
        self.solution_steps.clear();
        self.solution_steps.push(format!(
            "Step 1: Initial minterms: {} terms",
            self.minterms.len()
        ));

        let mut all_terms: Vec<E::Value> = self.minterms.clone();
        all_terms.extend(&self.dont_cares);

        let mut current_level: Vec<Implicant<E>> = all_terms
            .iter()
            .map(|&term| Implicant::from_minterm(term, self.variables))
            .collect();
        
        let mut prime_implicants: Vec<Implicant<E>> = Vec::new();
        let mut order = 1;

        while !current_level.is_empty() {
            let msg = format!(
                "Step {order}: Processing {order}-order implicants (#{})",
                current_level.len()
            );
            if self.logging_on {
                println!("{msg}");
            }
            self.solution_steps.push(msg);

            let mut next_level: Vec<Implicant<E>> = Vec::new();
            let mut used = vec![false; current_level.len()];

            // Use Hamming bit-count grouping with fast raw encoding operations
            // Two implicants can only combine if they differ by exactly 1 bit,
            // So we only need to compare groups[k] with groups[k+1]
            use std::collections::HashMap;
            let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
            
            let raw_encodings: Vec<E::Value> = current_level.iter().map(|i| i.bits).collect::<Vec<_>>();

            // Group by Hamming weight using fast pop-count on raw encoding
            for (idx, &raw_value) in raw_encodings.iter().enumerate() {
                let data = raw_value & self.mask;
                let ones_count = data.count_ones() as usize;
                groups.entry(ones_count).or_insert_with(Vec::new).push(idx);
            }

            // Use HashMap for deduplication while tracking covered_minterms
            use std::collections::HashMap as DedupeMap;
            use std::collections::HashSet;
            let mut next_level_map: DedupeMap<E::Value, HashSet<E::Value>> = DedupeMap::new();

            // Only compare adjacent Hamming weight groups
            let max_bit_count = groups.keys().max().copied().unwrap_or(0);
            for bit_count in 0..max_bit_count {
                if let (Some(group1), Some(group2)) =
                    (groups.get(&bit_count), groups.get(&(bit_count + 1)))
                {
                    let start_time = std::time::Instant::now();

                    // Use SIMD-optimized gray code pair finding: here most of the time is spent
                    let pairs = E::find_gray_code_pairs(group1, group2, &raw_encodings);

                    if self.logging_on {

                        println!(
                            "number of pairs found between bit-count {bit_count} and {}: {}; time spend {:?}",
                            bit_count + 1,
                            pairs.len(),
                            start_time.elapsed()
                        );
                    }
                    for (i, j) in pairs {
                        used[i] = true;
                        used[j] = true;

                        let raw_i: E::Value = raw_encodings[i];
                        let raw_j: E::Value = raw_encodings[j];
                        let raw_combined: E::Value =
                            Implicant::<E>::replace_complements(raw_i, raw_j, self.variables);

                        let entry = next_level_map.entry(raw_combined).or_insert_with(HashSet::new);
                        entry.extend(&current_level[i].covered_minterms);
                        entry.extend(&current_level[j].covered_minterms);
                    }
                }
            }

            // Convert back to Implicants with proper covered_minterms
            for (raw_value, covered) in next_level_map {
                let mut combined_imp = Implicant::<E>::from_raw_encoding(raw_value, self.variables);
                combined_imp.covered_minterms = covered;
                next_level.push(combined_imp);
            }

            if self.logging_on {
                println!(
                    "Level {order}: next_level size = {}, prime_implicants so far = {}",
                    next_level.len(),
                    prime_implicants.len()
                );
            }

            #[cfg(debug_assertions)]
            validate_prime_implicants(&next_level, self.variables);

            for (i, implicant) in current_level.into_iter().enumerate() {
                if !used[i] {
                    prime_implicants.push(implicant);
                }
            }

            current_level = next_level;
            order += 1;
        }

        self.solution_steps
            .push(format!("Found {} prime implicants", prime_implicants.len()));

        #[cfg(debug_assertions)]
        validate_prime_implicants(&prime_implicants, self.variables);

        prime_implicants
    }

    /// Find essential prime implicants (those that uniquely cover certain minterms).
    ///
    /// Essential prime implicants are those that are the only ones covering
    /// specific minterms. These must be included in any minimal solution.
    /// Returns (all_prime_implicants, essential_prime_implicants)
    pub fn find_essential_prime_implicants(&mut self) -> (Vec<Implicant<E>>, Vec<Implicant<E>>) {
        let all_pis: Vec<Implicant<E>> = self.find_prime_implicants();

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

        let essential_pis: Vec<Implicant<E>> = all_pis
            .iter()
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
            self.solution_steps.len() + 1,
            essential_pis.len()
        ));

        (all_pis, essential_pis)
    }

    /// Get the step-by-step description of the minimization process.
    ///
    /// Returns the sequence of operations performed during the last
    /// `find_prime_implicants()` call, including initial minterms,
    /// processing levels, and final count.
    pub fn get_solution_steps(&self) -> &[String] {
        &self.solution_steps
    }
}

/// Validate a list of prime implicants for correctness.
///
/// 1. **Duplicates** — two implicants with the same raw encoding should not appear.
/// 2. **Don't-care consistency** — every entry in `covered_minterms` must have
///    0 in every bit position that is marked as DontCare in the implicant.
///
/// Prints warnings and triggers an `int 3` breakpoint if anything is wrong.
/// Only active in debug builds via `#[cfg(debug_assertions)]` guards on callers.
#[allow(dead_code)]
pub fn validate_prime_implicants<E: MintermEncoding>(
    implicants: &[Implicant<E>],
    variables: usize,
) {
    let mut seen = std::collections::HashMap::new();
    for (idx, pi) in implicants.iter().enumerate() {
        let raw: E::Value = pi.bits;

        validate_prime_implicant::<E>(&raw, variables);

        if let Some(first) = seen.insert(raw, idx) {
            println!(
                "validate_prime_implicants: duplicate PI {} at index {} matches first at index {}",
                idx, idx, first
            );
            int3();
            break;
        }
    }
}

pub fn validate_prime_implicant<E: MintermEncoding>(raw: &E::Value, variables: usize) {
    for i in 0..variables {
        let dont_know = raw.get_bit(i + variables);
        let data_bit = raw.get_bit(i);

        if dont_know && !data_bit {
            println!(
                "validate_prime_implicants: DontCare bit is set while data bit is cleared. {:032b}",
                raw.to_u64()
            );
            int3();
        }
    }
}
