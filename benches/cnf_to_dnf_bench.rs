// Benchmark comparing different encodings for CNF to DNF conversion
//
// This benchmark compares:
// - Encoding16 (auto-selects Avx512_16bits)
// - Encoding32 (auto-selects Avx512_32bits)
// - Encoding64 (auto-selects Avx512_64bits)
//
// Tests are run with different problem sizes and patterns to show scaling behavior

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use qm_agent::cnf_dnf::{self, OptimizedFor};
use qm_agent::qm::{Enc16, Enc32, Enc64};
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
            let var = rng.random_range(0..n_variables);
            conjunction |= 1u64 << var;
        }
        cnf.push(conjunction);
    }

    cnf
}

/// Benchmark CNF to DNF conversion with different encodings
fn bench_encoding_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding_types");

    // Small problem: 8 variables, 5 conjunctions, 3 literals each
    let cnf_small = generate_random_cnf(8, 5, 3, 42);
    group.throughput(Throughput::Elements(cnf_small.len() as u64));

    // Test Encoding16 (auto-selects Avx512_16bits)
    group.bench_function("Encoding16", |b| {
        b.iter(|| {
            cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::AutoDetect}>(black_box(&cnf_small), black_box(8))
        });
    });

    // Test Encoding32 (auto-selects Avx512_32bits)
    group.bench_function("Encoding32", |b| {
        b.iter(|| {
            cnf_dnf::convert_cnf_to_dnf::<Enc32, {OptimizedFor::AutoDetect}>(black_box(&cnf_small), black_box(8))
        });
    });

    // Test Encoding64 (auto-selects Avx512_64bits)
    group.bench_function("Encoding64", |b| {
        b.iter(|| {
            cnf_dnf::convert_cnf_to_dnf::<Enc64, {OptimizedFor::AutoDetect}>(black_box(&cnf_small), black_box(8))
        });
    });

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

        // Test with appropriate encoding for the problem size
        if n_vars <= 16 {
            group.bench_with_input(BenchmarkId::new("Encoding16", name), &cnf, |b, cnf| {
                b.iter(|| {
                    cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::AutoDetect}>(black_box(cnf), black_box(n_vars))
                });
            });
        }

        if n_vars <= 32 {
            group.bench_with_input(BenchmarkId::new("Encoding32", name), &cnf, |b, cnf| {
                b.iter(|| {
                    cnf_dnf::convert_cnf_to_dnf::<Enc32, {OptimizedFor::AutoDetect}>(black_box(cnf), black_box(n_vars))
                });
            });
        }

        group.bench_with_input(BenchmarkId::new("Encoding64", name), &cnf, |b, cnf| {
            b.iter(|| {
                cnf_dnf::convert_cnf_to_dnf::<Enc64, {OptimizedFor::AutoDetect}>(black_box(cnf), black_box(n_vars))
            });
        });
    }

    group.finish();
}

/// Benchmark different encodings with matching problem sizes
fn bench_encoding_variants(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding_variants");

    // Test each encoding with appropriate problem size
    // Encoding16: 16 variables
    let cnf_16 = generate_random_cnf(16, 6, 4, 42);
    group.throughput(Throughput::Elements(cnf_16.len() as u64));
    group.bench_function("Encoding16_16vars", |b| {
        b.iter(|| {
            cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::AutoDetect}>(black_box(&cnf_16), black_box(16))
        });
    });

    // Encoding32: 32 variables
    let cnf_32 = generate_random_cnf(32, 6, 5, 42);
    group.throughput(Throughput::Elements(cnf_32.len() as u64));
    group.bench_function("Encoding32_32vars", |b| {
        b.iter(|| {
            cnf_dnf::convert_cnf_to_dnf::<Enc32, {OptimizedFor::AutoDetect}>(black_box(&cnf_32), black_box(32))
        });
    });

    // Encoding64: 64 variables
    let cnf_64 = generate_random_cnf(64, 6, 6, 42);
    group.throughput(Throughput::Elements(cnf_64.len() as u64));
    group.bench_function("Encoding64_64vars", |b| {
        b.iter(|| {
            cnf_dnf::convert_cnf_to_dnf::<Enc64, {OptimizedFor::AutoDetect}>(black_box(&cnf_64), black_box(64))
        });
    });

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
            cnf_dnf::convert_cnf_to_dnf_minimal::<Enc16, {OptimizedFor::AutoDetect}>(
                black_box(&cnf),
                black_box(16),
                black_box(false),
            )
        });
    });

    // With early pruning
    group.bench_function("with_pruning", |b| {
        b.iter(|| {
            cnf_dnf::convert_cnf_to_dnf_minimal::<Enc16, {OptimizedFor::AutoDetect}>(
                black_box(&cnf),
                black_box(16),
                black_box(true),
            )
        });
    });

    group.finish();
}

/// Benchmark encoding comparison for 64-bit problems
fn bench_64bit_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("64bit_comparison");

    let cnf = generate_random_cnf(64, 8, 6, 42);
    group.throughput(Throughput::Elements(cnf.len() as u64));

    // All encodings support 64 variables, compare them
    group.bench_function("Encoding64", |b| {
        b.iter(|| {
            cnf_dnf::convert_cnf_to_dnf::<Enc64, {OptimizedFor::AutoDetect}>(black_box(&cnf), black_box(64))
        });
    });

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
            cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::AutoDetect}>(
                black_box(&cnf_sparse),
                black_box(n_vars),
            )
        });
    });

    // Medium: 4 literals per conjunction
    let cnf_medium = generate_random_cnf(n_vars, n_conj, 4, 42);

    group.bench_function("medium_4lit", |b| {
        b.iter(|| {
            cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::AutoDetect}>(
                black_box(&cnf_medium),
                black_box(n_vars),
            )
        });
    });

    // Dense: 8 literals per conjunction
    let cnf_dense = generate_random_cnf(n_vars, n_conj, 8, 42);

    group.bench_function("dense_8lit", |b| {
        b.iter(|| {
            cnf_dnf::convert_cnf_to_dnf::<Enc16, {OptimizedFor::AutoDetect}>(
                black_box(&cnf_dense),
                black_box(n_vars),
            )
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_encoding_types,
    bench_problem_sizes,
    bench_encoding_variants,
    bench_minimal_with_pruning,
    bench_64bit_comparison,
    bench_conjunction_density,
);
criterion_main!(benches);
