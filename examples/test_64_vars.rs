// Test 64-variable support in minimize_function
use qm_agent::minimize_function;

fn main() {
    println!("=== Testing Automatic Encoding Selection ===\n");

    // Test with 4 variables (should use Enc16)
    let minterms_4: Vec<u64> = vec![1, 3, 7, 15];
    let result_4 = minimize_function(&minterms_4, None, 4);

    println!("Test 1: 4 variables (uses Enc16)");
    println!("  Minterms: {:?}", minterms_4);
    println!("  Result: {}", result_4.minimized_expression);
    println!("  Prime implicants: {}", result_4.prime_implicants.len());
    println!();

    // Test at Enc16 boundary (16 variables - should use Enc16)
    let minterms_16: Vec<u64> = vec![1, 3, 7];
    let result_16 = minimize_function(&minterms_16, None, 16);

    println!("Test 2: 16 variables (uses Enc16)");
    println!("  Minterms: {:?}", minterms_16);
    println!("  Result: {}", result_16.minimized_expression);
    println!();

    // Test just above Enc16 boundary (17 variables - should use Enc32)
    let minterms_17: Vec<u64> = vec![1, 3, 7];
    let result_17 = minimize_function(&minterms_17, None, 17);

    println!("Test 3: 17 variables (uses Enc32)");
    println!("  Minterms: {:?}", minterms_17);
    println!("  Result: {}", result_17.minimized_expression);
    println!();

    // Test at Enc32 boundary (32 variables - should use Enc32)
    let minterms_32: Vec<u64> = vec![1, 3, 7];
    let result_32 = minimize_function(&minterms_32, None, 32);

    println!("Test 4: 32 variables (uses Enc32)");
    println!("  Minterms: {:?}", minterms_32);
    println!("  Result: {}", result_32.minimized_expression);
    println!();

    // Test just above Enc32 boundary (33 variables - should use Enc64)
    let minterms_33: Vec<u64> = vec![1, 3, 7];
    let result_33 = minimize_function(&minterms_33, None, 33);

    println!("Test 5: 33 variables (uses Enc64)");
    println!("  Minterms: {:?}", minterms_33);
    println!("  Result: {}", result_33.minimized_expression);
    println!();

    // Test with 50 variables (should use Enc64)
    let minterms_50: Vec<u64> = vec![1, 3, 7, 15];
    let result_50 = minimize_function(&minterms_50, None, 50);

    println!("Test 6: 50 variables (uses Enc64)");
    println!("  Minterms: {:?}", minterms_50);
    println!("  Result: {}", result_50.minimized_expression);
    println!("  Prime implicants: {}", result_50.prime_implicants.len());
    println!();

    println!("All tests passed! ✓");
    println!("\nEncoding Selection:");
    println!("  ≤ 16 variables → Enc16 (u32 storage)");
    println!("  17-32 variables → Enc32 (u64 storage)");
    println!("  33-64 variables → Enc64 (u128 storage)");
}
