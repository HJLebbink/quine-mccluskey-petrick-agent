// Check: how many unique PIs does QM actually produce on the auth policy?
// QM formatted output shows strings — deduplicate them.
use qm_agent::qm::{Enc16, QMSolver};
use qm_agent::simplify::{BranchSet, analyzer::build_truth_table, parse_bool_expr};
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

    let minterms = extract_minterms(&branches);
    let n = 16;

    let var_names: Vec<String> = (0..n)
        .map(|i| char::from(b'A' + i as u8).to_string())
        .collect();

    let t0 = Instant::now();
    let mut solver = QMSolver::<Enc16>::new_with_variable_names(n as usize, var_names);
    solver.set_minterms(minterms.iter().map(|&x| x as u32).collect());
    let result = solver.solve();
    let total = t0.elapsed();

    // Deduplicate
    let mut unique_pis = std::collections::HashSet::new();
    for pi in &result.prime_implicants {
        unique_pis.insert(pi.clone());
    }

    let total_pis = result.prime_implicants.len();
    let unique = unique_pis.len();
    let duplicates = total_pis - unique;

    println!("=== QM PI Analysis (Auth Policy) ===\n");
    println!("Raw PI strings:       {}", total_pis);
    println!("Unique PIs:           {}", unique);
    println!("Duplicates:           {}", duplicates);
    println!(
        "Dedup ratio:          {:.1}%",
        (duplicates as f64 / total_pis as f64) * 100.0
    );
    println!("Total time:           {:?}\n", total);

    // Check unique QM vs min_cubes
    let t1 = Instant::now();
    solver.set_method(qm_agent::SolveMethod::MinCubes);
    let result2 = solver.solve();
    let mc_total = t1.elapsed();

    let mut mc_unique = std::collections::HashSet::new();
    for pi in &result2.prime_implicants {
        mc_unique.insert(pi.clone());
    }

    println!("CCubes PI strings:    {}", result2.prime_implicants.len());
    println!("CCubes unique PIs:    {}", mc_unique.len());
    println!("CCubes time:          {:?}", mc_total);

    // Compare sets
    let diff_qm_only: Vec<_> = unique_pis.difference(&mc_unique).collect();
    let diff_mc_only: Vec<_> = mc_unique
        .iter()
        .filter(|x| !unique_pis.contains(x.as_str()))
        .collect();

    println!("\n=== Set Diff ===");
    println!("PIs only in QM:       {}", diff_qm_only.len());
    println!("PIs only in CCubes:   {}", diff_mc_only.len());

    if !diff_qm_only.is_empty() {
        println!("\nQM-only PIs (first 5):");
        for pi in diff_qm_only.iter().take(5) {
            println!("  {}", pi);
        }
    }
}
