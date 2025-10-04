use std::fmt;

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

    /// Resolve AutoDetect to a concrete optimization level
    ///
    /// If this is `AutoDetect`, performs hardware detection to select the best
    /// optimization level. Otherwise, returns self unchanged.
    ///
    /// # Arguments
    /// * `n_variables` - The number of variables in the boolean function
    ///
    /// # Returns
    /// A concrete `OptimizedFor` variant (never `AutoDetect`)
    ///
    /// # Examples
    /// ```
    /// use qm_agent::cnf_dnf::OptimizedFor;
    ///
    /// let opt = OptimizedFor::AutoDetect.resolve(32);
    /// assert_ne!(opt, OptimizedFor::AutoDetect); // Always returns concrete variant
    ///
    /// let opt2 = OptimizedFor::X64.resolve(32);
    /// assert_eq!(opt2, OptimizedFor::X64); // Non-AutoDetect unchanged
    /// ```
    pub fn resolve(self, n_variables: usize) -> Self {
        match self {
            Self::AutoDetect => Self::detect_best(n_variables),
            other => other,
        }
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
    /// This is a `const fn` that returns a `&'static str` without allocating.
    /// For converting to an owned `String`, use the `Display` trait via `.to_string()`.
    ///
    /// # Examples
    /// ```
    /// use qm_agent::cnf_dnf::OptimizedFor;
    ///
    /// let opt = OptimizedFor::Avx512_16bits;
    /// assert_eq!(opt.as_str(), "AVX-512 (16-bit)");
    /// assert_eq!(opt.to_string(), "AVX-512 (16-bit)"); // Via Display trait
    /// ```
    pub const fn as_str(&self) -> &'static str {
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

impl fmt::Display for OptimizedFor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
