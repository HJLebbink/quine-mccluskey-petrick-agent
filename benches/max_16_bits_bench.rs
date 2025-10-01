// Benchmark to measure performance difference between MAX_16_BITS = true vs false
//
// To run these benchmarks:
// 1. Set MAX_16_BITS = false in src/qm/classic.rs (current default)
// 2. cargo bench --bench max_16_bits_bench > results_32bit.txt
// 3. Set MAX_16_BITS = true in src/qm/classic.rs
// 4. cargo bench --bench max_16_bits_bench > results_16bit.txt
// 5. Compare the results

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use qm_agent::qm::classic::{
    reduce_minterms, reduce_minterms_classic, minterms_to_string, minterm_to_string, MintermSet,
    Encoding16, Encoding32,
};

/// Generate minterms for a given number of variables
/// This creates a realistic problem with about 40% coverage
fn generate_minterms(n_variables: usize) -> Vec<u64> {
    let total = 1u64 << n_variables;
    let mut minterms = Vec::new();

    // Generate approximately 40% of possible minterms
    for i in 0..total {
        if (i * 7919) % 100 < 40 {  // Pseudo-random 40% selection
            minterms.push(i);
        }
    }

    minterms
}

/// Benchmark the core reduction algorithm - 32-bit mode
fn bench_reduce_minterms_32bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("reduce_minterms_32bit");

    for n_vars in [4, 8, 10, 12, 14, 16].iter() {
        let minterms: Vec<u64> = generate_minterms(*n_vars);
        let size = minterms.len();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("optimized_32bit", format!("{}_vars_{}_terms", n_vars, size)),
            &minterms,
            |b, minterms| {
                b.iter(|| {
                    reduce_minterms::<Encoding32>(black_box(minterms), false)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark the core reduction algorithm - 16-bit mode
fn bench_reduce_minterms_16bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("reduce_minterms_16bit");

    for n_vars in [4, 8, 10, 12, 14, 16].iter() {
        let minterms: Vec<u32> = generate_minterms(*n_vars).into_iter().map(|x| x as u32).collect();
        let size = minterms.len();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("optimized_16bit", format!("{}_vars_{}_terms", n_vars, size)),
            &minterms,
            |b, minterms| {
                b.iter(|| {
                    reduce_minterms::<Encoding16>(black_box(minterms), false)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark the classic O(n²) algorithm - 32-bit mode
fn bench_reduce_minterms_classic_32bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("reduce_minterms_classic_32bit");

    // Only test smaller sizes for classic algorithm (it's O(n²))
    for n_vars in [4, 6, 8, 10].iter() {
        let minterms: Vec<u64> = generate_minterms(*n_vars);
        let size = minterms.len();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("classic_32bit", format!("{}_vars_{}_terms", n_vars, size)),
            &minterms,
            |b, minterms| {
                b.iter(|| {
                    reduce_minterms_classic::<Encoding32>(black_box(minterms), *n_vars, false)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark the classic O(n²) algorithm - 16-bit mode
fn bench_reduce_minterms_classic_16bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("reduce_minterms_classic_16bit");

    // Only test smaller sizes for classic algorithm (it's O(n²))
    for n_vars in [4, 6, 8, 10].iter() {
        let minterms: Vec<u32> = generate_minterms(*n_vars).into_iter().map(|x| x as u32).collect();
        let size = minterms.len();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("classic_16bit", format!("{}_vars_{}_terms", n_vars, size)),
            &minterms,
            |b, minterms| {
                b.iter(|| {
                    reduce_minterms_classic::<Encoding16>(black_box(minterms), *n_vars, false)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark minterm to string conversion - 32-bit mode
fn bench_minterm_to_string_32bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("minterm_to_string_32bit");

    for n_vars in [4, 8, 12, 16, 20, 24, 28, 32].iter() {
        let minterm: u64 = 0b10101010_10101010_10101010_10101010u64;

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_vars", n_vars)),
            n_vars,
            |b, &n_vars| {
                b.iter(|| {
                    minterm_to_string::<Encoding32>(black_box(n_vars), black_box(minterm))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark minterm to string conversion - 16-bit mode
fn bench_minterm_to_string_16bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("minterm_to_string_16bit");

    for n_vars in [4, 8, 12, 16].iter() {
        let minterm: u32 = 0b10101010_10101010u32;

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_vars", n_vars)),
            n_vars,
            |b, &n_vars| {
                b.iter(|| {
                    minterm_to_string::<Encoding16>(black_box(n_vars), black_box(minterm))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark minterms to string - 32-bit mode
fn bench_minterms_to_string_32bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("minterms_to_string_32bit");

    for n_vars in [4, 8, 12, 16].iter() {
        let minterms: Vec<u64> = generate_minterms(*n_vars);
        let size = minterms.len();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_32bit", format!("{}_vars_{}_terms", n_vars, size)),
            &minterms,
            |b, minterms| {
                b.iter(|| {
                    minterms_to_string::<Encoding32>(black_box(*n_vars), black_box(minterms))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark minterms to string - 16-bit mode
fn bench_minterms_to_string_16bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("minterms_to_string_16bit");

    for n_vars in [4, 8, 12, 16].iter() {
        let minterms: Vec<u32> = generate_minterms(*n_vars).into_iter().map(|x| x as u32).collect();
        let size = minterms.len();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_16bit", format!("{}_vars_{}_terms", n_vars, size)),
            &minterms,
            |b, minterms| {
                b.iter(|| {
                    minterms_to_string::<Encoding16>(black_box(*n_vars), black_box(minterms))
                })
            },
        );
    }

    group.finish();
}

/// Benchmark MintermSet operations - 32-bit mode
fn bench_minterm_set_32bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("minterm_set_32bit");

    for n_vars in [4, 8, 12, 16].iter() {
        let minterms: Vec<u64> = generate_minterms(*n_vars);
        let size = minterms.len();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("add_all_32bit", format!("{}_vars_{}_terms", n_vars, size)),
            &minterms,
            |b, minterms| {
                b.iter(|| {
                    let mut set = MintermSet::<Encoding32>::new();
                    set.add_all(black_box(minterms));
                    set
                })
            },
        );
    }

    group.finish();
}

/// Benchmark MintermSet operations - 16-bit mode
fn bench_minterm_set_16bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("minterm_set_16bit");

    for n_vars in [4, 8, 12, 16].iter() {
        let minterms: Vec<u32> = generate_minterms(*n_vars).into_iter().map(|x| x as u32).collect();
        let size = minterms.len();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("add_all_16bit", format!("{}_vars_{}_terms", n_vars, size)),
            &minterms,
            |b, minterms| {
                b.iter(|| {
                    let mut set = MintermSet::<Encoding16>::new();
                    set.add_all(black_box(minterms));
                    set
                })
            },
        );
    }

    group.finish();
}

/// Benchmark MintermSet retrieval - 32-bit mode
fn bench_minterm_set_get_32bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("minterm_set_get_32bit");

    for n_vars in [4, 8, 12, 16].iter() {
        let minterms: Vec<u64> = generate_minterms(*n_vars);
        let mut set = MintermSet::<Encoding32>::new();
        set.add_all(&minterms);
        let max_bit_count = set.get_max_bit_count();

        group.bench_with_input(
            BenchmarkId::new("iterate_32bit", format!("{}_vars", n_vars)),
            &(set, max_bit_count),
            |b, (set, max_bit_count)| {
                b.iter(|| {
                    let mut total = 0;
                    for bit_count in 0..=*max_bit_count {
                        total += set.get(black_box(bit_count)).len();
                    }
                    total
                })
            },
        );
    }

    group.finish();
}

/// Benchmark MintermSet retrieval - 16-bit mode
fn bench_minterm_set_get_16bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("minterm_set_get_16bit");

    for n_vars in [4, 8, 12, 16].iter() {
        let minterms: Vec<u32> = generate_minterms(*n_vars).into_iter().map(|x| x as u32).collect();
        let mut set = MintermSet::<Encoding16>::new();
        set.add_all(&minterms);
        let max_bit_count = set.get_max_bit_count();

        group.bench_with_input(
            BenchmarkId::new("iterate_16bit", format!("{}_vars", n_vars)),
            &(set, max_bit_count),
            |b, (set, max_bit_count)| {
                b.iter(|| {
                    let mut total = 0;
                    for bit_count in 0..=*max_bit_count {
                        total += set.get(black_box(bit_count)).len();
                    }
                    total
                })
            },
        );
    }

    group.finish();
}

/// Full end-to-end benchmark - 32-bit mode
fn bench_full_reduction_32bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_reduction_32bit");
    group.sample_size(20); // Fewer samples for longer benchmarks

    for n_vars in [4, 6, 8, 10, 12].iter() {
        let minterms: Vec<u64> = generate_minterms(*n_vars);
        let size = minterms.len();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("until_fixed_point_32bit", format!("{}_vars_{}_terms", n_vars, size)),
            &minterms,
            |b, minterms| {
                b.iter(|| {
                    let mut current = minterms.clone();
                    let mut iteration = 0;
                    loop {
                        let next = reduce_minterms::<Encoding32>(black_box(&current), false);
                        iteration += 1;
                        if current == next || iteration > 100 {
                            break;
                        }
                        current = next;
                    }
                    current
                })
            },
        );
    }

    group.finish();
}

/// Full end-to-end benchmark - 16-bit mode
fn bench_full_reduction_16bit(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_reduction_16bit");
    group.sample_size(20); // Fewer samples for longer benchmarks

    for n_vars in [4, 6, 8, 10, 12].iter() {
        let minterms: Vec<u32> = generate_minterms(*n_vars).into_iter().map(|x| x as u32).collect();
        let size = minterms.len();

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("until_fixed_point_16bit", format!("{}_vars_{}_terms", n_vars, size)),
            &minterms,
            |b, minterms| {
                b.iter(|| {
                    let mut current = minterms.clone();
                    let mut iteration = 0;
                    loop {
                        let next = reduce_minterms::<Encoding16>(black_box(&current), false);
                        iteration += 1;
                        if current == next || iteration > 100 {
                            break;
                        }
                        current = next;
                    }
                    current
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_reduce_minterms_32bit,
    bench_reduce_minterms_16bit,
    bench_reduce_minterms_classic_32bit,
    bench_reduce_minterms_classic_16bit,
    bench_minterm_to_string_32bit,
    bench_minterm_to_string_16bit,
    bench_minterms_to_string_32bit,
    bench_minterms_to_string_16bit,
    bench_minterm_set_32bit,
    bench_minterm_set_16bit,
    bench_minterm_set_get_32bit,
    bench_minterm_set_get_16bit,
    bench_full_reduction_32bit,
    bench_full_reduction_16bit,
);
criterion_main!(benches);
