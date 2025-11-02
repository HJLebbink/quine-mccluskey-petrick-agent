//! Benchmark comparing scalar vs SIMD coverage matrix computation
//!
//! This benchmark measures the performance of prime implicant coverage checking
//! for the Quine-McCluskey algorithm, comparing:
//! - Scalar implementation: Checks each minterm-implicant pair individually
//! - SIMD implementation: Checks 512 pairs simultaneously using AVX-512

use qm_agent::qm::encoding::Enc16;
use qm_agent::qm::implicant::{BitState, Implicant};
use qm_agent::qm::CoverageMatrix;
use std::time::{Duration, Instant};

const NUM_PRIME_IMPLICANTS: usize = 100;
const NUM_MINTERMS: usize = 10000;
const WARMUP_ITERATIONS: usize = 10;
const BENCHMARK_ITERATIONS: usize = 100;

/// Check if an implicant covers a minterm based on bit pattern
fn implicant_covers_minterm(implicant: &Implicant<Enc16>, minterm: u32) -> bool {
    for (bit_idx, bit_state) in implicant.bits.iter().enumerate() {
        let minterm_bit = (minterm >> bit_idx) & 1;
        match bit_state {
            BitState::Zero if minterm_bit != 0 => return false,
            BitState::One if minterm_bit != 1 => return false,
            BitState::DontCare => continue,
            _ => continue,
        }
    }
    true
}

/// Build coverage matrix using scalar implementation (reference)
fn build_coverage_matrix_scalar(
    prime_implicants: &[Implicant<Enc16>],
    minterms: &[u32],
) -> CoverageMatrix {
    let num_pi = prime_implicants.len();
    let num_mt = minterms.len();
    let mut coverage_matrix = CoverageMatrix::new(num_pi, num_mt);

    for (pi_idx, pi) in prime_implicants.iter().enumerate() {
        for (mt_idx, &minterm) in minterms.iter().enumerate() {
            coverage_matrix.set(pi_idx, mt_idx, implicant_covers_minterm(pi, minterm));
        }
    }

    coverage_matrix
}

/// Build coverage matrix using SIMD implementation
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
fn build_coverage_matrix_simd(
    prime_implicants: &[Implicant<Enc16>],
    minterms: &[u32],
) -> CoverageMatrix {
    use qm_agent::qm::simd_coverage;

    unsafe { simd_coverage::build_coverage_matrix_simd_4bit(prime_implicants, minterms) }
}

/// Generate random prime implicants for testing
fn generate_test_prime_implicants(count: usize, num_bits: usize) -> Vec<Implicant<Enc16>> {
    use rand::Rng;
    let mut rng = rand::rng();
    let mut implicants = Vec::new();

    for _ in 0..count {
        let mut pi = Implicant::<Enc16>::from_minterm(0, num_bits);

        // Randomly set each bit to Zero, One, or DontCare
        for bit_idx in 0..num_bits {
            pi.bits[bit_idx] = match rng.random_range(0u8..3) {
                0 => BitState::Zero,
                1 => BitState::One,
                _ => BitState::DontCare,
            };
        }

        implicants.push(pi);
    }

    implicants
}

/// Generate random minterms for testing
fn generate_test_minterms(count: usize, max_value: u32) -> Vec<u32> {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..count).map(|_| rng.random_range(0..max_value)).collect()
}

fn main() {
    println!("=== QMC Coverage Matrix: Scalar vs SIMD Benchmark ===\n");

    // Check hardware support
    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    let simd_available = is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("gfni");

    #[cfg(not(all(target_arch = "x86_64", feature = "simd")))]
    let simd_available = false;

    if !simd_available {
        eprintln!("ERROR: AVX-512F or GFNI not available. SIMD benchmark will be skipped.");
        eprintln!("NOTE: Your CPU does not support the required AVX-512F and GFNI features.");
    }

    println!("Configuration:");
    println!("  Prime implicants: {}", NUM_PRIME_IMPLICANTS);
    println!("  Minterms: {}", NUM_MINTERMS);
    println!("  Total checks: {} million",
        (NUM_PRIME_IMPLICANTS * NUM_MINTERMS) / 1_000_000);
    println!("  Warmup iterations: {}", WARMUP_ITERATIONS);
    println!("  Benchmark iterations: {}", BENCHMARK_ITERATIONS);
    println!("  SIMD available: {}\n", if simd_available { "Yes" } else { "No" });

    // Generate test data
    println!("Generating test data...");
    let prime_implicants = generate_test_prime_implicants(NUM_PRIME_IMPLICANTS, 4);
    let minterms = generate_test_minterms(NUM_MINTERMS, 16);
    println!("  Generated {} prime implicants", prime_implicants.len());
    println!("  Generated {} minterms\n", minterms.len());

    // Verify correctness first
    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    if simd_available {
        println!("Verifying SIMD correctness...");
        let scalar_result = build_coverage_matrix_scalar(&prime_implicants, &minterms);
        let simd_result = build_coverage_matrix_simd(&prime_implicants, &minterms);

        let mut mismatch_count = 0;
        for pi_idx in 0..scalar_result.num_rows() {
            for mt_idx in 0..scalar_result.num_cols() {
                let scalar_val = scalar_result.get(pi_idx, mt_idx);
                let simd_val = simd_result.get(pi_idx, mt_idx);
                if scalar_val != simd_val {
                    if mismatch_count < 5 {
                        eprintln!(
                            "  Mismatch at PI[{}] MT[{}]: scalar={} simd={}",
                            pi_idx, mt_idx, scalar_val, simd_val
                        );
                    }
                    mismatch_count += 1;
                }
            }
        }

        if mismatch_count == 0 {
            println!("  ✓ SIMD results match scalar implementation\n");
        } else {
            eprintln!("  ✗ SIMD correctness check FAILED!");
            eprintln!("    {} mismatches found\n", mismatch_count);
            return;
        }
    }

    // Warmup
    println!("Warming up...");
    for _ in 0..WARMUP_ITERATIONS {
        let _result = build_coverage_matrix_scalar(&prime_implicants, &minterms);

        #[cfg(all(target_arch = "x86_64", feature = "simd"))]
        if simd_available {
            let _result = build_coverage_matrix_simd(&prime_implicants, &minterms);
        }
    }
    println!("  Warmup complete\n");

    // Benchmark scalar implementation
    println!("Benchmarking scalar implementation...");
    let mut scalar_times = Vec::new();
    for _ in 0..BENCHMARK_ITERATIONS {
        let start = Instant::now();
        let _result = build_coverage_matrix_scalar(&prime_implicants, &minterms);
        let duration = start.elapsed();
        scalar_times.push(duration);
    }

    let scalar_avg = scalar_times.iter().sum::<Duration>() / BENCHMARK_ITERATIONS as u32;
    let scalar_min = *scalar_times.iter().min().unwrap();
    let scalar_max = *scalar_times.iter().max().unwrap();
    let scalar_checks_per_sec =
        (NUM_PRIME_IMPLICANTS * NUM_MINTERMS) as f64 / scalar_avg.as_secs_f64();

    println!("  Average: {:?}", scalar_avg);
    println!("  Min: {:?}", scalar_min);
    println!("  Max: {:?}", scalar_max);
    println!("  Throughput: {:.2} million checks/sec\n", scalar_checks_per_sec / 1_000_000.0);

    // Benchmark SIMD implementation
    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    if simd_available {
        println!("Benchmarking SIMD implementation...");
        let mut simd_times = Vec::new();
        for _ in 0..BENCHMARK_ITERATIONS {
            let start = Instant::now();
            let _result = build_coverage_matrix_simd(&prime_implicants, &minterms);
            let duration = start.elapsed();
            simd_times.push(duration);
        }

        let simd_avg = simd_times.iter().sum::<Duration>() / BENCHMARK_ITERATIONS as u32;
        let simd_min = *simd_times.iter().min().unwrap();
        let simd_max = *simd_times.iter().max().unwrap();
        let simd_checks_per_sec =
            (NUM_PRIME_IMPLICANTS * NUM_MINTERMS) as f64 / simd_avg.as_secs_f64();

        println!("  Average: {:?}", simd_avg);
        println!("  Min: {:?}", simd_min);
        println!("  Max: {:?}", simd_max);
        println!("  Throughput: {:.2} million checks/sec\n", simd_checks_per_sec / 1_000_000.0);

        // Calculate speedup
        let speedup = scalar_avg.as_secs_f64() / simd_avg.as_secs_f64();
        let throughput_improvement = simd_checks_per_sec / scalar_checks_per_sec;

        println!("=== Results ===");
        println!("  Speedup: {:.2}×", speedup);
        println!("  Throughput improvement: {:.2}×", throughput_improvement);
        println!("  Time saved: {:?} per iteration", scalar_avg - simd_avg);

        // Efficiency analysis
        let theoretical_max_speedup = 512.0; // Process 512 values simultaneously
        let efficiency = (speedup / theoretical_max_speedup) * 100.0;
        println!("\n  Theoretical max speedup: {:.0}× (512 values in parallel)", theoretical_max_speedup);
        println!("  Efficiency: {:.1}%", efficiency);

        if speedup > 1.5 {
            println!("\n  ✓ SIMD provides significant speedup!");
        } else {
            println!("\n  ⚠ SIMD speedup is modest (overhead dominates for this problem size)");
        }
    }

    #[cfg(not(all(target_arch = "x86_64", feature = "simd")))]
    {
        println!("SIMD benchmark skipped (SIMD support not available on this platform)");
    }
}
