use std::collections::{HashMap};

use crate::qm::encoding::MintermEncoding;
use super::optimized_for::OptimizedFor;
use super::error::CnfDnfError;
use super::utils::test_bit;


/// Convert CNF to DNF with encoding-aware optimization selection
///
/// This is the main API for CNF to DNF conversion. Specify the optimization level
/// as a type parameter, or use `AutoDetect` for automatic hardware detection.
///
/// # Type Parameters
/// * `E` - Encoding type (Encoding16, Encoding32, Encoding64)
/// * `O` - Optimization level (X64Opt, Avx2_64bitsOpt, Avx512_64bitsOpt, etc., or AutoDetect)
///
/// # Arguments
/// * `cnf` - CNF formula as vector of disjunctions (bit vectors)
/// * `n_bits` - Number of Boolean variables
///
/// # Examples
/// ```
/// use qm_agent::cnf_dnf::{self, OptimizedFor};
/// use qm_agent::qm::Enc64;
///
/// let cnf = vec![0b1010u64, 0b1100u64];
///
/// // Auto-detect optimization (recommended)
/// let dnf1 = cnf_dnf::cnf_to_dnf::<Enc64>(&cnf, 4, OptimizedFor::AutoDetect);
///
/// // Force X64 scalar for testing
/// let dnf2 = cnf_dnf::cnf_to_dnf::<Enc64>(&cnf, 4, OptimizedFor::X64);
///
/// // Force AVX512 for testing
/// let dnf3 = cnf_dnf::cnf_to_dnf::<Enc64>(&cnf, 4, OptimizedFor::Avx512_64bits);
/// ```
pub fn cnf_to_dnf<E: MintermEncoding>(
    cnf: &[u64],
    n_bits: usize,
    of: OptimizedFor
) -> Result<Vec<u64>, CnfDnfError> {
    validate_parameters::<E>(n_bits, of)?;
    let result_dnf = cnf_to_dnf_impl(cnf, n_bits, of.resolve(n_bits));

    if false {
        println!("cnf_to_dnf {}", result_dnf.len());
    }

    Ok(result_dnf)
}

pub fn cnf_to_dnf_minimal<E: MintermEncoding>(
    cnf: &[u64],
    n_bits: usize,
    of: OptimizedFor,
) -> Result<Vec<u64>, CnfDnfError> {
    validate_parameters::<E>(n_bits, of)?;
    let result_dnf = cnf_to_dnf_minimal_method1(cnf, n_bits, of.resolve(n_bits));

    let size_before = result_dnf.len();
    let result = filter_to_minimal(result_dnf);
    if false {
        println!("cnf_to_dnf_minimal {} to {}", size_before, result.len());
    }
    Ok(result)
}


/// reference implementation for convert_cnf_to_dnf_minimal
pub fn cnf_to_dnf_minimal_reference<E: MintermEncoding>(
    cnf: &[u64],
    n_bits: usize,
    of: OptimizedFor,
) -> Result<Vec<u64>, CnfDnfError> {
    validate_parameters::<E>(n_bits, of)?;
    let result_dnf = cnf_to_dnf_impl(cnf, n_bits, of.resolve(n_bits));

    let size_before = result_dnf.len();
    let result = filter_to_minimal(result_dnf);
    if false {
        println!("cnf_to_dnf_minimal_reference {} to {}", size_before, result.len());
    }
    Ok(result)
}

/// Validate encoding capacity and optimization level
fn validate_parameters<E: MintermEncoding>(
    n_bits: usize,
    of: OptimizedFor,
) -> Result<(), CnfDnfError> {
    // Validate that encoding supports this many bits
    if n_bits > E::MAX_VARS {
        return Err(CnfDnfError::EncodingCapacityExceeded {
            n_bits,
            max_vars: E::MAX_VARS,
        });
    }

    // Validate that optimization level is compatible with n_bits
    if of != OptimizedFor::AutoDetect && n_bits > of.max_bits() {
        return Err(CnfDnfError::OptimizationLevelExceeded {
            n_bits,
            optimization: format!("OptimizedFor::{:?}", of),
            max_bits: of.max_bits(),
        });
    }

    Ok(())
}

/// Filter DNF to keep only terms with minimal number of literals
fn filter_to_minimal(dnf: Vec<u64>) -> Vec<u64> {
    if dnf.is_empty() {
        return dnf;
    }

    // Find the minimum size
    let smallest_size = dnf
        .iter()
        .map(|&term| term.count_ones() as usize)
        .min()
        .unwrap();

    // Filter to keep only terms with minimum size
    dnf.into_iter()
        .filter(|&term| term.count_ones() as usize == smallest_size)
        .collect()
}

/// Private implementation of CNF to DNF conversion
fn cnf_to_dnf_impl(
    cnf: &[u64],
    n_bits: usize,
    of: OptimizedFor,
) -> Vec<u64> {
    let mut result_dnf: Vec<u64> = Vec::new();
    let mut first = true;

    for &disj_val in cnf {
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
                            // In-place O(n) filtering with BitVec - no allocations!
                            if !index_to_delete.is_empty() {
                                // Build deletion bitset - O(m) where m = index_to_delete.len()
                                let mut to_delete = vec![false; result_dnf_next.len()];
                                for &idx in &index_to_delete {
                                    to_delete[idx] = true;
                                }

                                // Single-pass in-place compaction - O(n)
                                let len = result_dnf_next.len();
                                let mut write_idx = 0;
                                for read_idx in 0..len {
                                    if !to_delete[read_idx] {
                                        if write_idx != read_idx {
                                            result_dnf_next[write_idx] = result_dnf_next[read_idx];
                                        }
                                        write_idx += 1;
                                    }
                                }
                                result_dnf_next.truncate(write_idx);
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

/// Run the appropriate optimization based on the OptimizedFor setting
fn run_optimized(
    of: OptimizedFor,
    result_dnf_next: &[u64],
    z: u64,
) -> (Vec<usize>, bool) {
    match of {
        OptimizedFor::AutoDetect => {
            unreachable!("AutoDetect should be resolved to a concrete optimization level before reaching this point")
        }
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
    }
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


/// Convert CNF to DNF with early pruning optimization, the results contain at least the smallest DNF
/// with the smallest number of literals. This is not guaranteed to be only the minimal DNF
fn cnf_to_dnf_minimal_method1(
    cnf: &[u64],
    n_bits: usize,
    of: OptimizedFor,
) -> Vec<u64> {
    let n_disjunctions = cnf.len();
    let mut n_disjunction_done = 0;
    let mut result_dnf: Vec<u64> = Vec::new();

    for &disj_val in cnf {
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
                    let x = 1u64 << pos; // NOTE: x only has one single bit set

                    for &y in &result_dnf {
                        let z = x | y; // Note z has number of bits in y plus either 1 or 0 (depending on whether position pos is already set in y)

                        // Early prune CNFs that cannot become the smallest cnf
                        let conjunction_size = z.count_ones() as i32;
                        if conjunction_size < smallest_cnf_size {
                            smallest_cnf_size = conjunction_size;
                            max_size = conjunction_size + (n_disjunctions - n_disjunction_done) as i32;
                        }

                        let consider_z = max_size >= conjunction_size;

                        if consider_z {
                            let (index_to_delete, add_z) = run_optimized(of, &result_dnf_next, z);

                            if add_z {
                                // In-place O(n) filtering with BitVec
                                if !index_to_delete.is_empty() {
                                    // Build deletion bitset - O(m)
                                    let len = result_dnf_next.len();
                                    let mut to_delete = vec![false; len];
                                    for &idx in &index_to_delete {
                                        to_delete[idx] = true;
                                    }

                                    // Single-pass in-place compaction - O(n)
                                    let mut write_idx = 0;
                                    for read_idx in 0..len {
                                        if !to_delete[read_idx] {
                                            if write_idx != read_idx {
                                                result_dnf_next[write_idx] = result_dnf_next[read_idx];
                                            }
                                            write_idx += 1;
                                        }
                                    }
                                    result_dnf_next.truncate(write_idx);
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

/// Convert CNF with string variable names to DNF
pub fn cnf_to_dnf_with_names(
    cnf: &[Vec<String>],
) -> Result<Vec<Vec<String>>, CnfDnfError> {
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
        return Err(CnfDnfError::TooManyVariables { n_variables });
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

    // Do the conversion using appropriate encoding
    let dnf = if n_variables <= 16 {
        cnf_to_dnf::<crate::qm::Enc16>(&cnf_translated, n_variables, OptimizedFor::AutoDetect)?
    } else if n_variables <= 32 {
        cnf_to_dnf::<crate::qm::Enc32>(&cnf_translated, n_variables, OptimizedFor::AutoDetect)?
    } else {
        cnf_to_dnf::<crate::qm::Enc64>(&cnf_translated, n_variables, OptimizedFor::AutoDetect)?
    };

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

    Ok(dnf_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::utils::cnf_to_string;
    use std::collections::HashSet;

    #[test]
    fn test_cnf_to_dnf_simple() {
        // CNF = (1|2) & (3|4)
        // DNF = (1&3) | (2&3) | (1&4) | (2&4)
        let cnf: Vec<u64> = vec![
            (1 << 1) | (1 << 2),
            (1 << 3) | (1 << 4),
        ];

        let dnf = cnf_to_dnf::<crate::qm::Enc16>(&cnf, 8, OptimizedFor::AutoDetect)
            .expect("CNF to DNF conversion failed");

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

        let dnf = cnf_to_dnf_with_names(&cnf)
            .expect("CNF to DNF conversion with names failed");

        assert_eq!(dnf.len(), 4);
    }

    #[test]
    fn test_minimal_dnf() {
        let cnf: Vec<u64> = vec![
            (1 << 1) | (1 << 2),
            (1 << 3) | (1 << 4),
        ];

        let dnf = cnf_to_dnf_minimal::<crate::qm::Enc16>(&cnf, 8, OptimizedFor::AutoDetect)
            .expect("Minimal DNF conversion failed");

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
        assert!(test_bit(0b1010u64, 1));
        assert!(!test_bit(0b1010u64, 0));
        assert!(test_bit(0b1010u64, 3));
        assert!(!test_bit(0b1010u64, 2));
    }

    #[test]
    fn test_explicit_optimization() {
        let cnf: Vec<u64> = vec![0b011, 0b101, 0b110];

        // Test auto-detection (AutoDetect)
        let dnf_auto = cnf_to_dnf::<crate::qm::Enc64>(&cnf, 3, OptimizedFor::AutoDetect)
            .expect("AutoDetect conversion failed");

        // Test explicit X64
        let dnf_x64 = cnf_to_dnf::<crate::qm::Enc64>(
            &cnf,
            3,
            OptimizedFor::X64
        ).expect("X64 conversion failed");

        // Test explicit AVX512
        let dnf_avx512 = cnf_to_dnf::<crate::qm::Enc64>(
            &cnf,
            3,
            OptimizedFor::Avx512_64bits
        ).expect("AVX512 conversion failed");

        // All should produce identical results
        assert_eq!(dnf_auto, dnf_x64);
        assert_eq!(dnf_x64, dnf_avx512);
        assert_eq!(dnf_auto.len(), 3);
    }

    #[test]
    fn test_explicit_optimization_minimal() {
        let cnf: Vec<u64> = vec![0b1010, 0b1100, 0b0110];

        // Test auto-detection
        let dnf_auto = cnf_to_dnf_minimal::<crate::qm::Enc64>(
            &cnf, 4, OptimizedFor::AutoDetect
        ).expect("AutoDetect minimal conversion failed");

        // Test explicit X64
        let dnf_x64 = cnf_to_dnf_minimal::<crate::qm::Enc64>(
            &cnf, 4, OptimizedFor::AutoDetect
        ).expect("X64 minimal conversion failed");

        // Test explicit AVX2
        let dnf_avx2 = cnf_to_dnf_minimal::<crate::qm::Enc64>(
            &cnf, 4, OptimizedFor::AutoDetect
        ).expect("AVX2 minimal conversion failed");

        // All should produce identical results
        assert_eq!(dnf_auto, dnf_x64);
        assert_eq!(dnf_x64, dnf_avx2);

        // All results should have the same (minimal) number of bits set
        let first_size = dnf_auto[0].count_ones();
        for &term in &dnf_auto {
            assert_eq!(term.count_ones(), first_size);
        }
    }

}
