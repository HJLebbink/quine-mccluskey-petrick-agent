use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use qm_agent::qm::min_cubes::{
    BnBSolver, LagrangianSolver, SCP_solver, SetCoverSolver, covers,
    find_prime_implicants as mc_find_pis, get_solver, primes::PrimeCube, solve_set_cover,
};
use std::hint::black_box;

// ============================================================================
// TEST CASE GENERATORS
// ============================================================================

fn gen_and(n: u8) -> Vec<u64> {
    vec![(1u64 << n) - 1]
}

fn gen_or(n: u8) -> Vec<u64> {
    let all = (1u64 << n) - 1;
    (0..all).collect()
}

fn gen_xor(n: u8) -> Vec<u64> {
    (0..(1u64 << n))
        .filter(|&i| i.count_ones() & 1 == 1)
        .collect()
}

fn gen_random(n: u8, seed: u64, density: f64) -> Vec<u64> {
    let limit = 1u64 << n;
    let mut rng = seed;
    (0..limit)
        .filter(|&i| {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (rng as f64 / u64::MAX as f64) < density
        })
        .collect()
}

// ============================================================================
// HELPER: Generate PiCubes from minterms
// ============================================================================

fn get_pis_and_minterms(name: &str, n: u8, minterms: &[u64]) -> (Vec<PrimeCube>, Vec<u64>) {
    let tt = qm_agent::qm::min_cubes::TruthTable::from_minterms(n as usize, minterms, &[])
        .unwrap_or_else(|| panic!("{}: invalid truth table", name));
    let pis = mc_find_pis(&tt, n as usize);
    (pis, minterms.to_vec())
}

// ============================================================================
// BENCHMARK: B&B Solver
// ============================================================================

fn bench_bnb(c: &mut Criterion) {
    let mut group = c.benchmark_group("solver_bnb");

    let cases = [
        ("and3", gen_and(3), 3),
        ("or3", gen_or(3), 3),
        ("xor3", gen_xor(3), 3),
        ("xor4", gen_xor(4), 4),
        (
            "maj5",
            gen_and(5)
                .iter()
                .cloned()
                .chain((0..8).filter(|&i| i.count_ones() >= 3))
                .collect(),
            5,
        ),
        ("rand10", gen_random(10, 42, 0.5), 10),
        ("rand12", gen_random(12, 42, 0.3), 12),
    ];

    for (name, minterms, n) in cases {
        let (pis, mts) = get_pis_and_minterms(name, n, &minterms);

        group.throughput(Throughput::Elements(mts.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("bnb", name),
            &(pis, mts),
            |b, (pis, mts)| {
                let solver = BnBSolver::default();
                b.iter(|| {
                    let sol = solve_set_cover(&solver, black_box(pis), black_box(mts));
                    black_box(sol.num_selected)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// BENCHMARK: Lagrangian Solver
// ============================================================================

fn bench_lagrangian(c: &mut Criterion) {
    let mut group = c.benchmark_group("solver_lagrangian");

    let cases = [
        ("and3", gen_and(3), 3),
        ("xor3", gen_xor(3), 3),
        ("xor4", gen_xor(4), 4),
        ("rand10", gen_random(10, 42, 0.5), 10),
        ("rand12", gen_random(12, 42, 0.3), 12),
    ];

    for (name, minterms, n) in cases {
        let (pis, mts) = get_pis_and_minterms(name, n, &minterms);

        group.bench_with_input(
            BenchmarkId::new("lagrangian", name),
            &(pis, mts),
            |b, (pis, mts)| {
                let solver = LagrangianSolver::default();
                b.iter(|| {
                    let sol = solve_set_cover(&solver, black_box(pis), black_box(mts));
                    black_box(sol.num_selected)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// BENCHMARK: SCP Solver
// ============================================================================

fn bench_scp(c: &mut Criterion) {
    let mut group = c.benchmark_group("solver_scp");

    let cases = [
        ("and3", gen_and(3), 3),
        ("xor3", gen_xor(3), 3),
        ("xor4", gen_xor(4), 4),
    ];

    for (name, minterms, n) in cases {
        let (pis, mts) = get_pis_and_minterms(name, n, &minterms);

        group.bench_with_input(
            BenchmarkId::new("scp", name),
            &(pis, mts),
            |b, (pis, mts)| {
                let solver = SCP_solver::default();
                b.iter(|| {
                    let sol = solve_set_cover(&solver, black_box(pis), black_box(mts));
                    black_box(sol.num_selected)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// BENCHMARK: Cross-Solver Comparison
// ============================================================================

fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("solver_comparison");

    let cases = [
        ("and3", gen_and(3), 3),
        ("xor3", gen_xor(3), 3),
        ("xor4", gen_xor(4), 4),
        ("rand10", gen_random(10, 42, 0.5), 10),
    ];

    for (name, minterms, n) in cases {
        let (pis, mts) = get_pis_and_minterms(name, n, &minterms);

        // Test each solver
        for solver_type in &["bnb", "lagrangian", "scp"] {
            let solver = get_solver(solver_type);

            group.bench_with_input(
                BenchmarkId::new(name, solver.name()),
                &(pis, mts),
                |b, (pis, mts)| {
                    let solver = get_solver(solver_type);
                    b.iter(|| {
                        let sol = solve_set_cover(solver.as_ref(), black_box(pis), black_box(mts));
                        black_box(sol.num_selected)
                    })
                },
            );
        }
    }

    group.finish();
}

// ============================================================================
// CRITERION SETUP
// ============================================================================

criterion_group!(
    benches,
    bench_bnb,
    bench_lagrangian,
    bench_scp,
    bench_comparison,
);

criterion_main!(benches);
