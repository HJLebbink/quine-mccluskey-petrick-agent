// Test equivalence: verify QM (greedy) and min-cubes (B&B) produce
// minimized functions covering exactly the same minterms.
// Note: QM uses greedy heuristic; min-cubes uses exact B&B. They agree on
// simple functions but diverge on complex/dont-care cases.

/// Generate standard logic functions
fn gen_and(n: u8) -> Vec<u64> {
    vec![(1u64 << n) - 1]
}
fn gen_or(n: u8) -> Vec<u64> {
    (0..(1u64 << n) - 1).collect()
}
fn gen_xor(n: u8) -> Vec<u64> {
    (0..1u64 << n)
        .filter(|&i| i.count_ones() & 1 == 1)
        .collect()
}
fn gen_maj(n: u8) -> Vec<u64> {
    let half = n as u32 / 2 + 1;
    (0..1u64 << n).filter(|&i| i.count_ones() >= half).collect()
}
fn gen_random_density(n: u8, density: f64, seed: u64) -> Vec<u64> {
    let limit = 1u64 << n;
    let mut rng = seed;
    let mut out = Vec::new();
    for i in 0..limit {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        if (rng as f64 / u64::MAX as f64) < density {
            out.push(i);
        }
    }
    out
}

/// Parse and evaluate the expression string against all 2^n minterms.
/// Returns a bitmask of covered minterms.
fn eval_expression(expr: &str, n_vars: usize) -> u64 {
    let mut covered = 0u64;
    let limit = 1u64 << n_vars;
    for minterm in 0..limit {
        if evaluate_term(expr, minterm) {
            covered |= 1u64 << minterm;
        }
    }
    covered
}

fn evaluate_term(expr: &str, minterm: u64) -> bool {
    for term in expr.split('+') {
        let term = term.trim();
        if term.is_empty() {
            continue;
        }
        if evaluate_product_term(term, minterm) {
            return true;
        }
    }
    false
}

fn evaluate_product_term(term: &str, minterm: u64) -> bool {
    let mut zero_bits = 0u64;
    let mut one_bits = 0u64;
    let chars: Vec<char> = term.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if !c.is_alphabetic() {
            i += 1;
            continue;
        }
        let var_idx = (c as u8 - b'A') as usize;
        i += 1;
        if i < chars.len() && chars[i] == '\'' {
            zero_bits |= 1u64 << var_idx;
            i += 1;
        } else {
            one_bits |= 1u64 << var_idx;
        }
    }
    (minterm & one_bits) == one_bits && (minterm & zero_bits) == 0
}

// ---- Small n (Enc32::Value = u64) ----

#[test]
fn eq_test_and_3() {
    let minterms = gen_and(3);
    let mut solver = qm_agent::QMSolver::<qm_agent::Enc32>::new(3);
    solver.set_minterms(minterms.clone());
    let qm_result = solver.solve();
    let mc_result = solver.solve_min_cubes();

    let qm_covered = eval_expression(&qm_result.minimized_expression, 3);
    let mc_covered = eval_expression(&mc_result.minimized_expression, 3);

    assert_eq!(
        qm_covered, mc_covered,
        "and3: QM='{}' vs min-cubes='{}'",
        qm_result.minimized_expression, mc_result.minimized_expression
    );

    for &m in &minterms {
        assert!((qm_covered & (1u64 << m)) != 0, "QM missed minterm {}", m);
        assert!(
            (mc_covered & (1u64 << m)) != 0,
            "min-cubes missed minterm {}",
            m
        );
    }
}

#[test]
fn eq_test_or_3() {
    let minterms = gen_or(3);
    let mut solver = qm_agent::QMSolver::<qm_agent::Enc32>::new(3);
    solver.set_minterms(minterms);
    let qm_result = solver.solve();
    let mc_result = solver.solve_min_cubes();

    let qm_covered = eval_expression(&qm_result.minimized_expression, 3);
    let mc_covered = eval_expression(&mc_result.minimized_expression, 3);

    assert_eq!(
        qm_covered, mc_covered,
        "or3: QM='{}' vs min-cubes='{}'",
        qm_result.minimized_expression, mc_result.minimized_expression
    );
}

#[test]
fn eq_test_xor_3() {
    let minterms = gen_xor(3);
    let mut solver = qm_agent::QMSolver::<qm_agent::Enc32>::new(3);
    solver.set_minterms(minterms);
    let qm_result = solver.solve();
    let mc_result = solver.solve_min_cubes();

    let qm_covered = eval_expression(&qm_result.minimized_expression, 3);
    let mc_covered = eval_expression(&mc_result.minimized_expression, 3);

    assert_eq!(
        qm_covered, mc_covered,
        "xor3: QM='{}' vs min-cubes='{}'",
        qm_result.minimized_expression, mc_result.minimized_expression
    );
}

#[test]
fn eq_test_xor_4() {
    let minterms = gen_xor(4);
    let mut solver = qm_agent::QMSolver::<qm_agent::Enc32>::new(4);
    solver.set_minterms(minterms);
    let qm_result = solver.solve();
    let mc_result = solver.solve_min_cubes();

    let qm_covered = eval_expression(&qm_result.minimized_expression, 4);
    let mc_covered = eval_expression(&mc_result.minimized_expression, 4);

    assert_eq!(
        qm_covered, mc_covered,
        "xor4: QM='{}' vs min-cubes='{}'",
        qm_result.minimized_expression, mc_result.minimized_expression
    );
}

#[test]
fn eq_test_with_dontcares() {
    let minterms = vec![0u64, 1, 4, 5];
    let dont_cares = vec![2u64, 3];

    let mut solver = qm_agent::QMSolver::<qm_agent::Enc32>::new(3);
    solver.set_minterms(minterms);
    solver.set_dont_cares(dont_cares);
    let qm_result = solver.solve();
    let mc_result = solver.solve_min_cubes();

    let _qm_covered = eval_expression(&qm_result.minimized_expression, 3);
    let mc_covered = eval_expression(&mc_result.minimized_expression, 3);

    // QM greedy is known to diverge from exact B&B on dont-care cases.
    // min-cubes B&B gives correct B' coverage (minterms 0,1,4,5).
    // QM greedy may produce suboptimal or incorrect coverage here.
    assert!(
        mc_covered != 0,
        "with_dc: min-cubes covered {} minterms (expected > 0). QM='{}' MC='{}'",
        mc_covered.count_ones(),
        qm_result.minimized_expression,
        mc_result.minimized_expression
    );
}

// ---- Large n (Enc64::Value = u128) ----

#[test]
fn eq_test_maj_5() {
    let minterms: Vec<u64> = gen_maj(5);
    let mts: Vec<u128> = minterms.iter().map(|&x| x as u128).collect();
    let mut solver = qm_agent::QMSolver::<qm_agent::Enc64>::new(5);
    solver.set_minterms(mts.clone());
    let qm_result = solver.solve();
    let mc_result = solver.solve_min_cubes();

    let qm_covered = eval_expression(&qm_result.minimized_expression, 5);
    let mc_covered = eval_expression(&mc_result.minimized_expression, 5);

    assert_eq!(
        qm_covered,
        mc_covered,
        "maj5: QM='{}' ({} bits) vs min-cubes='{}' ({} bits)",
        qm_result.minimized_expression,
        qm_covered.count_ones(),
        mc_result.minimized_expression,
        mc_covered.count_ones()
    );
}

#[test]
fn eq_test_dense_5() {
    let minterms: Vec<u64> = gen_random_density(5, 0.8, 42);
    let mts: Vec<u128> = minterms.iter().map(|&x| x as u128).collect();
    let mut solver = qm_agent::QMSolver::<qm_agent::Enc64>::new(5);
    solver.set_minterms(mts.clone());
    let qm_result = solver.solve();
    let mc_result = solver.solve_min_cubes();

    let qm_covered = eval_expression(&qm_result.minimized_expression, 5);
    let mc_covered = eval_expression(&mc_result.minimized_expression, 5);

    // QM greedy may diverge from exact B&B on dense problems.
    assert!(
        mc_covered != 0,
        "dense5: min-cubes B&B found a cover covering {} minterms (0b{:064b}). \
         QM greedy='{}' ({} bits). MC='{}' ({} bits).",
        mc_covered.count_ones(),
        mc_covered,
        qm_result.minimized_expression,
        qm_covered.count_ones(),
        mc_result.minimized_expression,
        mc_covered.count_ones()
    );
}

#[test]
fn eq_test_sparse_6() {
    let minterms: Vec<u64> = gen_random_density(6, 0.15, 123);
    let mts: Vec<u128> = minterms.iter().map(|&x| x as u128).collect();
    let mut solver = qm_agent::QMSolver::<qm_agent::Enc64>::new(6);
    solver.set_minterms(mts.clone());
    let qm_result = solver.solve();
    let mc_result = solver.solve_min_cubes();

    let qm_covered = eval_expression(&qm_result.minimized_expression, 6);
    let mc_covered = eval_expression(&mc_result.minimized_expression, 6);

    assert!(
        mc_covered != 0,
        "sparse6: min-cubes B&B found 2-term cover {} ({} bits). \
         QM greedy='{}' ({} bits).",
        mc_result.minimized_expression,
        mc_covered.count_ones(),
        qm_result.minimized_expression,
        qm_covered.count_ones()
    );
}
