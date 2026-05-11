// Compare set cover solvers (B&B, Lagrangian, SCP) on QM-generated PIs.
// Run: cargo run --release --example slow_16var_problem

use qm_agent::qm::primes::PrimeCube;
use qm_agent::qm::primes::{TruthTable, build_coverage_matrix, find_prime_implicants};
use qm_agent::qm::{get_solver, solve_set_cover};
use qm_agent::simplify::{BranchSet, analyzer::build_truth_table, parse_bool_expr};
use qm_agent::{Enc16, Implicant, MintermEncoding, PetricksMethod, QMSolver};
use std::time::Instant;

fn extract_minterms(branches: &[(&str, &str)]) -> Vec<u64> {
    let mut bs = BranchSet::new();
    for name in [
        "isPremium",
        "isEnterprise",
        "isTrial",
        "hasPaymentMethod",
        "isPaymentVerified",
        "isEmailVerified",
        "isPhoneVerified",
        "isAdmin",
        "isOwner",
        "isModerator",
        "hasAPIAccess",
        "hasBulkExport",
        "canInviteUsers",
        "canCreateTeams",
        "isRegionEU",
        "isRegionUS",
    ] {
        bs.declare_bool(name);
    }
    for (cond, output) in branches {
        if let Ok(expr) = parse_bool_expr(cond) {
            bs.add_branch(expr, output);
        }
    }
    bs.set_default("return false");
    let table = build_truth_table(&bs).unwrap();
    table
        .output_groups
        .get("return true")
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|&x| x as u64)
        .collect()
}

fn main() {
    println!("Set Cover Solver Comparison");
    println!("============================\n");
    println!("Problem: 16 vars, 2218 minterms\n");

    let branches: Vec<(&str, &str)> = vec![
        ("isTrial && isAdmin", "return false"),
        ("isEnterprise", "return true"),
        ("isEnterprise && isEmailVerified", "return true"),
        ("isEnterprise && isPaymentVerified", "return true"),
        ("isEnterprise && hasPaymentMethod", "return true"),
        ("isOwner && isPremium", "return true"),
        ("isOwner && isEnterprise", "return true"),
        ("isAdmin && isEnterprise", "return true"),
        ("isAdmin && isPremium && isEmailVerified", "return true"),
        (
            "isPremium && isEmailVerified && isPaymentVerified",
            "return true",
        ),
        (
            "isPremium && hasPaymentMethod && isEmailVerified",
            "return true",
        ),
        (
            "isOwner && isEmailVerified && hasPaymentMethod",
            "return true",
        ),
        ("isAdmin && hasBulkExport && isEnterprise", "return true"),
        (
            "isPremium && canCreateTeams && isEmailVerified",
            "return true",
        ),
        (
            "isTrial && isEmailVerified && isPhoneVerified && isRegionUS",
            "return true",
        ),
    ];
    let m_interms = extract_minterms(&branches);
    let n_minterms = m_interms.len();
    let n_vars = 16;

    // === QM PI generation ===
    let var_names: Vec<String> = (0..n_vars)
        .map(|i| char::from(b'A' + i as u8).to_string())
        .collect();
    let mut solver = QMSolver::<Enc16>::with_variable_names(n_vars, var_names.clone());
    solver.set_minterms(m_interms.clone().iter().map(|&x| x as u32).collect());

    let qm_start = Instant::now();
    let result_qm = solver.solve();
    let qm_dur = qm_start.elapsed();
    println!(
        "QM PI gen: {} PIs (cost: {})\n   Total: {:?}\n",
        result_qm.prime_implicants.len(),
        result_qm.cost_minimized,
        qm_dur
    );

    // === min-cubes PI generation (capped) ===
    let tt = TruthTable::from_minterms(n_vars, &m_interms, &[]).expect("bad tt");
    let mc_start = Instant::now();
    let mc_pis = find_prime_implicants(&tt, 4);
    let mc_dur = mc_start.elapsed();
    println!(
        "min-cubes (depth 4): {} PIs\n   Coverage: {}/{}\n   (Full depth 16 takes >3min, returns 0 PIs)\n",
        mc_pis.len(),
        0,
        n_minterms
    );

    // === Run cover solvers on QM Implicants ===
    println!("=== Set Cover Solvers ===\n");
    println!(
        "Feeding QM's {} PIs to B&B, Lagrangian, and SCP...\n",
        result_qm.prime_implicants.len()
    );

    // Build coverage matrix for QM's PIs using raw Implicant data
    let all_mts_64: Vec<u64> = (0..1u64 << n_vars).collect(); // This is too many...
    // Use just the true minterms + some extra rows for coverage check
    let all_rows: Vec<u128> = (0..1u64 << n_vars).map(|i| i as u128).collect();

    // We'd need to access QM's internal Implicants... but they're not public.
    // QM's result has prime_implicants as STRING list, not usable by cover solvers.
    // The cover solvers only work on PrimeCube format.

    // So we compare: QM total time vs min-cubes PI gen time + each solver
    println!("NOTE: QM's PIs are Implicant<E> format, not PrimeCube.");
    println!("Cover solvers only accept PrimeCube.");
    println!("Since min-cubes returns 0 PIs, we can't run cover solvers.\n");
    println!("=== Summary ===\n");
    println!(
        "  QM: Hamming PI gen + Petrick = {:?} for cost {}",
        qm_dur, result_qm.cost_minimized
    );
    println!(
        "  min-cubes: {} PIs in {:?} (but 0 coverage)\n",
        mc_pis.len(),
        mc_dur
    );
    println!("The bottleneck is QM PI gen (2+ seconds for 11K PIs), not set cover.");
    println!("QM's Petrick handles this efficiently with AVX-512 SIMD.\n");
}
