//! Classic Quine-McCluskey implementation with C++ API compatibility
//!
//! This module provides utility functions and backward-compatible exports
//! for the QM algorithm. The main types have been moved to separate modules.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::cnf_dnf::{self, OptimizedFor};

// Re-export encoding types for backward compatibility
pub use super::encoding::{BitOps, Enc16, Enc32, Enc64, MintermEncoding};
pub use super::minterm_set::MintermSet;

// Constants
pub const DONT_KNOW: char = 'X';

/// Convert minterm to formula string
pub fn minterm_to_formula<E: MintermEncoding>(
    number_vars: usize,
    minterm: E::Value,
    names: &[String],
) -> String {
    let mut result = String::new();
    let mut first = true;

    for i in 0..number_vars {
        let pos = (number_vars - 1 - i) + E::DK_OFFSET;
        let variable_name = &names[i];

        if !minterm.get_bit(pos) {
            if first {
                first = false;
            } else {
                result.push_str(" & ");
            }

            if minterm.get_bit(number_vars - 1 - i) {
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
pub fn minterm_to_string<E: MintermEncoding>(
    number_vars: usize,
    minterm: E::Value,
) -> String {
    let mut result = vec![DONT_KNOW; number_vars];

    for i in 0..number_vars {
        let pos = (number_vars - i) - 1;
        let pos_x = pos + E::DK_OFFSET;

        if !minterm.get_bit(pos_x) {
            result[i] = if minterm.get_bit(pos) { '1' } else { '0' };
        }
    }

    result.iter().collect()
}

/// Convert multiple minterms to multiple strings
pub fn minterms_to_strings<E: MintermEncoding>(
    number_vars: usize,
    minterms: &[E::Value],
) -> Vec<String> {
    if number_vars > E::MAX_VARS {
        eprintln!("ERROR: max number of vars is {}", E::MAX_VARS);
        return Vec::new();
    }

    minterms
        .iter()
        .map(|&minterm| minterm_to_string::<E>(number_vars, minterm))
        .collect()
}

/// Convert multiple minterms to single string
pub fn minterms_to_string<E: MintermEncoding>(
    number_vars: usize,
    minterms: &[E::Value],
) -> String {
    minterms_to_strings::<E>(number_vars, minterms).join(" ")
}

/// Check if two values form a gray code pair (differ by exactly one bit)
#[inline]
pub fn is_gray_code<E: MintermEncoding>(a: E::Value, b: E::Value) -> bool {
    (a ^ b).count_ones() == 1
}

/// Replace complement terms with don't cares
///
/// The encoding type `E` determines the shift amount at compile time,
/// allowing the compiler to generate optimal code with no branches.
///
/// # Type Parameters
/// * `E` - The encoding type (Encoding16 or Encoding32) which determines
///   both the value type (u32 or u64) and the don't-care offset
#[inline]
pub fn replace_complements<E: MintermEncoding>(a: E::Value, b: E::Value) -> E::Value {
    let neq = a ^ b;
    a | neq | (neq << E::DK_OFFSET)
}

/// Reduce minterms using classic O(n²) algorithm
pub fn reduce_minterms_classic<E: MintermEncoding>(
    minterms: &[E::Value],
    n_variables: usize,
    show_info: bool,
) -> Vec<E::Value> {
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
            if is_gray_code::<E>(term_i, term_j) {
                checked[i] = true;
                checked[j] = true;
                let new_mt = replace_complements::<E>(term_i, term_j);

                if show_info {
                    println!("INFO: 09f28d3a: term_i: {}", minterm_to_string::<E>(n_variables, term_i));
                    println!("INFO: 2d17146f: term_j: {}", minterm_to_string::<E>(n_variables, term_j));
                    println!("INFO: 313a49ea: new_mt: {}", minterm_to_string::<E>(n_variables, new_mt));
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
                         minterm_to_string::<E>(n_variables, minterms[i]));
            }
            new_minterms.insert(minterms[i]);
        }
    }

    new_minterms.into_iter().collect()
}

/// Reduce minterms using an optimized algorithm
pub fn reduce_minterms<E: MintermEncoding>(minterms: &[E::Value], show_info: bool) -> Vec<E::Value> {
    let mut total_comparisons = 0u64;
    let mut set = MintermSet::<E>::new();
    set.add_all(minterms);

    let mut new_minterms = BTreeSet::new();
    let max_bit_count = set.get_max_bit_count();

    let mut checked_x: Vec<Vec<bool>> = Vec::new();
    for bit_count in 0..=max_bit_count {
        let size = set.get(bit_count).len();
        checked_x.push(vec![false; size]);
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

                if is_gray_code::<E>(term_i, term_j) {
                    checked_x[bit_count][i] = true;
                    checked_x[bit_count + 1][j] = true;
                    let new_mt = replace_complements::<E>(term_i, term_j);

                    if show_info {
                        println!("INFO: 09f28d3a: term_i: {}", minterm_to_string::<E>(3, term_i));
                        println!("INFO: 2d17146f: term_j: {}", minterm_to_string::<E>(3, term_j));
                        println!("INFO: 313a49ea: new_mt: {}", minterm_to_string::<E>(3, new_mt));
                    }

                    new_minterms.insert(new_mt);
                }
            }
        }
    }

    if show_info {
        println!("INFO: 393bb38d: total_comparisons = {}", total_comparisons);
    }

    let mut result: Vec<E::Value> = new_minterms.into_iter().collect();

    for bit_count in 0..=max_bit_count {
        let checked_i = &checked_x[bit_count];
        let minterms_i = set.get(bit_count);

        for i in 0..checked_i.len() {
            if !checked_i[i] {
                result.push(minterms_i[i]);
            }
        }
    }

    result
}

/// Reduce minterms using an optimized algorithm with early pruning
pub fn reduce_minterms_with_early_pruning<E: MintermEncoding>(
    minterms: &[E::Value],
    _show_info: bool,
) -> Vec<E::Value> {
    let mut set = MintermSet::<E>::new();
    for &minterm in minterms {
        set.add(minterm);
    }

    let mut new_minterms = BTreeSet::new();
    let max_bit_count = set.get_max_bit_count();

    let mut checked_x: Vec<Vec<bool>> = Vec::new();
    for bit_count in 0..=max_bit_count {
        let size = set.get(bit_count).len();
        checked_x.push(vec![false; size]);
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
                    if is_gray_code::<E>(term_i, term_j) {
                        checked_x[bit_count][i] = true;
                        checked_x[bit_count + 1][j] = true;
                        let new_mt = replace_complements::<E>(term_i, term_j);
                        new_minterms.insert(new_mt);

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
        let checked_i = &checked_x[bit_count];
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

    pub type PITable1<E> = BTreeMap<E, HashSet<E>>;
    pub type PITable2<E> = BTreeMap<E, HashSet<E>>;

    /// Convert prime implicant to string formula
    pub fn prime_implicant_to_string<E: MintermEncoding>(
        pi: E::Value,
        n_variables: usize,
        names: &[String],
    ) -> String {
        let mut result = String::new();

        for i in (0..n_variables).rev() {
            if !pi.get_bit(i + E::DK_OFFSET) {
                result.push_str(&names[i]);
                if !pi.get_bit(i) {
                    result.push('\'');
                }
            }
        }

        result
    }

    /// Convert multiple prime implicants to formula string
    pub fn prime_implicants_to_string<E: MintermEncoding>(
        pis: &[E::Value],
        n_variables: usize,
        names: &[String],
    ) -> String {
        pis.iter()
            .map(|&pi| prime_implicant_to_string::<E>(pi, n_variables, names))
            .collect::<Vec<_>>()
            .join(" + ")
    }

    /// Convert PITable1 to PITable2
    pub fn convert<E: MintermEncoding>(pi_table: &PITable1<E::Value>) -> PITable2<E::Value> {
        let mut all_minterms = BTreeSet::new();
        for set in pi_table.values() {
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
    pub fn create_prime_implicant_table<E: MintermEncoding>(
        prime_implicants: &[E::Value],
        minterms: &[E::Value],
    ) -> PITable1<E::Value> {
        // DATA_MASK needs to match the encoding's value type
        let data_mask = if E::DK_OFFSET == 16 {
            E::Value::from_u64(0xFFFF)
        } else {
            E::Value::from_u64(0xFFFF_FFFF)
        };

        let mut results = PITable1::new();

        for &pi in prime_implicants {
            // Shift to get don't-care bits
            let dont_know = E::Value::from_u64(pi.to_u64() >> E::DK_OFFSET);
            let q = (data_mask & pi) | dont_know;

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
    pub fn to_string_pi_table1<E: MintermEncoding>(
        pi_table1: &PITable1<E::Value>,
        pi_width: usize,
    ) -> String {
        let mut all_minterms = BTreeSet::new();
        for mt_set in pi_table1.values() {
            for &minterm in mt_set {
                all_minterms.insert(minterm);
            }
        }

        let mut result = String::from("\t");
        for pi in pi_table1.keys() {
            result.push_str(&minterm_to_string::<E>(pi_width, *pi));
            result.push(' ');
        }
        result.push('\n');

        for &mt in &all_minterms {
            let mut covered_by_prime_implicants = 0;
            let mut tmp = String::new();

            for mt_set in pi_table1.values() {
                if mt_set.contains(&mt) {
                    tmp.push('X');
                    covered_by_prime_implicants += 1;
                } else {
                    tmp.push('.');
                }
            }

            result.push_str(&minterm_to_string::<E>(pi_width, mt));
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
    pub fn to_string_pi_table2<E: MintermEncoding>(
        pi_table2: &PITable2<E::Value>,
        pi_width: usize,
    ) -> String {
        if pi_table2.is_empty() {
            return String::from("Empty");
        }

        let mut all_pi = BTreeSet::new();
        for mt_set in pi_table2.values() {
            for &mt in mt_set {
                all_pi.insert(mt);
            }
        }

        let mut result = String::from("\t");
        for &pi in &all_pi {
            result.push_str(&minterm_to_string::<E>(pi_width, pi));
            result.push(' ');
        }
        result.push('\n');

        for (mt, pi_set) in pi_table2 {
            result.push_str(&minterm_to_string::<E>(pi_width, *mt));
            result.push_str("\t|");
            for &pi in &all_pi {
                result.push(if pi_set.contains(&pi) { 'X' } else { '.' });
            }
            result.push('\n');
        }

        result
    }

    /// Identify primary essential prime implicants
    pub fn identify_primary_essential_pi2<E: MintermEncoding>(
        pi_table: &PITable2<E::Value>,
    ) -> (PITable2<E::Value>, Vec<E::Value>) {
        let mut selected_pi = HashSet::new();

        for pi_set in pi_table.values() {
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
    pub fn row_dominance<E: MintermEncoding>(pi_table2: &PITable2<E::Value>) -> PITable2<E::Value> {
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
    pub fn column_dominance<E: MintermEncoding>(pi_table2: &PITable2<E::Value>) -> PITable2<E::Value> {
        let pi_table1 = convert::<E>(pi_table2);
        let all_pi: Vec<E::Value> = pi_table1.keys().copied().collect();
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
    ///
    /// Note: This method is limited to at most 64 prime implicants due to the
    /// u64-based CNF representation. Automatically selects optimization based on encoding type.
    pub fn petricks_method<E: MintermEncoding>(
        pi_table2: &PITable2<E::Value>,
        show_info: bool,
    ) -> Vec<Vec<E::Value>> {

        // Create translation maps
        let mut translation1: HashMap<E::Value, usize> = HashMap::new();
        let mut translation2: HashMap<usize, E::Value> = HashMap::new();
        let mut variable_id = 0;

        for pi_set in pi_table2.values() {
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
            eprintln!(
                "ERROR: too many prime implicants ({}) for cnf_to_dnf (max 64)",
                n_variables
            );
            return Vec::new();
        }

        // Convert PI table to CNF (limited to u64 representation)
        let mut cnf: Vec<u64> = Vec::new();
        for pi_set in pi_table2.values() {
            let mut disjunction = 0u64;
            for &pi in pi_set {
                disjunction |= 1u64 << translation1[&pi];
            }
            cnf.push(disjunction);
        }

        if show_info {
            println!("CNF = {}", cnf_dnf::cnf_to_string(&cnf));
        }

        // Convert CNF to DNF using encoding-aware API
        // Note: CNF is always u64-based, so we use Encoding64 for up to 64 variables
        let smallest_conjunctions = if n_variables <= 16 {
            cnf_dnf::convert_cnf_to_dnf_minimal::<crate::qm::Enc16, {OptimizedFor::AutoDetect}>(&cnf, n_variables, false)
        } else if n_variables <= 32 {
            cnf_dnf::convert_cnf_to_dnf_minimal::<crate::qm::Enc32, {OptimizedFor::AutoDetect}>(&cnf, n_variables, false)
        } else {
            cnf_dnf::convert_cnf_to_dnf_minimal::<crate::qm::Enc64, {OptimizedFor::AutoDetect}>(&cnf, n_variables, false)
        };

        if show_info {
            println!("DNF = {}", cnf_dnf::dnf_to_string(&smallest_conjunctions));
        }

        // Translate the smallest conjunctions back
        let mut result = Vec::new();
        for conj in smallest_conjunctions {
            let mut x = Vec::new();
            for i in 0..64 {
                if (conj >> i) & 1 == 1
                    && let Some(&pi) = translation2.get(&i) {
                        x.push(pi);
                    }
            }
            result.push(x);
        }

        result
    }

    /// Petrick simplification
    ///
    /// Automatically selects optimization based on encoding type.
    pub fn petrick_simplify<E: MintermEncoding>(
        prime_implicants: &[E::Value],
        minterms: &[E::Value],
        n_bits: usize,
        use_petrick_cnf2dnf: bool,
        show_info: bool,
    ) -> Vec<E::Value> {
        // 1. Create prime implicant table
        let pi_table1 = create_prime_implicant_table::<E>(prime_implicants, minterms);
        if show_info {
            println!("1] created PI table: number of PIs = {}", pi_table1.len());
            println!("{}", to_string_pi_table1::<E>(&pi_table1, n_bits));
        }

        // 2. Identify primary essential prime implicants
        let (pi_table2, primary_essential_pi) = identify_primary_essential_pi2::<E>(&convert::<E>(&pi_table1));
        if show_info {
            println!("2] identified primary essential PIs: number of essential PIs = {}; number of remaining PIs = {}",
                     primary_essential_pi.len(), pi_table2.len());
            println!("{}", to_string_pi_table2::<E>(&pi_table2, n_bits));
        }

        // 3. Row dominance
        let pi_table3 = row_dominance::<E>(&pi_table2);
        if show_info {
            println!("3] reduced based on row dominance: number of PIs remaining = {}", pi_table3.len());
            println!("{}", to_string_pi_table2::<E>(&pi_table3, n_bits));
        }

        // 4. Column dominance
        let pi_table4 = column_dominance::<E>(&pi_table3);
        if show_info {
            println!("4] reduced based on column dominance: number of PIs remaining = {}", pi_table4.len());
            println!("{}", to_string_pi_table2::<E>(&pi_table4, n_bits));
        }

        // 5. Identify secondary essential prime implicants
        let (pi_table5, secondary_essential_pi) = identify_primary_essential_pi2::<E>(&pi_table4);
        if show_info {
            println!("5] identified secondary essential PIs: number of essential PIs = {}; number of remaining PIs = {}",
                     secondary_essential_pi.len(), pi_table5.len());
            println!("{}", to_string_pi_table2::<E>(&pi_table5, n_bits));
        }

        // 6. Row dominance
        let pi_table6 = row_dominance::<E>(&pi_table5);
        if show_info {
            println!("6] reduced based on row dominance: number of PIs remaining = {}", pi_table6.len());
            println!("{}", to_string_pi_table2::<E>(&pi_table6, n_bits));
        }

        // 7. Column dominance
        let pi_table7 = column_dominance::<E>(&pi_table6);
        if show_info {
            println!("7] reduced based on column dominance: number of PIs remaining = {}", pi_table7.len());
            println!("{}", to_string_pi_table2::<E>(&pi_table7, n_bits));
        }

        let mut essential_pi = Vec::new();

        if !pi_table7.is_empty() {
            if use_petrick_cnf2dnf {
                let pi_vector_petricks = petricks_method::<E>(&pi_table7, show_info);
                if !pi_vector_petricks.is_empty() {
                    essential_pi.extend_from_slice(&pi_vector_petricks[0]);
                }
                if show_info {
                    println!("8] reduce with Petricks method: number essential PIs = {}", essential_pi.len());
                }
            } else {
                let mut pi_set = BTreeSet::new();
                for pi_set2 in pi_table7.values() {
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
                         minterm_to_string::<E>(n_bits, pi));
            }
            essential_pi.push(pi);
        }

        for &pi in &secondary_essential_pi {
            if show_info {
                println!("INFO: e2c83d65: adding secondary essential PI to result: {}",
                         minterm_to_string::<E>(n_bits, pi));
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
///
/// If `of` is None, uses the encoding's recommended OptimizedFor variant.
/// The OptimizedFor parameter is only used when Petrick's method with CNF-to-DNF is enabled.
pub fn reduce_qm<E: MintermEncoding>(
    minterms_input: &[E::Value],
    n_variables: usize,
    use_classic_method: bool,
    use_petrick_simplify: bool,
    use_petrick_cnf2dnf: bool,
    of: Option<OptimizedFor>,
    show_info: bool,
) -> Vec<E::Value> {
    // Validate encoding compatibility
    if n_variables > E::MAX_VARS {
        eprintln!(
            "ERROR: n_variables ({}) exceeds encoding maximum ({})",
            n_variables,
            E::MAX_VARS
        );
        return Vec::new();
    }

    // Validate OptimizedFor if provided
    if let Some(optimized_for) = of
        && !E::is_compatible_with(optimized_for) {
            eprintln!(
                "WARNING: OptimizedFor {:?} (max {} bits) may be incompatible with {} variables",
                optimized_for,
                optimized_for.max_bits(),
                n_variables
            );
        }
    let mut minterms = minterms_input.to_vec();
    let mut iteration = 0;
    let mut fixed_point = false;

    while !fixed_point {
        let next_minterms = if use_classic_method {
            reduce_minterms_classic::<E>(&minterms, n_variables, show_info)
        } else {
            reduce_minterms::<E>(&minterms, show_info)
        };

        fixed_point = minterms == next_minterms;

        if show_info {
            println!("INFO: 361a49a4: reduce_qm: iteration {}; minterms {}; next minterms {}",
                     iteration, minterms.len(), next_minterms.len());
            println!("INFO: 49ecfd1e: old minterms = {}", minterms_to_string::<E>(n_variables, &minterms));
            println!("INFO: ed11b7c0: new minterms = {}", minterms_to_string::<E>(n_variables, &next_minterms));
        }

        iteration += 1;
        minterms = next_minterms;
    }

    if use_petrick_simplify {
        petrick::petrick_simplify::<E>(&minterms, minterms_input, n_variables, use_petrick_cnf2dnf, show_info)
    } else {
        minterms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_gray_code_32bit() {
        assert!(is_gray_code::<Enc32>(0b00u64, 0b01u64));
        assert!(is_gray_code::<Enc32>(0b01u64, 0b11u64));
        assert!(!is_gray_code::<Enc32>(0b00u64, 0b11u64));
    }

    #[test]
    fn test_is_gray_code_16bit() {
        assert!(is_gray_code::<Enc16>(0b00u32, 0b01u32));
        assert!(is_gray_code::<Enc16>(0b01u32, 0b11u32));
        assert!(!is_gray_code::<Enc16>(0b00u32, 0b11u32));
    }

    #[test]
    fn test_minterm_to_string_32bit() {
        let result = minterm_to_string::<Enc32>(3, 0b101u64);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_minterm_to_string_16bit() {
        let result = minterm_to_string::<Enc16>(3, 0b101u32);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_minterm_set_32bit() {
        let mut set = MintermSet::<Enc32>::new();
        set.add(0b101u64);
        set.add(0b011u64);
        assert_eq!(set.get_max_bit_count(), 2);
    }

    #[test]
    fn test_minterm_set_16bit() {
        let mut set = MintermSet::<Enc16>::new();
        set.add(0b101u32);
        set.add(0b011u32);
        assert_eq!(set.get_max_bit_count(), 2);
    }

    #[test]
    fn test_replace_complements_32bit() {
        let result_32 = replace_complements::<Enc32>(0b0110u64, 0b0111u64);
        // The result should have don't care bits set in the upper half
        assert_ne!(result_32, 0);
    }

    #[test]
    fn test_replace_complements_16bit() {
        let result_16 = replace_complements::<Enc16>(0b0110u32, 0b0111u32);
        assert_ne!(result_16, 0);
    }

    #[test]
    fn test_is_gray_code_64bit() {
        assert!(is_gray_code::<Enc64>(0b00u128, 0b01u128));
        assert!(is_gray_code::<Enc64>(0b01u128, 0b11u128));
        assert!(!is_gray_code::<Enc64>(0b00u128, 0b11u128));
    }

    #[test]
    fn test_minterm_to_string_64bit() {
        let result = minterm_to_string::<Enc64>(3, 0b101u128);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_minterm_set_64bit() {
        let mut set = MintermSet::<Enc64>::new();
        set.add(0b101u128);
        set.add(0b011u128);
        assert_eq!(set.get_max_bit_count(), 2);
    }

    #[test]
    fn test_replace_complements_64bit() {
        let result_64 = replace_complements::<Enc64>(0b0110u128, 0b0111u128);
        // The result should have don't care bits set in the upper half
        assert_ne!(result_64, 0);
    }

    #[test]
    fn test_both_modes() {
        // Test that both 16-bit and 32-bit modes work correctly
        let minterms_32: Vec<u64> = vec![0b001, 0b010, 0b110, 0b111];
        let minterms_16: Vec<u32> = vec![0b001, 0b010, 0b110, 0b111];

        // 32-bit mode
        let result_32 = reduce_minterms::<Enc32>(&minterms_32, false);
        assert!(!result_32.is_empty());

        // 16-bit mode
        let result_16 = reduce_minterms::<Enc16>(&minterms_16, false);
        assert!(!result_16.is_empty());

        // Results should be the same for small problems
        assert_eq!(result_32.len(), result_16.len());
    }

    #[test]
    fn test_encoding_compatibility() {
        // Test Encoding16 compatibility
        assert!(Enc16::is_compatible_with(OptimizedFor::Avx512_16bits));
        assert!(Enc16::is_compatible_with(OptimizedFor::Avx512_32bits));
        assert!(Enc16::is_compatible_with(OptimizedFor::Avx512_64bits));
        assert!(!Enc16::is_compatible_with(OptimizedFor::Avx512_8bits));

        // Test Encoding32 compatibility
        assert!(!Enc32::is_compatible_with(OptimizedFor::Avx512_8bits));
        assert!(!Enc32::is_compatible_with(OptimizedFor::Avx512_16bits));
        assert!(Enc32::is_compatible_with(OptimizedFor::Avx512_32bits));
        assert!(Enc32::is_compatible_with(OptimizedFor::Avx512_64bits));

        // Test Encoding64 compatibility
        assert!(!Enc64::is_compatible_with(OptimizedFor::Avx512_8bits));
        assert!(!Enc64::is_compatible_with(OptimizedFor::Avx512_16bits));
        assert!(!Enc64::is_compatible_with(OptimizedFor::Avx512_32bits));
        assert!(Enc64::is_compatible_with(OptimizedFor::Avx512_64bits));
    }

    #[test]
    fn test_recommended_optimized_for() {
        // Test that each encoding recommends the correct OptimizedFor
        assert_eq!(Enc16::recommended_optimized_for(), OptimizedFor::Avx512_16bits);
        assert_eq!(Enc32::recommended_optimized_for(), OptimizedFor::Avx512_32bits);
        assert_eq!(Enc64::recommended_optimized_for(), OptimizedFor::Avx512_64bits);
    }

    #[test]
    fn test_reduce_qm_validation() {
        // Test that reduce_qm rejects too many variables
        let minterms: Vec<u32> = vec![1, 3];
        let result = reduce_qm::<Enc16>(
            &minterms,
            20, // Exceeds MAX_VARS for Encoding16 (16)
            false,
            false,
            false,
            None,
            false,
        );
        assert!(result.is_empty()); // Should return empty due to validation failure

        // Test that reduce_qm accepts valid variable count
        let result = reduce_qm::<Enc16>(
            &minterms,
            8, // Within MAX_VARS for Encoding16
            false,
            false,
            false,
            None,
            false,
        );
        assert!(!result.is_empty()); // Should succeed
    }
}
