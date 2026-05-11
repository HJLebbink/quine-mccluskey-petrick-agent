// Trace: find ONE example duplicate in the 3-var OR(3) problem to understand the root cause
use qm_agent::qm::{Enc32, QMSolver};

fn main() {
    // f(A,B,C) = OR(3) = Σ(1,2,3,4,5,6,7)
    let mut solver = QMSolver::<Enc32>::new(3);
    solver.set_minterms(vec![1, 2, 3, 4, 5, 6, 7]);
    let result = solver.solve();

    println!("=== 3-var OR(3): OR of A,B,C ===\n");
    println!(
        "Minterms: {}",
        result.prime_implicants.iter().filter(|_| true).count()
    );
    println!("PIs:\n");
    for (i, pi) in result.prime_implicants.iter().enumerate() {
        println!("  [{}] {}", i, pi);
    }

    // Now with minterm 0 added → all-ones → one PI
    println!("\n=== 3-var ALL-ONES: Σ(0,1,2,3,4,5,6,7) ===\n");
    let mut solver2 = QMSolver::<Enc32>::new(3);
    solver2.set_minterms(vec![0, 1, 2, 3, 4, 5, 6, 7]);
    let result2 = solver2.solve();
    println!("PIs:");
    for (i, pi) in result2.prime_implicants.iter().enumerate() {
        println!("  [{}] {}", i, pi);
    }

    // Check for duplicate PI in small problem
    let mut seen = std::collections::HashMap::new();
    for (i, pi) in result.prime_implicants.iter().enumerate() {
        seen.entry(pi.clone()).or_insert(Vec::new()).push(i);
    }
    println!("\n=== Duplicate check on OR(3) ===");
    for (pi, positions) in &seen {
        if positions.len() > 1 {
            println!("Duplicate: '{}' at positions: {:?}", pi, positions);
        }
    }
    if seen.values().all(|v| v.len() == 1) {
        println!("No duplicates found in OR(3).");
    }

    // Check all-ones
    let mut seen2 = std::collections::HashMap::new();
    for (i, pi) in result2.prime_implicants.iter().enumerate() {
        seen2.entry(pi.clone()).or_insert(Vec::new()).push(i);
    }
    println!("\n=== Duplicate check on ALL-ONES ===");
    for (pi, positions) in &seen2 {
        if positions.len() > 1 {
            println!("Duplicate: '{}' at positions: {:?}", pi, positions);
        }
    }
    if seen2.values().all(|v| v.len() == 1) {
        println!("No duplicates found in ALL-ONES.");
    }

    // Try a simpler case: 3 bits, minterm 0 only
    println!("\n=== 3-var: just minterm 0 (A'B'C') ===\n");
    let mut solver3 = QMSolver::<Enc32>::new(3);
    solver3.set_minterms(vec![0]);
    let result3 = solver3.solve();
    println!("PIs:");
    for (i, pi) in result3.prime_implicants.iter().enumerate() {
        println!("  [{}] {}", i, pi);
    }

    // Now: what about f(A,B,C) = Σ(0,1)? Only 2 minterms
    println!("\n=== 3-var: Σ(0,1) ===\n");
    let mut solver4 = QMSolver::<Enc32>::new(3);
    solver4.set_minterms(vec![0, 1]);
    let result4 = solver4.solve();
    println!("PIs:");
    for (i, pi) in result4.prime_implicants.iter().enumerate() {
        println!("  [{}] {}", i, pi);
    }

    // Check for duplicates
    let mut seen4 = std::collections::HashMap::new();
    for (i, pi) in result4.prime_implicants.iter().enumerate() {
        seen4.entry(pi.clone()).or_insert(Vec::new()).push(i);
    }
    for (pi, positions) in &seen4 {
        if positions.len() > 1 {
            println!("Duplicate: '{}' at positions: {:?}", pi, positions);
        }
    }
    if seen4.values().all(|v| v.len() == 1) {
        println!("No duplicates found.");
    }
}
