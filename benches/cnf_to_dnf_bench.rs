// Benchmark comparing different SIMD optimization levels for CNF to DNF conversion
//
// This benchmark compares:
// - X64 (scalar baseline)
// - AVX2 64-bit
// - AVX512 8-bit, 16-bit, 32-bit, 64-bit
//
// Tests are run with different problem sizes and patterns to show scaling behavior

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use qm_agent::cnf_dnf::{convert_cnf_to_dnf, convert_cnf_to_dnf_minimal, OptimizedFor};
use rand::{rngs::StdRng, Rng, SeedableRng};

/// Generate a random CNF formula for benchmarking
fn generate_random_cnf(
    n_variables: usize,
    n_conjunctions: usize,
    literals_per_conjunction: usize,
    seed: u64,
) -> Vec<u64> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut cnf = Vec::new();

    for _ in 0..n_conjunctions {
        let mut conjunction = 0u64;
        for _ in 0..literals_per_conjunction {
            let var = rng.gen_range(0..n_variables);
            conjunction |= 1u64 << var;
        }
        cnf.push(conjunction);
    }

    cnf
}

/// Benchmark CNF to DNF conversion with different optimization levels
fn bench_optimization_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_levels");

    // Small problem: 8 variables, 5 conjunctions, 3 literals each
    let cnf_small = generate_random_cnf(8, 5, 3, 42);
    group.throughput(Throughput::Elements(cnf_small.len() as u64));

    let optimization_levels = vec![
        ("X64", OptimizedFor::X64),
        ("AVX2_64bits", OptimizedFor::Avx2_64bits),
        ("AVX512_8bits", OptimizedFor::Avx512_8bits),
        ("AVX512_16bits", OptimizedFor::Avx512_16bits),
        ("AVX512_32bits", OptimizedFor::Avx512_32bits),
        ("AVX512_64bits", OptimizedFor::Avx512_64bits),
    ];

    for (name, opt_level) in optimization_levels {
        group.bench_with_input(BenchmarkId::new("small", name), &opt_level, |b, &opt| {
            b.iter(|| {
                convert_cnf_to_dnf(black_box(&cnf_small), black_box(8), black_box(opt))
            });
        });
    }

    group.finish();
}

/// Benchmark with different problem sizes
fn bench_problem_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("problem_sizes");

    let sizes = vec![
        ("8var_5conj", 8, 5, 3),
        ("16var_8conj", 16, 8, 4),
        ("32var_10conj", 32, 10, 5),
    ];

    for (name, n_vars, n_conj, literals) in sizes {
        let cnf = generate_random_cnf(n_vars, n_conj, literals, 42);
        group.throughput(Throughput::Elements(cnf.len() as u64));

        // Test X64 baseline
        group.bench_with_input(BenchmarkId::new("X64", name), &cnf, |b, cnf| {
            b.iter(|| {
                convert_cnf_to_dnf(black_box(cnf), black_box(n_vars), black_box(OptimizedFor::X64))
            });
        });

        // Test best AVX512 for this size
        let opt_level = if n_vars <= 8 {
            OptimizedFor::Avx512_8bits
        } else if n_vars <= 16 {
            OptimizedFor::Avx512_16bits
        } else if n_vars <= 32 {
            OptimizedFor::Avx512_32bits
        } else {
            OptimizedFor::Avx512_64bits
        };

        group.bench_with_input(BenchmarkId::new("AVX512_optimal", name), &cnf, |b, cnf| {
            b.iter(|| {
                convert_cnf_to_dnf(black_box(cnf), black_box(n_vars), black_box(opt_level))
            });
        });
    }

    group.finish();
}

/// Benchmark AVX512 variants with matching bit widths
fn bench_avx512_variants(c: &mut Criterion) {
    let mut group = c.benchmark_group("avx512_variants");

    // Test each AVX512 variant with appropriate problem size
    let test_cases = vec![
        ("8bit_8vars", 8, 6, 3, OptimizedFor::Avx512_8bits),
        ("16bit_16vars", 16, 6, 4, OptimizedFor::Avx512_16bits),
        ("32bit_32vars", 32, 6, 5, OptimizedFor::Avx512_32bits),
        ("64bit_64vars", 64, 6, 6, OptimizedFor::Avx512_64bits),
    ];

    for (name, n_vars, n_conj, literals, opt_level) in test_cases {
        let cnf = generate_random_cnf(n_vars, n_conj, literals, 42);
        group.throughput(Throughput::Elements(cnf.len() as u64));

        group.bench_with_input(BenchmarkId::from_parameter(name), &cnf, |b, cnf| {
            b.iter(|| {
                convert_cnf_to_dnf(black_box(cnf), black_box(n_vars), black_box(opt_level))
            });
        });
    }

    group.finish();
}

/// Benchmark minimal DNF with early pruning
fn bench_minimal_with_pruning(c: &mut Criterion) {
    let mut group = c.benchmark_group("minimal_dnf");

    let cnf = generate_random_cnf(16, 8, 4, 42);
    group.throughput(Throughput::Elements(cnf.len() as u64));

    // Without early pruning
    group.bench_function("without_pruning", |b| {
        b.iter(|| {
            convert_cnf_to_dnf_minimal(
                black_box(&cnf),
                black_box(16),
                black_box(OptimizedFor::Avx512_16bits),
                black_box(false),
            )
        });
    });

    // With early pruning
    group.bench_function("with_pruning", |b| {
        b.iter(|| {
            convert_cnf_to_dnf_minimal(
                black_box(&cnf),
                black_box(16),
                black_box(OptimizedFor::Avx512_16bits),
                black_box(true),
            )
        });
    });

    group.finish();
}

/// Benchmark X64 vs AVX2 vs AVX512 for 64-bit problems
fn bench_64bit_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("64bit_comparison");

    let cnf = generate_random_cnf(64, 8, 6, 42);
    group.throughput(Throughput::Elements(cnf.len() as u64));

    let opt_levels = vec![
        ("X64_scalar", OptimizedFor::X64),
        ("AVX2_64bits", OptimizedFor::Avx2_64bits),
        ("AVX512_64bits", OptimizedFor::Avx512_64bits),
    ];

    for (name, opt_level) in opt_levels {
        group.bench_with_input(BenchmarkId::from_parameter(name), &opt_level, |b, &opt| {
            b.iter(|| {
                convert_cnf_to_dnf(black_box(&cnf), black_box(64), black_box(opt))
            });
        });
    }

    group.finish();
}

/// Benchmark dense vs sparse conjunctions
fn bench_conjunction_density(c: &mut Criterion) {
    let mut group = c.benchmark_group("conjunction_density");

    let n_vars = 16;
    let n_conj = 8;

    // Sparse: 2 literals per conjunction
    let cnf_sparse = generate_random_cnf(n_vars, n_conj, 2, 42);
    group.throughput(Throughput::Elements(cnf_sparse.len() as u64));

    group.bench_function("sparse_2lit", |b| {
        b.iter(|| {
            convert_cnf_to_dnf(
                black_box(&cnf_sparse),
                black_box(n_vars),
                black_box(OptimizedFor::Avx512_16bits),
            )
        });
    });

    // Medium: 4 literals per conjunction
    let cnf_medium = generate_random_cnf(n_vars, n_conj, 4, 42);

    group.bench_function("medium_4lit", |b| {
        b.iter(|| {
            convert_cnf_to_dnf(
                black_box(&cnf_medium),
                black_box(n_vars),
                black_box(OptimizedFor::Avx512_16bits),
            )
        });
    });

    // Dense: 8 literals per conjunction
    let cnf_dense = generate_random_cnf(n_vars, n_conj, 8, 42);

    group.bench_function("dense_8lit", |b| {
        b.iter(|| {
            convert_cnf_to_dnf(
                black_box(&cnf_dense),
                black_box(n_vars),
                black_box(OptimizedFor::Avx512_16bits),
            )
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_optimization_levels,
    bench_problem_sizes,
    bench_avx512_variants,
    bench_minimal_with_pruning,
    bench_64bit_comparison,
    bench_conjunction_density,
);
criterion_main!(benches);
