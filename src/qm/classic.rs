use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::cnf_dnf;
use crate::cnf_dnf::OptimizedFor;

// Constants
pub const MAX_16_BITS: bool = false;
pub const DONT_KNOW: char = 'X';

/// Convert minterm to formula string
pub fn minterm_to_formula<T: Into<u64> + Copy>(
    number_vars: usize,
    minterm: T,
    names: &[String],
) -> String {
    let minterm_val: u64 = minterm.into();
    let mut result = String::new();
    let mut first = true;

    let n_bits = if MAX_16_BITS { 16 } else { 32 };

    for i in 0..number_vars {
        let pos = (number_vars - 1 - i) + n_bits;
        let variable_name = &names[i];

        if !get_bit(minterm_val, pos) {
            if first {
                first = false;
            } else {
                result.push_str(" & ");
            }

            if get_bit(minterm_val, number_vars - 1 - i) {
                result.push_str(variable_name);
            } else {
                result.push('~');
                result.push_str(variable_name);
            }
        }
    }
    result
}

/// Convert one minterm to string representation
pub fn minterm_to_string<T: Into<u64> + Copy>(number_vars: usize, minterm: T) -> String {
    let minterm_val: u64 = minterm.into();
    let dk_offset = if MAX_16_BITS { 16 } else { 32 };
    let mut result = vec![DONT_KNOW; number_vars];

    for i in 0..number_vars {
        let pos = (number_vars - i) - 1;
        let pos_x = pos + dk_offset;

        if !get_bit(minterm_val, pos_x) {
            result[i] = if get_bit(minterm_val, pos) { '1' } else { '0' };
        }
    }

    result.iter().collect()
}

/// Convert multiple minterms to multiple strings
pub fn minterms_to_strings<T: Into<u64> + Copy>(
    number_vars: usize,
    minterms: &[T],
) -> Vec<String> {
    if number_vars > 16 {
        eprintln!("ERROR: max number of vars is 16");
        return Vec::new();
    }

    minterms
        .iter()
        .map(|&minterm| minterm_to_string(number_vars, minterm))
        .collect()
}

/// Convert multiple minterms to single string
pub fn minterms_to_string<T: Into<u64> + Copy>(number_vars: usize, minterms: &[T]) -> String {
    minterms_to_strings(number_vars, minterms).join(" ")
}

/// Helper function to get bit at position
#[inline]
fn get_bit(value: u64, pos: usize) -> bool {
    (value & (1u64 << pos)) != 0
}

/// MintermSet structure for organizing minterms by popcount
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MintermSet {
    data: Vec<Vec<u64>>,
    max_bit_count: usize,
}

impl MintermSet {
    pub fn new() -> Self {
        let width = if MAX_16_BITS { 33 } else { 65 };
        Self {
            data: vec![Vec::new(); width],
            max_bit_count: 0,
        }
    }

    pub fn add(&mut self, value: u64) {
        let bit_count = value.count_ones() as usize;
        if bit_count > self.max_bit_count {
            self.max_bit_count = bit_count;
        }
        self.data[bit_count].push(value);
    }

    pub fn add_all(&mut self, values: &[u64]) {
        for &value in values {
            self.add(value);
        }
    }

    pub fn get(&self, bit_count: usize) -> &[u64] {
        &self.data[bit_count]
    }

    pub fn get_max_bit_count(&self) -> usize {
        self.max_bit_count
    }
}

impl Default for MintermSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if two values form a gray code pair (differ by exactly one bit)
#[inline]
pub fn is_gray_code(a: u64, b: u64) -> bool {
    (a ^ b).count_ones() == 1
}

/// Replace complement terms with don't cares
pub fn replace_complements(a: u64, b: u64) -> u64 {
    let neq = a ^ b;
    if MAX_16_BITS {
        a | neq | (neq << 16)
    } else {
        a | neq | (neq << 32)
    }
}

/// Reduce minterms using classic O(n²) algorithm
#[allow(non_snake_case)]
pub fn reduce_minterms_CLASSIC(
    minterms: &[u64],
    n_variables: usize,
    show_info: bool,
) -> Vec<u64> {
    let mut total_comparisons = 0u64;
    let max = minterms.len();
    let mut checked = vec![false; max];
    let mut new_minterms = BTreeSet::new();

    for i in 0..max {
        let term_i = minterms[i];
        for j in (i + 1)..max {
            if show_info {
                total_comparisons += 1;
            }

            let term_j = minterms[j];
            if is_gray_code(term_i, term_j) {
                checked[i] = true;
                checked[j] = true;
                let new_mt = replace_complements(term_i, term_j);

                if show_info {
                    println!("INFO: 09f28d3a: term_i: {}", minterm_to_string(n_variables, term_i));
                    println!("INFO: 2d17146f: term_j: {}", minterm_to_string(n_variables, term_j));
                    println!("INFO: 313a49ea: new_mt: {}", minterm_to_string(n_variables, new_mt));
                }

                new_minterms.insert(new_mt);
            }
        }
    }

    if show_info {
        println!("INFO: 393bb38d: total_comparisons = {}", total_comparisons);
    }

    for i in 0..max {
        if !checked[i] {
            if show_info {
                println!("INFO: 6dc50c80: adding existing minterm: {}",
                         minterm_to_string(n_variables, minterms[i]));
            }
            new_minterms.insert(minterms[i]);
        }
    }

    new_minterms.into_iter().collect()
}

/// Reduce minterms using optimized algorithm
#[allow(non_snake_case)]
pub fn reduce_minterms(minterms: &[u64], show_info: bool) -> Vec<u64> {
    let mut total_comparisons = 0u64;
    let mut set = MintermSet::new();
    set.add_all(minterms);

    let mut new_minterms = BTreeSet::new();
    let max_bit_count = set.get_max_bit_count();

    let mut checked_X: Vec<Vec<bool>> = Vec::new();
    for bit_count in 0..=max_bit_count {
        let size = set.get(bit_count).len();
        checked_X.push(vec![false; size]);
    }

    for bit_count in 0..max_bit_count {
        let minterms_i = set.get(bit_count);
        let minterms_j = set.get(bit_count + 1);
        let max_i = minterms_i.len();
        let max_j = minterms_j.len();

        total_comparisons += (max_i * max_j) as u64;

        if show_info {
            println!("INFO: 413d6ad8: max_i = {}; max_j = {}; total_comparisons = {}",
                     max_i, max_j, total_comparisons);
        }

        for i in 0..max_i {
            let term_i = minterms_i[i];

            for j in 0..max_j {
                let term_j = minterms_j[j];

                if is_gray_code(term_i, term_j) {
                    checked_X[bit_count][i] = true;
                    checked_X[bit_count + 1][j] = true;
                    let new_mt = replace_complements(term_i, term_j);

                    if show_info {
                        println!("INFO: 09f28d3a: term_i: {}", minterm_to_string(3, term_i));
                        println!("INFO: 2d17146f: term_j: {}", minterm_to_string(3, term_j));
                        println!("INFO: 313a49ea: new_mt: {}", minterm_to_string(3, new_mt));
                    }

                    new_minterms.insert(new_mt);
                }
            }
        }
    }

    if show_info {
        println!("INFO: 393bb38d: total_comparisons = {}", total_comparisons);
    }

    let mut result: Vec<u64> = new_minterms.into_iter().collect();

    for bit_count in 0..=max_bit_count {
        let checked_i = &checked_X[bit_count];
        let minterms_i = set.get(bit_count);

        for i in 0..checked_i.len() {
            if !checked_i[i] {
                result.push(minterms_i[i]);
            }
        }
    }

    result
}

/// Reduce minterms using optimized algorithm with early pruning
#[allow(non_snake_case)]
pub fn reduce_minterms_X(minterms: &[u64], _show_info: bool) -> Vec<u64> {
    let mut set = MintermSet::new();
    for &minterm in minterms {
        set.add(minterm);
    }

    let mut new_minterms = BTreeSet::new();
    let max_bit_count = set.get_max_bit_count();

    let mut checked_X: Vec<Vec<bool>> = Vec::new();
    for bit_count in 0..=max_bit_count {
        let size = set.get(bit_count).len();
        checked_X.push(vec![false; size]);
    }

    for bit_count in 0..max_bit_count {
        let minterms_i = set.get(bit_count);
        let minterms_j = set.get(bit_count + 1);
        let max_i = minterms_i.len();
        let max_j = minterms_j.len();

        // Build impossible_j map for pruning
        let mut impossible_j: HashMap<usize, Vec<usize>> = HashMap::new();
        for j in 0..max_j {
            let vj = minterms_j[j];
            let mut p = Vec::new();
            for k in (j + 1)..max_j {
                let vk = minterms_j[k];
                if (vj ^ vk).count_ones() > 2 {
                    p.push(k);
                }
            }
            impossible_j.insert(j, p);
        }

        for i in 0..max_i {
            let term_i = minterms_i[i];
            let mut done = vec![false; max_j];

            for j in 0..max_j {
                if !done[j] {
                    let term_j = minterms_j[j];
                    if is_gray_code(term_i, term_j) {
                        checked_X[bit_count][i] = true;
                        checked_X[bit_count + 1][j] = true;
                        new_minterms.insert(replace_complements(term_i, term_j));

                        for &k in &impossible_j[&j] {
                            done[k] = true;
                        }
                    }
                }
            }
        }
    }

    // Add unchecked minterms
    for bit_count in 0..=max_bit_count {
        let checked_i = &checked_X[bit_count];
        let minterms_i = set.get(bit_count);

        for i in 0..checked_i.len() {
            if !checked_i[i] {
                new_minterms.insert(minterms_i[i]);
            }
        }
    }

    new_minterms.into_iter().collect()
}

pub mod petrick {
    use super::*;

    pub type PrimeImplicant = u64;
    pub type MinTerm = u64;
    pub type PITable1 = BTreeMap<PrimeImplicant, HashSet<MinTerm>>;
    pub type PITable2 = BTreeMap<MinTerm, HashSet<PrimeImplicant>>;

    /// Convert prime implicant to string formula
    pub fn prime_implicant_to_string(
        pi: PrimeImplicant,
        n_variables: usize,
        names: &[String],
    ) -> String {
        let mut result = String::new();

        for i in (0..n_variables).rev() {
            if !get_bit(pi, i + 32) {
                result.push_str(&names[i]);
                if !get_bit(pi, i) {
                    result.push('\'');
                }
            }
        }

        result
    }

    /// Convert multiple prime implicants to formula string
    pub fn prime_implicants_to_string(
        pis: &[PrimeImplicant],
        n_variables: usize,
        names: &[String],
    ) -> String {
        pis.iter()
            .map(|&pi| prime_implicant_to_string(pi, n_variables, names))
            .collect::<Vec<_>>()
            .join(" + ")
    }

    /// Convert PITable1 to PITable2
    pub fn convert(pi_table: &PITable1) -> PITable2 {
        let mut all_minterms = BTreeSet::new();
        for (_, set) in pi_table {
            for &minterm in set {
                all_minterms.insert(minterm);
            }
        }

        let mut result = PITable2::new();
        for &mt in &all_minterms {
            let mut set2 = HashSet::new();
            for (&pi, set) in pi_table {
                if set.contains(&mt) {
                    set2.insert(pi);
                }
            }
            result.insert(mt, set2);
        }

        result
    }

    /// Create prime implicant table
    pub fn create_prime_implicant_table(
        prime_implicants: &[PrimeImplicant],
        minterms: &[MinTerm],
    ) -> PITable1 {
        const DK_OFFSET: usize = if MAX_16_BITS { 16 } else { 32 };
        const DATA_MASK: u64 = 0xFFFF_FFFF;

        let mut results = PITable1::new();

        for &pi in prime_implicants {
            let dont_know = pi >> DK_OFFSET;
            let q = (DATA_MASK & pi) | dont_know;

            let mut set = HashSet::new();
            for &mt in minterms {
                if (mt | dont_know) == q {
                    set.insert(mt);
                }
            }
            results.insert(pi, set);
        }

        results
    }

    /// Convert PITable1 to string
    pub fn to_string_pi_table1(pi_table1: &PITable1, pi_width: usize) -> String {
        let mut all_minterms = BTreeSet::new();
        for (_, mt_set) in pi_table1 {
            for &minterm in mt_set {
                all_minterms.insert(minterm);
            }
        }

        let mut result = String::from("\t");
        for (pi, _) in pi_table1 {
            result.push_str(&minterm_to_string(pi_width, *pi));
            result.push(' ');
        }
        result.push('\n');

        for &mt in &all_minterms {
            let mut covered_by_prime_implicants = 0;
            let mut tmp = String::new();

            for (_, mt_set) in pi_table1 {
                if mt_set.contains(&mt) {
                    tmp.push('X');
                    covered_by_prime_implicants += 1;
                } else {
                    tmp.push('.');
                }
            }

            result.push_str(&minterm_to_string(pi_width, mt));
            if covered_by_prime_implicants == 1 {
                result.push('*');
            }
            result.push_str("\t|");
            result.push_str(&tmp);
            result.push('\n');
        }

        result
    }

    /// Convert PITable2 to string
    pub fn to_string_pi_table2(pi_table2: &PITable2, pi_width: usize) -> String {
        if pi_table2.is_empty() {
            return String::from("Empty");
        }

        let mut all_pi = BTreeSet::new();
        for (_, mt_set) in pi_table2 {
            for &mt in mt_set {
                all_pi.insert(mt);
            }
        }

        let mut result = String::from("\t");
        for &pi in &all_pi {
            result.push_str(&minterm_to_string(pi_width, pi));
            result.push(' ');
        }
        result.push('\n');

        for (mt, pi_set) in pi_table2 {
            result.push_str(&minterm_to_string(pi_width, *mt));
            result.push_str("\t|");
            for &pi in &all_pi {
                result.push(if pi_set.contains(&pi) { 'X' } else { '.' });
            }
            result.push('\n');
        }

        result
    }

    /// Identify primary essential prime implicants
    pub fn identify_primary_essential_pi2(
        pi_table: &PITable2,
    ) -> (PITable2, Vec<PrimeImplicant>) {
        let mut selected_pi = HashSet::new();

        for (_, pi_set) in pi_table {
            if pi_set.len() == 1 {
                let pi = *pi_set.iter().next().unwrap();
                selected_pi.insert(pi);
            }
        }

        let mut mt_to_be_deleted = Vec::new();
        for (mt, pi_set) in pi_table {
            for &pi in &selected_pi {
                if pi_set.contains(&pi) {
                    mt_to_be_deleted.push(*mt);
                    break;
                }
            }
        }

        let mut pi_table_out = pi_table.clone();
        for mt in mt_to_be_deleted {
            pi_table_out.remove(&mt);
        }

        (pi_table_out, selected_pi.into_iter().collect())
    }

    /// Check if one set is a subset of another
    fn subset<T: Eq + std::hash::Hash>(sub_set: &HashSet<T>, super_set: &HashSet<T>) -> bool {
        sub_set.iter().all(|e| super_set.contains(e))
    }

    /// Apply row dominance reduction
    pub fn row_dominance(pi_table2: &PITable2) -> PITable2 {
        let mut mt_to_be_deleted = BTreeSet::new();

        for (mt1, pi_set1) in pi_table2 {
            if !mt_to_be_deleted.contains(mt1) {
                for (mt2, pi_set2) in pi_table2 {
                    if mt1 != mt2 && subset(pi_set1, pi_set2) {
                        mt_to_be_deleted.insert(*mt2);
                    }
                }
            }
        }

        let mut pi_table_out = pi_table2.clone();
        for mt in mt_to_be_deleted {
            pi_table_out.remove(&mt);
        }

        pi_table_out
    }

    /// Apply column dominance reduction
    pub fn column_dominance(pi_table2: &PITable2) -> PITable2 {
        let pi_table1 = convert(pi_table2);
        let all_pi: Vec<PrimeImplicant> = pi_table1.keys().copied().collect();
        let mut pi_to_be_deleted = HashSet::new();

        let s = all_pi.len();
        for i in 0..s {
            let pi1 = all_pi[i];
            let mt_set1 = &pi_table1[&pi1];

            for j in (i + 1)..s {
                let pi2 = all_pi[j];
                let mt_set2 = &pi_table1[&pi2];

                let q1 = subset(mt_set1, mt_set2);
                let q2 = subset(mt_set2, mt_set1);

                if q1 && q2 {
                    continue;
                } else if q1 {
                    pi_to_be_deleted.insert(pi1);
                } else if q2 {
                    pi_to_be_deleted.insert(pi2);
                }
            }
        }

        let mut result = pi_table2.clone();
        if !pi_to_be_deleted.is_empty() {
            for (_, x) in result.iter_mut() {
                for &pi in &pi_to_be_deleted {
                    x.remove(&pi);
                }
            }
        }

        result
    }

    /// Petrick's method using CNF to DNF conversion
    pub fn petricks_method(pi_table2: &PITable2, show_info: bool) -> Vec<Vec<PrimeImplicant>> {
        // Create translation maps
        let mut translation1: HashMap<PrimeImplicant, usize> = HashMap::new();
        let mut translation2: HashMap<usize, PrimeImplicant> = HashMap::new();
        let mut variable_id = 0;

        for (_, pi_set) in pi_table2 {
            for &pi in pi_set {
                if !translation1.contains_key(&pi) {
                    translation1.insert(pi, variable_id);
                    translation2.insert(variable_id, pi);
                    variable_id += 1;
                }
            }
        }

        let n_variables = variable_id;
        if n_variables > 64 {
            eprintln!("ERROR: too many variables ({}) for cnf_to_dnf", n_variables);
            return Vec::new();
        }

        // Convert PI table to CNF
        let mut cnf: Vec<u64> = Vec::new();
        for (_, pi_set) in pi_table2 {
            let mut disjunction = 0u64;
            for &pi in pi_set {
                disjunction |= 1u64 << translation1[&pi];
            }
            cnf.push(disjunction);
        }

        if show_info {
            println!("CNF = {}", cnf_dnf::cnf_to_string(&cnf));
        }

        // Convert CNF to DNF using the cnf_to_dnf module
        let smallest_conjunctions = cnf_dnf::convert_cnf_to_dnf_minimal(
            &cnf,
            n_variables,
            OptimizedFor::Avx512_64bits,
            false, // EARLY_PRUNE
        );

        if show_info {
            println!("DNF = {}", cnf_dnf::dnf_to_string(&smallest_conjunctions));
        }

        // Translate the smallest conjunctions back
        let mut result = Vec::new();
        for conj in smallest_conjunctions {
            let mut x = Vec::new();
            for i in 0..64 {
                if (conj >> i) & 1 == 1 {
                    if let Some(&pi) = translation2.get(&i) {
                        x.push(pi);
                    }
                }
            }
            result.push(x);
        }

        result
    }

    /// Petrick simplification
    pub fn petrick_simplify(
        prime_implicants: &[PrimeImplicant],
        minterms: &[MinTerm],
        n_bits: usize,
        use_petrick_cnf2dnf: bool,
        show_info: bool,
    ) -> Vec<PrimeImplicant> {
        // 1. Create prime implicant table
        let pi_table1 = create_prime_implicant_table(prime_implicants, minterms);
        if show_info {
            println!("1] created PI table: number of PIs = {}", pi_table1.len());
            println!("{}", to_string_pi_table1(&pi_table1, n_bits));
        }

        // 2. Identify primary essential prime implicants
        let (pi_table2, primary_essential_pi) = identify_primary_essential_pi2(&convert(&pi_table1));
        if show_info {
            println!("2] identified primary essential PIs: number of essential PIs = {}; number of remaining PIs = {}",
                     primary_essential_pi.len(), pi_table2.len());
            println!("{}", to_string_pi_table2(&pi_table2, n_bits));
        }

        // 3. Row dominance
        let pi_table3 = row_dominance(&pi_table2);
        if show_info {
            println!("3] reduced based on row dominance: number of PIs remaining = {}", pi_table3.len());
            println!("{}", to_string_pi_table2(&pi_table3, n_bits));
        }

        // 4. Column dominance
        let pi_table4 = column_dominance(&pi_table3);
        if show_info {
            println!("4] reduced based on column dominance: number of PIs remaining = {}", pi_table4.len());
            println!("{}", to_string_pi_table2(&pi_table4, n_bits));
        }

        // 5. Identify secondary essential prime implicants
        let (pi_table5, secondary_essential_pi) = identify_primary_essential_pi2(&pi_table4);
        if show_info {
            println!("5] identified secondary essential PIs: number of essential PIs = {}; number of remaining PIs = {}",
                     secondary_essential_pi.len(), pi_table5.len());
            println!("{}", to_string_pi_table2(&pi_table5, n_bits));
        }

        // 6. Row dominance
        let pi_table6 = row_dominance(&pi_table5);
        if show_info {
            println!("6] reduced based on row dominance: number of PIs remaining = {}", pi_table6.len());
            println!("{}", to_string_pi_table2(&pi_table6, n_bits));
        }

        // 7. Column dominance
        let pi_table7 = column_dominance(&pi_table6);
        if show_info {
            println!("7] reduced based on column dominance: number of PIs remaining = {}", pi_table7.len());
            println!("{}", to_string_pi_table2(&pi_table7, n_bits));
        }

        let mut essential_pi = Vec::new();

        if !pi_table7.is_empty() {
            if use_petrick_cnf2dnf {
                let pi_vector_petricks = petricks_method(&pi_table7, show_info);
                if !pi_vector_petricks.is_empty() {
                    essential_pi.extend_from_slice(&pi_vector_petricks[0]);
                }
                if show_info {
                    println!("8] reduce with Petricks method: number essential PIs = {}", essential_pi.len());
                }
            } else {
                let mut pi_set = BTreeSet::new();
                for (_, pi_set2) in &pi_table7 {
                    for &pi in pi_set2 {
                        pi_set.insert(pi);
                    }
                }
                essential_pi.extend(pi_set);
            }
        }

        for &pi in &primary_essential_pi {
            if show_info {
                println!("INFO: b650c460: adding primary essential PI to result: {}",
                         minterm_to_string(n_bits, pi));
            }
            essential_pi.push(pi);
        }

        for &pi in &secondary_essential_pi {
            if show_info {
                println!("INFO: e2c83d65: adding secondary essential PI to result: {}",
                         minterm_to_string(n_bits, pi));
            }
            essential_pi.push(pi);
        }

        if show_info {
            println!("INFO: 6b723975: simplify removed {} from (initially) {} PIs",
                     prime_implicants.len() - essential_pi.len(), prime_implicants.len());
        }

        essential_pi
    }
}

/// Main Quine-McCluskey reduction function
pub fn reduce_qm(
    minterms_input: &[u64],
    n_variables: usize,
    use_classic_method: bool,
    use_petrick_simplify: bool,
    use_petrick_cnf2dnf: bool,
    show_info: bool,
) -> Vec<u64> {
    let mut minterms = minterms_input.to_vec();
    let mut iteration = 0;
    let mut fixed_point = false;

    while !fixed_point {
        let next_minterms = if use_classic_method {
            reduce_minterms_CLASSIC(&minterms, n_variables, show_info)
        } else {
            reduce_minterms(&minterms, show_info)
        };

        fixed_point = minterms == next_minterms;

        if show_info {
            println!("INFO: 361a49a4: reduce_qm: iteration {}; minterms {}; next minterms {}",
                     iteration, minterms.len(), next_minterms.len());
            println!("INFO: 49ecfd1e: old minterms = {}", minterms_to_string(n_variables, &minterms));
            println!("INFO: ed11b7c0: new minterms = {}", minterms_to_string(n_variables, &next_minterms));
        }

        iteration += 1;
        minterms = next_minterms;
    }

    if use_petrick_simplify {
        petrick::petrick_simplify(&minterms, minterms_input, n_variables, use_petrick_cnf2dnf, show_info)
    } else {
        minterms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_gray_code() {
        assert!(is_gray_code(0b00, 0b01));
        assert!(is_gray_code(0b01, 0b11));
        assert!(!is_gray_code(0b00, 0b11));
    }

    #[test]
    fn test_minterm_to_string() {
        let result = minterm_to_string(3, 0b101u64);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_minterm_set() {
        let mut set = MintermSet::new();
        set.add(0b101);
        set.add(0b011);
        assert_eq!(set.get_max_bit_count(), 2);
    }

    #[test]
    fn test_replace_complements() {
        let result = replace_complements(0b0110, 0b0111);
        // The result should have don't care bits set in the upper half
        assert_ne!(result, 0);
    }
}
