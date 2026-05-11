use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use qm_agent::qm::encoding::{Enc32, Enc64};
use qm_agent::qm::primes::{TruthTable, find_prime_implicants as mc_find_pis};
use std::hint::black_box;

fn gen_and(n: u8) -> Vec<u64> {
    vec![(1u64 << n) - 1]
}

fn gen_or(n: u8) -> Vec<u64> {
    let all = (1u64 << n) - 1;
    (0..all).collect()
}

fn gen_xor(n: u8) -> Vec<u64> {
    (0..(1u64 << n))
        .filter(|&_i| _i.count_ones() & 1 == 1)
        .collect()
}

fn gen_maj(n: u8) -> Vec<u64> {
    let half = n as u32 / 2 + 1;
    (0..(1u64 << n))
        .filter(|&_i| _i.count_ones() >= half)
        .collect()
}

fn gen_random_density(n: u8, density: f64, seed: u64) -> Vec<u64> {
    let limit = 1u64 << n;
    let mut rng = seed;
    (0..limit)
        .filter(|&_i| {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (rng as f64 / u64::MAX as f64) < density
        })
        .collect()
}

fn bench_qm_pi(c: &mut Criterion) {
    let mut group = c.benchmark_group("pi_gen_qm");

    // Enc32 uses u64 as Value type
    let cases = [
        ("and3", gen_and(3), 3usize),
        ("or3", gen_or(3), 3usize),
        ("xor4", gen_xor(4), 4usize),
        ("maj5", gen_maj(5), 5usize),
        ("and6", gen_and(6), 6usize),
        ("and7", gen_and(7), 7usize),
        ("and8", gen_and(8), 8usize),
    ];

    for (name, minterms, n_vars) in cases {
        let n = n_vars;
        group.throughput(Throughput::Elements(minterms.len() as u64));
        group.bench_with_input(BenchmarkId::new("qm", name), &minterms, |b, mts| {
            let n = n_vars;
            let mts_vec: Vec<u64> = mts.to_vec();
            b.iter(|| {
                let mut solver = qm_agent::qm::quine_mccluskey::QuineMcCluskey::<Enc32>::new(n);
                solver.set_minterms(mts_vec.clone());
                let pis = solver.find_prime_implicants();
                black_box(pis.len())
            })
        });
    }

    group.finish();
}

fn bench_mc_pi(c: &mut Criterion) {
    let mut group = c.benchmark_group("pi_gen_min_cubes");

    let cases = [
        ("and3", gen_and(3), 3usize),
        ("or3", gen_or(3), 3usize),
        ("xor4", gen_xor(4), 4usize),
        ("maj5", gen_maj(5), 5usize),
        ("and6", gen_and(6), 6usize),
        ("and7", gen_and(7), 7usize),
        ("and8", gen_and(8), 8usize),
    ];

    for (name, minterms, n_vars) in cases {
        let n = n_vars;
        group.throughput(Throughput::Elements(minterms.len() as u64));
        group.bench_with_input(BenchmarkId::new("min_cubes", name), &minterms, |b, mts| {
            let tt = TruthTable::from_minterms(n, mts, &[]).expect("valid truth table");
            b.iter(|| {
                let pis = mc_find_pis(black_box(&tt), black_box(n));
                black_box(pis.len())
            })
        });
    }

    group.finish();
}

fn bench_comparison(c: &mut Criterion) {
    for (name, minterms, n_vars) in [
        ("and3", gen_and(3), 3usize),
        ("xor4", gen_xor(4), 4usize),
        ("maj5", gen_maj(5), 5usize),
        ("and6", gen_and(6), 6usize),
        ("and8", gen_and(8), 8usize),
    ] {
        let n = n_vars;

        // QM
        c.bench_with_input(
            BenchmarkId::new("compare", format!("{name}/qm")),
            &minterms,
            |b, _mts| {
                b.iter(|| {
                    let mut qm = qm_agent::qm::quine_mccluskey::QuineMcCluskey::<Enc64>::new(n);
                    let pis = qm.find_prime_implicants();
                    black_box(pis.len())
                })
            },
        );

        // Min-cubes
        c.bench_with_input(
            BenchmarkId::new("compare", format!("{name}/min_cubes")),
            &minterms,
            |b, mts| {
                let tt = TruthTable::from_minterms(n, mts, &[]).expect("valid truth table");
                b.iter(|| {
                    let pis = mc_find_pis(black_box(&tt), black_box(n));
                    black_box(pis.len())
                })
            },
        );
    }
}

criterion_group!(benches, bench_qm_pi, bench_mc_pi, bench_comparison,);
criterion_main!(benches);
