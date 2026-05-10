use criterion::{Criterion, criterion_group, criterion_main, BenchmarkId, Throughput};
use qm_agent::qm::encoding::{Enc64, MintermEncoding};
use qm_agent::qm::qm_solver::QMSolver;
use qm_agent::qm::min_cubes::{find_prime_implicants as mc_find_pis, TruthTable};
use std::hint::black_box;

fn gen_and(n: u8) -> Vec<u64> {
    vec![(1u64 << n) - 1]
}

fn gen_or(n: u8) -> Vec<u64> {
    let all = (1u64 << n) - 1;
    (0..all).collect()
}

fn gen_xor(n: u8) -> Vec<u64> {
    let mut out = Vec::new();
    for i in 0..(1u64 << n) {
        if i.count_ones() & 1 == 1 {
            out.push(i);
        }
    }
    out
}

fn gen_maj(n: u8) -> Vec<u64> {
    let half = n as u32 / 2 + 1;
    let mut out = Vec::new();
    for i in 0..(1u64 << n) {
        if i.count_ones() >= half {
            out.push(i);
        }
    }
    out
}

fn gen_random_density(n: u8, density: f64, seed: u64) -> Vec<u64> {
    let mut rng = seed;
    let limit = 1u64 << n;
    let mut out = Vec::new();
    for i in 0..limit {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        if (rng as f64 / u64::MAX as f64) < density {
            out.push(i);
        }
    }
    out
}

fn bench_qm_pi(c: &mut Criterion) {
    let mut group = c.benchmark_group("pi_gen_qm");

    let cases = [
        ("and3", gen_and(3), 3, 0),
        ("or3", gen_or(3), 3, 0),
        ("xor4", gen_xor(4), 4, 0),
        ("maj5", gen_maj(5), 5, 0),
        ("and6", gen_and(6), 6, 0),
        ("and7", gen_and(7), 7, 0),
        ("and8", gen_and(8), 8, 0),
        ("random10_0.5", gen_random_density(10, 0.5, 42), 10, 0),
        ("random10_0.1", gen_random_density(10, 0.1, 100), 10, 0),
        ("random16_0.5", gen_random_density(16, 0.5, 42), 16, 0),
    ];

    for (name, minterms, n_vars, dc) in cases {
        let n_vars = n_vars as usize;
        group.bench_with_input(
            BenchmarkId::new("qm", name),
            &(&minterms, n_vars, dc),
            |b, (mts, vars, dcs)| {
                let mts: Vec<u64> = mts.clone();
                let dcs: Vec<u64> = vec![*dcs as u64];
                let n = *vars;
                b.iter(|| {
                    let mut solver = QMSolver::<Enc64>::new(n);
                    solver.set_minterms(black_box(mts.clone()));
                    solver.set_dont_cares(black_box(dcs.clone()));
                    let qm = qm_agent::qm::quine_mccluskey::QuineMcCluskey::<Enc64>::new(n);
                    let pis = qm.find_prime_implicants();
                    black_box(pis.len())
                })
            },
        );
    }

    group.finish();
}

fn bench_mc_pi(c: &mut Criterion) {
    let mut group = c.benchmark_group("pi_gen_min_cubes");

    let cases = [
        ("and3", gen_and(3), 3, 0),
        ("or3", gen_or(3), 3, 0),
        ("xor4", gen_xor(4), 4, 0),
        ("maj5", gen_maj(5), 5, 0),
        ("and6", gen_and(6), 6, 0),
        ("and7", gen_and(7), 7, 0),
        ("and8", gen_and(8), 8, 0),
        ("random10_0.5", gen_random_density(10, 0.5, 42), 10, 0),
        ("random10_0.1", gen_random_density(10, 0.1, 100), 10, 0),
        ("random16_0.5", gen_random_density(16, 0.5, 42), 16, 0),
    ];

    for (name, minterms, n_vars, dc) in cases {
        let n_vars = n_vars as usize;
        group.bench_with_input(
            BenchmarkId::new("min_cubes", name),
            &(&minterms, n_vars, dc),
            |b, (mts, vars, dcs)| {
                let mts: Vec<u64> = mts.clone();
                let dcs: Vec<u64> = vec![*dcs as u64];
                let n = *vars;
                let tt = TruthTable::from_minterms(n, &mts, &dcs)
                    .expect("valid truth table");
                let pi_depth = n;
                b.iter(|| {
                    let pis = mc_find_pis(black_box(&tt), black_box(pi_depth));
                    black_box(pis.len())
                })
            },
        );
    }

    group.finish();
}

fn bench_comparison(c: &mut Criterion) {
    for (name, minterms, n_vars, dc) in [
        ("and3", gen_and(3), 3, 0),
        ("xor4", gen_xor(4), 4, 0),
        ("maj5", gen_maj(5), 5, 0),
        ("and6", gen_and(6), 6, 0),
        ("and8", gen_and(8), 8, 0),
        ("random10_0.5", gen_random_density(10, 0.5, 42), 10, 0),
    ] {
        let n_vars = n_vars as usize;
        
        // QM
        c.bench_with_input(
            BenchmarkId::new("compare", format!("{name}/qm")),
            &minterms,
            |b, mts| {
                let mts: Vec<u64> = mts.clone();
                b.iter(|| {
                    let qm = qm_agent::qm::quine_mccluskey::QuineMcCluskey::<Enc64>::new(n_vars);
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
                let mts: Vec<u64> = mts.clone();
                let tt = TruthTable::from_minterms(n_vars, &mts, &[])
                    .expect("valid truth table");
                let pi_depth = n_vars;
                b.iter(|| {
                    let pis = mc_find_pis(black_box(&tt), black_box(pi_depth));
                    black_box(pis.len())
                })
            },
        );
    }
}

criterion_group!(benches, bench_qm_pi, bench_mc_pi, bench_comparison);
criterion_main!(benches);
