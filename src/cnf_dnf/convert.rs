use std::collections::{HashMap, HashSet};

/// Number of bits in a u64
const U64_BITS: usize = std::mem::size_of::<u64>() * 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizedFor {
    X64,
    Avx512_64bits,
    Avx512_32bits,
    Avx512_16bits,
    Avx512_8bits,
    Avx2_64bits,
}

impl OptimizedFor {
    /// Returns the maximum number of bits this optimization level can handle
    pub const fn max_bits(self) -> usize {
        match self {
            Self::Avx512_8bits => 8,
            Self::Avx512_16bits => 16,
            Self::Avx512_32bits => 32,
            Self::Avx512_64bits | Self::Avx2_64bits | Self::X64 => 64,
        }
    }
}

/// Test if a bit is set at a given position
#[inline]
fn test_bit<T: Into<u64> + Copy>(data: T, pos: usize) -> bool {
    let val: u64 = data.into();
    (val >> pos) & 1 == 1
}

/// Check if we should add z, and return indices to delete (scalar version)
pub(crate) fn optimized_for_x64(
    result_dnf_next: &[u64],
    z: u64,
) -> (Vec<usize>, bool) {
    let mut index_to_delete = Vec::new();

    for (index, &q) in result_dnf_next.iter().enumerate() {
        let p = z | q;

        if p == z {
            // z is subsumed under q: no need to add z
            return (Vec::new(), false);
        }

        if p == q {
            // q is subsumed under z: add z and remove q
            index_to_delete.push(index);
        }
    }

    (index_to_delete, true)
}

/// Run the appropriate optimization based on the OptimizedFor setting
fn run_optimized(
    of: OptimizedFor,
    result_dnf_next: &[u64],
    z: u64,
) -> (Vec<usize>, bool) {
    match of {
        OptimizedFor::X64 => optimized_for_x64(result_dnf_next, z),
        #[cfg(target_arch = "x86_64")]
        OptimizedFor::Avx512_64bits => {
            super::simd::run_avx512_64bits(result_dnf_next, z)
        }
        #[cfg(target_arch = "x86_64")]
        OptimizedFor::Avx512_32bits => {
            super::simd::run_avx512_32bits(result_dnf_next, z)
        }
        #[cfg(target_arch = "x86_64")]
        OptimizedFor::Avx512_16bits => {
            super::simd::run_avx512_16bits(result_dnf_next, z)
        }
        #[cfg(target_arch = "x86_64")]
        OptimizedFor::Avx512_8bits => {
            super::simd::run_avx512_8bits(result_dnf_next, z)
        }
        #[cfg(target_arch = "x86_64")]
        OptimizedFor::Avx2_64bits => {
            super::simd::run_avx2_64bits(result_dnf_next, z)
        }
        #[cfg(not(target_arch = "x86_64"))]
        OptimizedFor::Avx512_64bits => optimized_for_x64(result_dnf_next, z),
        #[cfg(not(target_arch = "x86_64"))]
        OptimizedFor::Avx512_32bits => optimized_for_x64(result_dnf_next, z),
        #[cfg(not(target_arch = "x86_64"))]
        OptimizedFor::Avx512_16bits => optimized_for_x64(result_dnf_next, z),
        #[cfg(not(target_arch = "x86_64"))]
        OptimizedFor::Avx512_8bits => optimized_for_x64(result_dnf_next, z),
        #[cfg(not(target_arch = "x86_64"))]
        OptimizedFor::Avx2_64bits => optimized_for_x64(result_dnf_next, z),
    }
}

/// Convert CNF to DNF
pub fn convert_cnf_to_dnf<T: Into<u64> + Copy>(
    cnf: &[T],
    n_bits: usize,
    of: OptimizedFor,
) -> Vec<u64> {
    let max_n_bits = of.max_bits();

    if n_bits > max_n_bits {
        eprintln!("ERROR: nbits {} is too large", n_bits);
        return Vec::new();
    }

    let mut result_dnf: Vec<u64> = Vec::new();
    let mut first = true;

    for &disjunction in cnf {
        let disj_val: u64 = disjunction.into();
        if first {
            first = false;
            for i in 0..n_bits {
                if test_bit(disj_val, i) {
                    result_dnf.push(1u64 << i);
                }
            }
        } else {
            let mut result_dnf_next: Vec<u64> = Vec::new();

            for pos in 0..n_bits {
                if test_bit(disj_val, pos) {
                    let x = 1u64 << pos;

                    for &y in &result_dnf {
                        let z = x | y;

                        let (index_to_delete, add_z) = run_optimized(of, &result_dnf_next, z);

                        if add_z {
                            // Efficient O(n) filtering instead of O(n²) repeated removes
                            if !index_to_delete.is_empty() {
                                let delete_set: HashSet<usize> = index_to_delete.into_iter().collect();
                                result_dnf_next = result_dnf_next.into_iter()
                                    .enumerate()
                                    .filter(|(idx, _)| !delete_set.contains(idx))
                                    .map(|(_, val)| val)
                                    .collect();
                            }
                            result_dnf_next.push(z);
                        }
                    }
                }
            }

            result_dnf = result_dnf_next;
        }
    }

    result_dnf
}

/// Convert CNF to DNF and return only the minimal conjunctions
#[allow(non_snake_case)]
pub fn convert_cnf_to_dnf_minimal<T: Into<u64> + Copy>(
    cnf: &[T],
    n_bits: usize,
    of: OptimizedFor,
    EARLY_PRUNE: bool,
) -> Vec<u64> {
    let result_dnf = if EARLY_PRUNE {
        convert_cnf_to_dnf_minimal_private_method1(cnf, n_bits, of)
    } else {
        convert_cnf_to_dnf(cnf, n_bits, of)
    };

    // Select only the smallest DNFs
    if result_dnf.is_empty() {
        return result_dnf;
    }

    let mut smallest_cnf_size = usize::MAX;
    for &conjunction in &result_dnf {
        let size = conjunction.count_ones() as usize;
        if size < smallest_cnf_size {
            smallest_cnf_size = size;
        }
    }

    result_dnf
        .into_iter()
        .filter(|&conjunction| conjunction.count_ones() as usize == smallest_cnf_size)
        .collect()
}

/// Convert CNF to DNF with early pruning optimization
fn convert_cnf_to_dnf_minimal_private_method1<T: Into<u64> + Copy>(
    cnf: &[T],
    n_bits: usize,
    of: OptimizedFor,
) -> Vec<u64> {
    let n_disjunctions = cnf.len();
    let mut n_disjunction_done = 0;
    let mut result_dnf: Vec<u64> = Vec::new();

    for &disjunction in cnf {
        let disj_val: u64 = disjunction.into();
        if n_disjunction_done == 0 {
            for pos in 0..n_bits {
                if test_bit(disj_val, pos) {
                    result_dnf.push(1u64 << pos);
                }
            }
        } else {
            let mut result_dnf_next: Vec<u64> = Vec::new();
            let mut smallest_cnf_size = i32::MAX;
            let mut max_size = 0;

            for pos in 0..n_bits {
                if test_bit(disj_val, pos) {
                    let x = 1u64 << pos;

                    for &y in &result_dnf {
                        let z = x | y;

                        let mut consider_z = true;

                        // Early prune CNFs that cannot become the smallest cnf
                        let conjunction_size = z.count_ones() as i32;
                        if conjunction_size < smallest_cnf_size {
                            smallest_cnf_size = conjunction_size;
                            max_size = conjunction_size + (n_disjunctions - n_disjunction_done) as i32;
                        }
                        if max_size < conjunction_size {
                            consider_z = false;
                        }

                        if consider_z {
                            let (index_to_delete, add_z) = run_optimized(of, &result_dnf_next, z);

                            if add_z {
                                // Efficient O(n) filtering instead of O(n²) repeated removes
                                if !index_to_delete.is_empty() {
                                    let delete_set: HashSet<usize> = index_to_delete.into_iter().collect();
                                    result_dnf_next = result_dnf_next.into_iter()
                                        .enumerate()
                                        .filter(|(idx, _)| !delete_set.contains(idx))
                                        .map(|(_, val)| val)
                                        .collect();
                                }
                                result_dnf_next.push(z);
                            }
                        }
                    }
                }
            }

            result_dnf = result_dnf_next;
        }
        n_disjunction_done += 1;
    }

    result_dnf
}

/// Convert CNF to string representation
pub fn cnf_to_string(cnf: &[u64]) -> String {
    to_string(cnf, true)
}

/// Convert DNF to string representation
pub fn dnf_to_string(dnf: &[u64]) -> String {
    to_string(dnf, false)
}

/// Convert CNF or DNF to string representation
fn to_string(terms: &[u64], is_cnf: bool) -> String {
    let mut terms_vec: Vec<u64> = terms.to_vec();
    terms_vec.sort_unstable();

    let mut result = String::new();
    let mut first_disj = true;

    for disj in terms_vec {
        if first_disj {
            first_disj = false;
        } else {
            result.push_str(if is_cnf { " & " } else { " | " });
        }

        result.push('(');
        let mut first_e = true;

        for i in 0..U64_BITS {
            if test_bit(disj, i) {
                if first_e {
                    first_e = false;
                } else {
                    result.push_str(if is_cnf { "|" } else { "&" });
                }
                result.push_str(&i.to_string());
            }
        }

        result.push(')');
    }

    result
}

/// Convert CNF with string variable names to DNF
pub fn convert_cnf_to_dnf_with_names(
    cnf: &[Vec<String>],
    of: OptimizedFor,
) -> Vec<Vec<String>> {
    // Create translations
    let mut translation1: HashMap<String, usize> = HashMap::new();
    let mut translation2: HashMap<usize, String> = HashMap::new();
    let mut n_variables = 0;

    for conjunction in cnf {
        for var in conjunction {
            if !translation1.contains_key(var) {
                translation1.insert(var.clone(), n_variables);
                translation2.insert(n_variables, var.clone());
                n_variables += 1;
            }
        }
    }

    if n_variables > 64 {
        eprintln!("ERROR: too many different variables; found {} variables", n_variables);
        return Vec::new();
    }

    // Translate CNF to u64
    let mut cnf_translated: Vec<u64> = Vec::new();
    for conjunction in cnf {
        let mut v = 0u64;
        for var in conjunction {
            v |= 1u64 << translation1[var];
        }
        cnf_translated.push(v);
    }

    // Do the conversion
    let dnf = convert_cnf_to_dnf(&cnf_translated, n_variables, of);

    // Translate DNF back to strings
    let mut dnf_result: Vec<Vec<String>> = Vec::new();
    for &term in &dnf {
        let mut vars = Vec::new();
        for pos in 0..n_variables {
            if test_bit(term, pos) {
                vars.push(translation2[&pos].clone());
            }
        }
        dnf_result.push(vars);
    }

    dnf_result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_cnf_to_dnf_simple() {
        // CNF = (1|2) & (3|4)
        // DNF = (1&3) | (2&3) | (1&4) | (2&4)
        let cnf: Vec<u64> = vec![
            (1 << 1) | (1 << 2),
            (1 << 3) | (1 << 4),
        ];

        let dnf = convert_cnf_to_dnf(&cnf, 8, OptimizedFor::X64);

        assert_eq!(dnf.len(), 4);

        // Expected results
        let expected: HashSet<u64> = vec![
            (1 << 1) | (1 << 3), // 1&3
            (1 << 2) | (1 << 3), // 2&3
            (1 << 1) | (1 << 4), // 1&4
            (1 << 2) | (1 << 4), // 2&4
        ].into_iter().collect();

        let actual: HashSet<u64> = dnf.into_iter().collect();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_cnf_to_dnf_with_names() {
        let cnf = vec![
            vec!["A".to_string(), "B".to_string()],
            vec!["C".to_string(), "D".to_string()],
        ];

        let dnf = convert_cnf_to_dnf_with_names(&cnf, OptimizedFor::X64);

        assert_eq!(dnf.len(), 4);
    }

    #[test]
    fn test_minimal_dnf() {
        let cnf: Vec<u64> = vec![
            (1 << 1) | (1 << 2),
            (1 << 3) | (1 << 4),
        ];

        let dnf = convert_cnf_to_dnf_minimal(&cnf, 8, OptimizedFor::X64, false);

        // All results should have the same (minimal) number of bits set
        let first_size = dnf[0].count_ones();
        for &term in &dnf {
            assert_eq!(term.count_ones(), first_size);
        }
    }

    #[test]
    fn test_string_conversion() {
        let cnf: Vec<u64> = vec![
            (1 << 1) | (1 << 2),
            (1 << 3) | (1 << 4),
        ];

        let cnf_str = cnf_to_string(&cnf);
        assert!(cnf_str.contains("&"));
        assert!(cnf_str.contains("1"));
        assert!(cnf_str.contains("2"));
        assert!(cnf_str.contains("3"));
        assert!(cnf_str.contains("4"));
    }

    #[test]
    fn test_test_bit() {
        assert!(test_bit(0b1010u32, 1));
        assert!(!test_bit(0b1010u32, 0));
        assert!(test_bit(0b1010u32, 3));
        assert!(!test_bit(0b1010u32, 2));
    }
}
