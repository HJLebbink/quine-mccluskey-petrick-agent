use super::encoding::MintermEncoding;
use super::implicant::Implicant;
use super::simd_coverage;

pub struct PetricksMethod<E: MintermEncoding> {
    prime_implicants: Vec<Implicant<E>>,
    minterms: Vec<E::Value>,
}

impl<E: MintermEncoding> PetricksMethod<E> {
    /// Create a new Petrick's method solver.
    ///
    /// Takes the prime implicants found by the QM algorithm and the minterms
    /// that must be covered. The solver will select a minimal subset of
    /// prime implicants that covers all minterms.
    pub fn new(prime_implicants: &[Implicant<E>], minterms: &[E::Value]) -> Self {
        Self {
            prime_implicants: prime_implicants.to_vec(),
            minterms: minterms.to_vec(),
        }
    }

    /// Determine number of bits in the prime implicants
    fn get_num_bits(&self) -> usize {
        self.prime_implicants
            .first()
            .map(|pi| pi.n_variables)
            .unwrap_or(0)
    }

    /// Find a minimal cover of prime implicants that covers all minterms.
    ///
    /// Uses SIMD acceleration when available (AVX-512 with ≥1024 checks) and
    /// falls back to scalar greedy selection otherwise. The greedy approach
    /// iterates through prime implicants in order, selecting each one that
    /// covers at least one previously uncovered minterm.
    ///
    /// Returns an empty vector if no prime implicants are available.
    pub fn find_minimal_cover(&self) -> Vec<Implicant<E>> {
        if self.prime_implicants.is_empty() {
            return Vec::new();
        }

        let num_checks = self.prime_implicants.len() * self.minterms.len();
        let num_bits = self.get_num_bits();

        // Use SIMD if available and worthwhile
        if simd_coverage::should_use_simd(num_checks, num_bits) {
            #[cfg(all(target_arch = "x86_64", feature = "simd"))]
            {
                return unsafe { self.find_minimal_cover_simd() };
            }
        }

        // Fallback to scalar
        self.find_minimal_cover_scalar()
    }

    /// SIMD-accelerated minimal cover using pre-computed coverage matrix.
    ///
    /// Builds a bit-packed coverage matrix using AVX-512 4-bit or 5-bit
    /// operations, then performs greedy selection. Returns early once all
    /// minterms are covered.
    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    unsafe fn find_minimal_cover_simd(&self) -> Vec<Implicant<E>> {
        let num_bits = self.get_num_bits();

        // Build coverage matrix using SIMD (bit-packed)
        // Dispatch to appropriate bit-width implementation
        let coverage_matrix = unsafe {
            match num_bits {
                0..=4 => simd_coverage::build_coverage_matrix_simd_4bit(
                    &self.prime_implicants,
                    &self.minterms,
                ),
                5 => simd_coverage::build_coverage_matrix_simd_5bit(
                    &self.prime_implicants,
                    &self.minterms,
                ),
                _ => unreachable!("should_use_simd guards against >5 bits"),
            }
        };

        // Greedy selection using pre-computed matrix
        let mut covered_minterms = std::collections::HashSet::new();
        let mut selected = Vec::new();

        for (pi_idx, pi) in self.prime_implicants.iter().enumerate() {
            let mut covers_new = false;
            for (mt_idx, &minterm) in self.minterms.iter().enumerate() {
                if coverage_matrix.get(pi_idx, mt_idx) && !covered_minterms.contains(&minterm) {
                    covers_new = true;
                    break;
                }
            }

            if covers_new {
                selected.push(pi.clone());
                for &minterm in &pi.covered_minterms {
                    covered_minterms.insert(minterm);
                }
            }

            if covered_minterms.len() >= self.minterms.len() {
                break;
            }
        }

        selected
    }

    /// Original scalar implementation
    fn find_minimal_cover_scalar(&self) -> Vec<Implicant<E>> {
        let mut covered_minterms = std::collections::HashSet::new();
        let mut selected = Vec::new();

        for pi in &self.prime_implicants {
            let mut covers_new = false;
            for &minterm in &self.minterms {
                if pi.covers_minterm(minterm) && !covered_minterms.contains(&minterm) {
                    covers_new = true;
                    break;
                }
            }

            if covers_new {
                selected.push(pi.clone());
                for &minterm in &pi.covered_minterms {
                    covered_minterms.insert(minterm);
                }
            }

            if covered_minterms.len() >= self.minterms.len() {
                break;
            }
        }

        selected
    }

    /// Generate a product-of-sums expression from the prime implicant coverage.
    ///
    /// Currently returns a placeholder string. Full implementation would convert
    /// the minimal cover into POS form using the dual of Petrick's method.
    pub fn generate_product_of_sums(&self) -> String {
        "Dummy POS expression".to_string()
    }
}
