use std::collections::{HashMap, HashSet};

use crate::qm::encoding::MintermEncoding;

/// Number of bits in a u64
const U64_BITS: usize = std::mem::size_of::<u64>() * 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, core::marker::ConstParamTy)]
pub enum OptimizedFor {
    /// Automatic hardware detection (runtime CPU feature detection)
    AutoDetect,
    /// X64 scalar implementation (no SIMD)
    X64,
    /// AVX-512 optimized for 64-bit elements
    Avx512_64bits,
    /// AVX-512 optimized for 32-bit elements
    Avx512_32bits,
    /// AVX-512 optimized for 16-bit elements
    Avx512_16bits,
    /// AVX-512 optimized for 8-bit elements
    Avx512_8bits,
    /// AVX2 optimized for 64-bit elements
    Avx2_64bits,
}

impl OptimizedFor {
    /// Returns the maximum number of bits this optimization level can handle
    pub const fn max_bits(self) -> usize {
        match self {
            Self::AutoDetect => 64, // AutoDetect can handle up to 64
            Self::Avx512_8bits => 8,
            Self::Avx512_16bits => 16,
            Self::Avx512_32bits => 32,
            Self::Avx512_64bits | Self::Avx2_64bits | Self::X64 => 64,
        }
    }

    /// Automatically detect the best optimization level for the current hardware
    ///
    /// This function performs runtime CPU feature detection and selects the most
    /// advanced SIMD instruction set available. It checks in order:
    /// 1. AVX-512 (if available and n_variables <= 64)
    /// 2. AVX2 (if available and n_variables <= 64)
    /// 3. X64 scalar fallback (always available)
    ///
    /// # Arguments
    /// * `n_variables` - The number of variables in the boolean function
    ///
    /// # Returns
    /// The best `OptimizedFor` variant for the current hardware
    ///
    /// # Examples
    /// ```
    /// use qm_agent::cnf_dnf::OptimizedFor;
    ///
    /// let optimization = OptimizedFor::detect_best(32);
    /// println!("Using optimization: {:?}", optimization);
    /// ```
    pub fn detect_best(n_variables: usize) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            // Check for AVX-512 support
            if std::is_x86_feature_detected!("avx512f")
                && std::is_x86_feature_detected!("avx512bw")
            {
                // Choose AVX-512 variant based on number of variables
                if n_variables <= 8 {
                    return Self::Avx512_8bits;
                } else if n_variables <= 16 {
                    return Self::Avx512_16bits;
                } else if n_variables <= 32 {
                    return Self::Avx512_32bits;
                } else {
                    return Self::Avx512_64bits;
                }
            }

            // Check for AVX2 support
            if std::is_x86_feature_detected!("avx2") && n_variables <= 64 {
                return Self::Avx2_64bits;
            }
        }

        // Fallback to scalar X64 (always available)
        Self::X64
    }

    /// Check if this optimization level is supported on the current hardware
    ///
    /// Returns `true` if the CPU has the required instruction set for this optimization level.
    ///
    /// # Examples
    /// ```
    /// use qm_agent::cnf_dnf::OptimizedFor;
    ///
    /// if OptimizedFor::Avx512_64bits.is_supported() {
    ///     println!("AVX-512 is available!");
    /// } else {
    ///     println!("AVX-512 is not available, will use fallback");
    /// }
    /// ```
    pub fn is_supported(&self) -> bool {
        match self {
            // AutoDetect and X64 are always supported (X64 is the fallback)
            Self::AutoDetect | Self::X64 => true,

            #[cfg(target_arch = "x86_64")]
            Self::Avx512_8bits | Self::Avx512_16bits | Self::Avx512_32bits | Self::Avx512_64bits => {
                std::is_x86_feature_detected!("avx512f") && std::is_x86_feature_detected!("avx512bw")
            }

            #[cfg(target_arch = "x86_64")]
            Self::Avx2_64bits => std::is_x86_feature_detected!("avx2"),

            // On non-x86_64 platforms, only X64 and AutoDetect are supported
            #[cfg(not(target_arch = "x86_64"))]
            Self::Avx512_8bits | Self::Avx512_16bits | Self::Avx512_32bits | Self::Avx512_64bits | Self::Avx2_64bits => false,
        }
    }

    /// Returns a human-readable string representation of the optimization level
    ///
    /// # Examples
    /// ```
    /// use qm_agent::cnf_dnf::OptimizedFor;
    ///
    /// let opt = OptimizedFor::Avx512_16bits;
    /// assert_eq!(opt.to_string(), "AVX-512 (16-bit)");
    /// ```
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::AutoDetect => "Auto-detect",
            Self::X64 => "X64 (scalar)",
            Self::Avx512_64bits => "AVX-512 (64-bit)",
            Self::Avx512_32bits => "AVX-512 (32-bit)",
            Self::Avx512_16bits => "AVX-512 (16-bit)",
            Self::Avx512_8bits => "AVX-512 (8-bit)",
            Self::Avx2_64bits => "AVX2 (64-bit)",
        }
    }
}

impl std::fmt::Display for OptimizedFor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
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

/// Private implementation of CNF to DNF conversion
fn convert_cnf_to_dnf_impl<T: Into<u64> + Copy>(
    cnf: &[T],
    n_bits: usize,
    of: OptimizedFor,
) -> Vec<u64> {
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
/// let dnf1 = cnf_dnf::convert_cnf_to_dnf::<Enc64, {OptimizedFor::AutoDetect}>(&cnf, 4);
///
/// // Force X64 scalar for testing
/// let dnf2 = cnf_dnf::convert_cnf_to_dnf::<Enc64, {OptimizedFor::X64}>(&cnf, 4);
///
/// // Force AVX512 for testing
/// let dnf3 = cnf_dnf::convert_cnf_to_dnf::<Enc64, {OptimizedFor::Avx512_64bits}>(&cnf, 4);
/// ```
pub fn convert_cnf_to_dnf<E: MintermEncoding, const OPT: OptimizedFor>(
    cnf: &[u64],
    n_bits: usize,
) -> Vec<u64> {
    // Validate that encoding supports this many bits
    if n_bits > E::MAX_VARS {
        eprintln!("ERROR: n_bits ({n_bits}) exceeds encoding maximum ({})", E::MAX_VARS);
        return Vec::new();
    }

    // Validate that optimization level is compatible with n_bits
    if OPT != OptimizedFor::AutoDetect && n_bits > OPT.max_bits() {
        eprintln!(
            "ERROR: n_bits ({}) exceeds OptimizedFor::{:?} maximum ({} bits)",
            n_bits,
            OPT,
            OPT.max_bits()
        );
        return Vec::new();
    }

    // Get optimization level from const parameter
    let of = match OPT {
        OptimizedFor::AutoDetect => OptimizedFor::detect_best(n_bits),
        other => other,
    };
    convert_cnf_to_dnf_impl(cnf, n_bits, of)
}

/// Convert CNF to minimal DNF with encoding-aware optimization selection
///
/// Returns only the smallest disjunctions from the DNF conversion.
/// Specify the optimization level as a type parameter, or use `AutoDetect`
/// for automatic hardware detection.
///
/// # Type Parameters
/// * `E` - Encoding type (Encoding16, Encoding32, Encoding64)
/// * `O` - Optimization level (X64Opt, Avx2_64bitsOpt, Avx512_64bitsOpt, etc., or AutoDetect)
///
/// # Arguments
/// * `cnf` - CNF formula as vector of disjunctions (bit vectors)
/// * `n_bits` - Number of Boolean variables
/// * `early_prune` - Whether to use early pruning optimization
///
/// # Examples
/// ```
/// use qm_agent::cnf_dnf::{self, OptimizedFor};
/// use qm_agent::qm::Enc64;
///
/// let cnf = vec![0b1010u64, 0b1100u64];
///
/// // Auto-detect optimization (recommended)
/// let dnf1 = cnf_dnf::convert_cnf_to_dnf_minimal::<Enc64, {OptimizedFor::AutoDetect}>(&cnf, 4, true);
///
/// // Force X64 scalar for testing
/// let dnf2 = cnf_dnf::convert_cnf_to_dnf_minimal::<Enc64, {OptimizedFor::X64}>(&cnf, 4, true);
///
/// // Force AVX512 for testing
/// let dnf3 = cnf_dnf::convert_cnf_to_dnf_minimal::<Enc64, {OptimizedFor::Avx512_64bits}>(&cnf, 4, true);
/// ```
pub fn convert_cnf_to_dnf_minimal<E: MintermEncoding, const OPT: OptimizedFor>(
    cnf: &[u64],
    n_bits: usize,
    early_prune: bool,
) -> Vec<u64> {
    // Validate that encoding supports this many bits
    if n_bits > E::MAX_VARS {
        eprintln!(
            "ERROR: n_bits ({}) exceeds encoding maximum ({})",
            n_bits,
            E::MAX_VARS
        );
        return Vec::new();
    }

    // Validate that optimization level is compatible with n_bits
    if OPT != OptimizedFor::AutoDetect && n_bits > OPT.max_bits() {
        eprintln!(
            "ERROR: n_bits ({}) exceeds OptimizedFor::{:?} maximum ({} bits)",
            n_bits,
            OPT,
            OPT.max_bits()
        );
        return Vec::new();
    }

    // Get optimization level from const parameter
    let of = match OPT {
        OptimizedFor::AutoDetect => OptimizedFor::detect_best(n_bits),
        other => other,
    };

    // Compute DNF with or without early pruning
    let result_dnf = if early_prune {
        convert_cnf_to_dnf_minimal_private_method1(cnf, n_bits, of)
    } else {
        convert_cnf_to_dnf_impl(cnf, n_bits, of)
    };

    if result_dnf.is_empty() {
        return result_dnf;
    }

    // Select only the smallest DNFs
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
                    let x = 1u64 << pos; // NOTE: x only contains one single bit set

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
        eprintln!("ERROR: too many different variables; found {n_variables} variables");
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

    // Do the conversion using appropriate encoding
    let dnf = if n_variables <= 16 {
        convert_cnf_to_dnf::<crate::qm::Enc16, {OptimizedFor::AutoDetect}>(&cnf_translated, n_variables)
    } else if n_variables <= 32 {
        convert_cnf_to_dnf::<crate::qm::Enc32, {OptimizedFor::AutoDetect}>(&cnf_translated, n_variables)
    } else {
        convert_cnf_to_dnf::<crate::qm::Enc64, {OptimizedFor::AutoDetect}>(&cnf_translated, n_variables)
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

        let dnf = convert_cnf_to_dnf::<crate::qm::Enc16, {OptimizedFor::AutoDetect}>(&cnf, 8);

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

        let dnf = convert_cnf_to_dnf_with_names(&cnf);

        assert_eq!(dnf.len(), 4);
    }

    #[test]
    fn test_minimal_dnf() {
        let cnf: Vec<u64> = vec![
            (1 << 1) | (1 << 2),
            (1 << 3) | (1 << 4),
        ];

        let dnf = convert_cnf_to_dnf_minimal::<crate::qm::Enc16, {OptimizedFor::AutoDetect}>(&cnf, 8, false);

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

    #[test]
    fn test_explicit_optimization() {
        let cnf: Vec<u64> = vec![0b011, 0b101, 0b110];

        // Test auto-detection (AutoDetect)
        let dnf_auto = convert_cnf_to_dnf::<crate::qm::Enc64, {OptimizedFor::AutoDetect}>(&cnf, 3);

        // Test explicit X64
        let dnf_x64 = convert_cnf_to_dnf::<crate::qm::Enc64, {OptimizedFor::X64}>(
            &cnf,
            3,
        );

        // Test explicit AVX512
        let dnf_avx512 = convert_cnf_to_dnf::<crate::qm::Enc64, {OptimizedFor::Avx512_64bits}>(
            &cnf,
            3,
        );

        // All should produce identical results
        assert_eq!(dnf_auto, dnf_x64);
        assert_eq!(dnf_x64, dnf_avx512);
        assert_eq!(dnf_auto.len(), 3);
    }

    #[test]
    fn test_explicit_optimization_minimal() {
        let cnf: Vec<u64> = vec![0b1010, 0b1100, 0b0110];

        // Test auto-detection with early pruning
        let dnf_auto = convert_cnf_to_dnf_minimal::<crate::qm::Enc64, {OptimizedFor::AutoDetect}>(
            &cnf, 4, true,
        );

        // Test explicit X64 with early pruning
        let dnf_x64 = convert_cnf_to_dnf_minimal::<crate::qm::Enc64, {OptimizedFor::X64}>(
            &cnf,
            4,
            true,
        );

        // Test explicit AVX2
        let dnf_avx2 = convert_cnf_to_dnf_minimal::<crate::qm::Enc64, {OptimizedFor::Avx2_64bits}>(
            &cnf,
            4,
            true,
        );

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
