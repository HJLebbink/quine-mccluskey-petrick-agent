// Performance benchmark with 16 variables
//
// UPDATE (2025-10-06): Now significantly faster with AVX512 SIMD optimization!
// Expected time: ~15-30 seconds (AVX512), ~60-120 seconds (scalar)
//
// Run with: cargo run --release --example slow_16var_problem
// Profile with: cargo build --release --example slow_16var_problem
//               then use profiler on: target/release/examples/slow_16var_problem.exe

use qm_agent::simplify::{parse_bool_expr, simplify_branches, BranchSet};
use std::time::Instant;

fn main() {
    println!("⚡ 16-Variable Performance Benchmark");
    println!("====================================");
    println!("This example demonstrates QM algorithm performance on 16 boolean variables.");
    println!("Expected: ~15-30 seconds (AVX512) or ~60-120 seconds (scalar)");
    println!();

    let start = Instant::now();

    // Create branch set with 16 boolean variables
    let mut branch_set = BranchSet::new();

    // Declare all 16 variables
    println!("⏱️  Step 1: Declaring 16 boolean variables...");
    branch_set.declare_bool("isPremium");
    branch_set.declare_bool("isEnterprise");
    branch_set.declare_bool("isTrial");
    branch_set.declare_bool("hasPaymentMethod");
    branch_set.declare_bool("isPaymentVerified");
    branch_set.declare_bool("isEmailVerified");
    branch_set.declare_bool("isPhoneVerified");
    branch_set.declare_bool("isAdmin");
    branch_set.declare_bool("isOwner");
    branch_set.declare_bool("isModerator");
    branch_set.declare_bool("hasAPIAccess");
    branch_set.declare_bool("hasBulkExport");
    branch_set.declare_bool("canInviteUsers");
    branch_set.declare_bool("canCreateTeams");
    branch_set.declare_bool("isRegionEU");
    branch_set.declare_bool("isRegionUS");
    println!("   ✓ Done in {:?}", start.elapsed());

    // Add 15 branches (14 return true, 1 return false)
    println!();
    println!("⏱️  Step 2: Adding 15 branches...");
    let branch_start = Instant::now();

    // Branch 1: Trial admin block (return false)
    let cond1 = parse_bool_expr("isTrial && isAdmin").unwrap();
    branch_set.add_branch(cond1, "return false");

    // Branch 2: Enterprise alone
    let cond2 = parse_bool_expr("isEnterprise").unwrap();
    branch_set.add_branch(cond2, "return true");

    // Branch 3-5: Enterprise + others (redundant)
    let cond3 = parse_bool_expr("isEnterprise && isEmailVerified").unwrap();
    branch_set.add_branch(cond3, "return true");

    let cond4 = parse_bool_expr("isEnterprise && isPaymentVerified").unwrap();
    branch_set.add_branch(cond4, "return true");

    let cond5 = parse_bool_expr("isEnterprise && hasPaymentMethod").unwrap();
    branch_set.add_branch(cond5, "return true");

    // Branch 6-7: Owner combinations
    let cond6 = parse_bool_expr("isOwner && isPremium").unwrap();
    branch_set.add_branch(cond6, "return true");

    let cond7 = parse_bool_expr("isOwner && isEnterprise").unwrap();
    branch_set.add_branch(cond7, "return true");

    // Branch 8-9: Admin combinations
    let cond8 = parse_bool_expr("isAdmin && isEnterprise").unwrap();
    branch_set.add_branch(cond8, "return true");

    let cond9 = parse_bool_expr("isAdmin && isPremium && isEmailVerified").unwrap();
    branch_set.add_branch(cond9, "return true");

    // Branch 10-12: Premium combinations
    let cond10 = parse_bool_expr("isPremium && isEmailVerified && isPaymentVerified").unwrap();
    branch_set.add_branch(cond10, "return true");

    let cond11 = parse_bool_expr("isPremium && hasPaymentMethod && isEmailVerified").unwrap();
    branch_set.add_branch(cond11, "return true");

    let cond12 = parse_bool_expr("isOwner && isEmailVerified && hasPaymentMethod").unwrap();
    branch_set.add_branch(cond12, "return true");

    // Branch 13-14: More complex combinations
    let cond13 = parse_bool_expr("isAdmin && hasBulkExport && isEnterprise").unwrap();
    branch_set.add_branch(cond13, "return true");

    let cond14 = parse_bool_expr("isPremium && canCreateTeams && isEmailVerified").unwrap();
    branch_set.add_branch(cond14, "return true");

    // Branch 15: Trial with specific conditions
    let cond15 = parse_bool_expr("isTrial && isEmailVerified && isPhoneVerified && isRegionUS").unwrap();
    branch_set.add_branch(cond15, "return true");

    branch_set.set_default("return false");

    println!("   ✓ Done in {:?}", branch_start.elapsed());
    println!("   Total variables: 16");
    println!("   Total branches: 15");
    println!("   Truth table size: 2^16 = 65,536 rows");

    // Simplification step (optimized with AVX512 SIMD)
    println!();
    println!("⏱️  Step 3: Running simplification (SIMD-optimized)...");
    println!("   Expected: ~15-30 seconds (AVX512) or ~60-120 seconds (scalar)");
    println!();
    println!("   Progress indicators:");

    let simplify_start = Instant::now();

    // This call will take a very long time
    match simplify_branches(&branch_set) {
        Ok(result) => {
            let duration = simplify_start.elapsed();
            println!();
            println!("   ✅ COMPLETED in {:?}!", duration);
            println!();
            println!("📊 Results:");
            println!("   Original branches: {}", result.original_branch_count);
            println!("   Simplified branches: {}", result.simplified_branch_count);
            println!("   Complexity reduction: {:.1}%", result.complexity_reduction());
            println!();
            println!("Total time: {:?}", start.elapsed());
        }
        Err(e) => {
            println!();
            println!("   ❌ Error: {}", e);
            println!();
            println!("Total time before error: {:?}", start.elapsed());
        }
    }

    println!();
    println!("🔍 Performance Analysis:");
    println!("   For 16 variables, the QM algorithm must:");
    println!("   1. Generate truth table: 65,536 rows");
    println!("   2. Find all prime implicants (exponential)");
    println!("   3. Apply Petrick's method (exponential worst-case)");
    println!("   ");
    println!("   ✅ OPTIMIZED (Oct 2025):");
    println!("   - Gray code checking: Vectorized with AVX512 SIMD");
    println!("   - 4-16x speedup on compatible CPUs");
    println!("   - Automatic fallback to scalar code");
    println!("   ");
    println!("   Remaining bottlenecks:");
    println!("   - Petrick's method for minimal cover");
    println!("   - Deduplication and sorting of implicants");
    println!("   ");
    println!("   Recommendation: Profile to find next optimization target");
}
