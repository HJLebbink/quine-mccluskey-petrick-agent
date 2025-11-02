// Benchmark comparing gray code pair finding implementations
//
// Compares:
// 1. Scalar: O(n*m) nested loops with XOR + popcount
// 2. Level1 Optimized: HashMap lookup to skip impossible matches
// 3. AVX512: SIMD vectorization processing 16 u32s at a time
//
// To run these benchmarks:
// cargo bench --bench gray_code_bench
//
// To run specific benchmarks:
// cargo bench --bench gray_code_bench -- scalar_u32
// cargo bench --bench gray_code_bench -- level1_u32
// cargo bench --bench gray_code_bench -- avx512_u32

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use qm_agent::qm::gray_code::{
    find_gray_code_pairs_scalar_u32, find_gray_code_pairs_ref_u32,
    find_gray_code_pairs_avx512_u32,
};
use std::hint::black_box;

/// Generate test data: two groups of indices and their encodings
/// This simulates a realistic QM scenario where we're checking gray code pairs
/// between consecutive bit-count groups
fn generate_test_data(group1_size: usize, group2_size: usize) -> (Vec<usize>, Vec<usize>, Vec<u32>) {
    let total_size = group1_size + group2_size;
    let mut raw_encodings = Vec::with_capacity(total_size);

    // Generate encodings with varying bit patterns
    // Group1 will have fewer 1-bits than group2 (simulating k vs k+1 bit counts)
    for i in 0..group1_size {
        // Encodings with k bits set
        let encoding = generate_k_bit_pattern(i, 8); // 8 bits set
        raw_encodings.push(encoding);
    }

    for i in 0..group2_size {
        // Encodings with k+1 bits set (gray code pairs differ by 1 bit)
        let encoding = generate_k_bit_pattern(i, 9); // 9 bits set
        raw_encodings.push(encoding);
    }

    let group1_indices: Vec<usize> = (0..group1_size).collect();
    let group2_indices: Vec<usize> = (group1_size..total_size).collect();

    (group1_indices, group2_indices, raw_encodings)
}

/// Generate a pattern with approximately k bits set
/// Uses pseudo-random distribution based on index
fn generate_k_bit_pattern(index: usize, target_bits: u32) -> u32 {
    let mut pattern = 0u32;
    let mut bits_set = 0;
    let mut seed = (index * 7919 + 2531) as u32; // Pseudo-random seed

    while bits_set < target_bits {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let bit_pos = (seed % 32) as u32;
        if (pattern & (1 << bit_pos)) == 0 {
            pattern |= 1 << bit_pos;
            bits_set += 1;
        }
    }

    pattern
}

/// Benchmark scalar implementation
fn bench_scalar_u32(c: &mut Criterion) {
    let mut group = c.benchmark_group("gray_code_scalar_u32");

    // Test different problem sizes
    let sizes = [
        (50, 50),
        (100, 100),
        (200, 200),
        (500, 500),
        (1000, 1000),
    ];

    for (g1_size, g2_size) in sizes.iter() {
        let (group1_indices, group2_indices, raw_encodings) = generate_test_data(*g1_size, *g2_size);
        let total_checks = (g1_size * g2_size) as u64;

        group.throughput(Throughput::Elements(total_checks));
        group.bench_with_input(
            BenchmarkId::new("scalar", format!("{}x{}", g1_size, g2_size)),
            &(group1_indices, group2_indices, raw_encodings),
            |b, (g1, g2, encodings)| {
                b.iter(|| {
                    find_gray_code_pairs_scalar_u32(
                        black_box(g1),
                        black_box(g2),
                        black_box(encodings),
                    )
                })
            },
        );
    }

    group.finish();
}

/// Benchmark level1 optimized implementation (HashMap lookup)
fn bench_level1_u32(c: &mut Criterion) {
    let mut group = c.benchmark_group("gray_code_level1_u32");

    // Test different problem sizes
    let sizes = [
        (50, 50),
        (100, 100),
        (200, 200),
        (500, 500),
        (1000, 1000),
    ];

    for (g1_size, g2_size) in sizes.iter() {
        let (group1_indices, group2_indices, raw_encodings) = generate_test_data(*g1_size, *g2_size);
        let total_checks = (g1_size * g2_size) as u64;

        group.throughput(Throughput::Elements(total_checks));
        group.bench_with_input(
            BenchmarkId::new("level1", format!("{}x{}", g1_size, g2_size)),
            &(group1_indices, group2_indices, raw_encodings),
            |b, (g1, g2, encodings)| {
                b.iter(|| {
                    find_gray_code_pairs_ref_u32(
                        black_box(g1),
                        black_box(g2),
                        black_box(encodings),
                    )
                })
            },
        );
    }

    group.finish();
}

/// Benchmark AVX512 implementation
fn bench_avx512_u32(c: &mut Criterion) {
    let mut group = c.benchmark_group("gray_code_avx512_u32");

    // Test different problem sizes
    let sizes = [
        (50, 50),
        (100, 100),
        (200, 200),
        (500, 500),
        (1000, 1000),
        (2000, 2000), // Larger sizes where SIMD really shines
    ];

    for (g1_size, g2_size) in sizes.iter() {
        let (group1_indices, group2_indices, raw_encodings) = generate_test_data(*g1_size, *g2_size);
        let total_checks = (g1_size * g2_size) as u64;

        group.throughput(Throughput::Elements(total_checks));
        group.bench_with_input(
            BenchmarkId::new("avx512", format!("{}x{}", g1_size, g2_size)),
            &(group1_indices, group2_indices, raw_encodings),
            |b, (g1, g2, encodings)| {
                b.iter(|| {
                    find_gray_code_pairs_avx512_u32(
                        black_box(g1),
                        black_box(g2),
                        black_box(encodings),
                    )
                })
            },
        );
    }

    group.finish();
}

/// Benchmark comparing all three implementations side by side
fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("gray_code_comparison");

    // Use a medium-sized problem for direct comparison
    let sizes = [(100, 100), (500, 500), (1000, 1000)];

    for (g1_size, g2_size) in sizes.iter() {
        let (group1_indices, group2_indices, raw_encodings) = generate_test_data(*g1_size, *g2_size);
        let total_checks = (g1_size * g2_size) as u64;

        group.throughput(Throughput::Elements(total_checks));

        // Scalar
        group.bench_with_input(
            BenchmarkId::new("scalar", format!("{}x{}", g1_size, g2_size)),
            &(group1_indices.clone(), group2_indices.clone(), raw_encodings.clone()),
            |b, (g1, g2, encodings)| {
                b.iter(|| {
                    find_gray_code_pairs_scalar_u32(
                        black_box(g1),
                        black_box(g2),
                        black_box(encodings),
                    )
                })
            },
        );

        // Level1
        group.bench_with_input(
            BenchmarkId::new("level1", format!("{}x{}", g1_size, g2_size)),
            &(group1_indices.clone(), group2_indices.clone(), raw_encodings.clone()),
            |b, (g1, g2, encodings)| {
                b.iter(|| {
                    find_gray_code_pairs_ref_u32(
                        black_box(g1),
                        black_box(g2),
                        black_box(encodings),
                    )
                })
            },
        );

        // AVX512
        group.bench_with_input(
            BenchmarkId::new("avx512", format!("{}x{}", g1_size, g2_size)),
            &(group1_indices, group2_indices, raw_encodings),
            |b, (g1, g2, encodings)| {
                b.iter(|| {
                    find_gray_code_pairs_avx512_u32(
                        black_box(g1),
                        black_box(g2),
                        black_box(encodings),
                    )
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_scalar_u32,
    bench_level1_u32,
    bench_avx512_u32,
    bench_comparison,
);
criterion_main!(benches);
