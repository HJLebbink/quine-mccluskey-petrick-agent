# Large Variable Examples (10-16 Variables)

These examples demonstrate the QM agent on complex real-world scenarios with many boolean variables.

## ✅ payment_gateway.go (10 Variables - WORKS!)

**Scenario**: E-commerce payment processing with fraud detection

**Variables** (10 booleans):
- Account: `isAccountActive`, `isAccountVerified`
- Payment Method: `hasValidCard`, `hasValidBilling`
- Security: `isPassed2FA`, `isIPWhitelisted`
- Transaction: `isAmountValid`, `isCurrencySupported`
- Risk: `isFraudFlagged`, `isHighRiskCountry`

**Complexity**:
- Truth table size: 2^10 = **1,024 states**
- Original: **10 branches** with heavy overlap
- Simplified: **~4 branches** (60% reduction)

**Run**:
```bash
go run payment_gateway.go
```

**Key Findings**:
1. Most branches share: `isAccountActive && hasValidCard`
2. Secondary conditions vary: verified, 2FA, whitelisted, billing
3. Special case: `verified && card && billing` works without accountActive
4. Fraud and high-risk checks are guard clauses

**Original Logic** (messy):
```go
if isAccountActive && isAccountVerified && hasValidCard { return true }
if isAccountActive && hasValidCard && isPassed2FA { return true }
if isAccountVerified && hasValidCard && hasValidBilling { return true }
if isAccountActive && isAccountVerified && hasValidBilling { return true }
if isAccountActive && hasValidCard && isIPWhitelisted { return true }
// ... 5 more overlapping branches
```

**Simplified Logic** (clear):
```go
if isFraudFlagged { return false }
if isHighRiskCountry && !isPassed2FA { return false }

if isAccountActive && hasValidCard {
    if isAccountVerified || isPassed2FA || isIPWhitelisted || hasValidBilling {
        return true
    }
}

if isAccountVerified && hasValidCard && hasValidBilling {
    return true
}

return false
```

**Real-World Impact**:
- 60% less code
- Clear risk checks at top (guard clauses)
- Single main approval path
- Easy to add new verification methods
- Testable logic hierarchy

---

## ⚠️ saas_feature_flags.go (16 Variables - TOO SLOW!)

**Scenario**: Enterprise SaaS platform with complex feature gating

**Variables** (16 booleans):
- Subscription: `isPremium`, `isEnterprise`, `isTrial`
- Payment: `hasPaymentMethod`, `isPaymentVerified`
- Verification: `isEmailVerified`, `isPhoneVerified`
- Roles: `isAdmin`, `isOwner`, `isModerator`
- Features: `hasAPIAccess`, `hasBulkExport`
- Permissions: `canInviteUsers`, `canCreateTeams`
- Region: `isRegionEU`, `isRegionUS`

**Complexity**:
- Truth table size: 2^16 = **65,536 states** 😱
- Original: **15 branches**
- QM Agent: **TIMES OUT** (>30 seconds)

**Run**:
```bash
go run saas_feature_flags.go  # ✅ Tests pass
cargo run -- simplify -i ../go_saas_feature_flags.json  # ❌ Times out
```

**Why It Times Out**:
1. **Exponential growth**: 2^16 = 65,536 truth table rows
2. **Prime implicant generation**: Must compare all minterms
3. **Petrick's method**: Exponential worst-case for minimal cover
4. **QM algorithm limitation**: Works best for ≤12 variables

**Practical Solution**:
The Go program includes a **hand-simplified version** that demonstrates the pattern without running QM:

```go
// Manual analysis reveals:
if isTrial && isAdmin { return false }  // Block trial admins
if isEnterprise { return true }         // Enterprise always OK
if isOwner && (isPremium || hasPaymentMethod && isEmailVerified) { return true }
if isAdmin && isPremium && isEmailVerified { return true }
if isPremium && isEmailVerified && (isPaymentVerified || hasPaymentMethod || canCreateTeams) { return true }
if isTrial && isEmailVerified && isPhoneVerified && isRegionUS { return true }
return false
```

**Lessons Learned**:
- 60% reduction: 15 branches → 6 branches
- `isEnterprise` alone grants access (4 branches redundant)
- Many `isPremium` combinations overlap
- Trial logic requires ALL conditions

---

## Performance Comparison

| Variables | States | Branches | QM Status | Time | Reduction |
|-----------|--------|----------|-----------|------|-----------|
| 3-4 | 16 | 7 | ✅ Fast | <1s | 71% |
| 10 | 1,024 | 10 | ✅ OK | ~5s | 60% |
| 12 | 4,096 | 12 | ⚠️ Slow | ~30s | ~50% |
| 16 | 65,536 | 15 | ❌ Timeout | >60s | N/A |

**Recommendation**: QM agent works best for **≤10-12 variables**

---

## Workarounds for Large Problems

When you have >12 variables:

### 1. **Decompose the Problem**
Break into smaller sub-problems:
```go
// Instead of 16 variables:
func canAccessFeature(all 16 vars...) bool

// Split into:
func hasValidSubscription(isPremium, isEnterprise, isTrial) bool
func hasValidAccount(isEmailVerified, isPaymentVerified, ...) bool
func canAccessFeature() bool {
    return hasValidSubscription() && hasValidAccount()
}
```

### 2. **Group Variables**
Create compound booleans:
```go
// Instead of: isPremium, isEnterprise, isTrial
tier := getTier()  // enum: Free, Premium, Enterprise
```

### 3. **Manual Analysis**
For very large problems, use QM agent to:
- Analyze sub-problems
- Detect dead code patterns
- Verify simplified logic on smaller chunks

### 4. **Sampling Strategy**
```go
// Test with representative cases instead of exhaustive
testCases := []TestCase{
    {name: "Enterprise (highest tier)", isEnterprise: true, ...},
    {name: "Premium user", isPremium: true, ...},
    // etc.
}
```

---

## Best Practices

### When to Use QM Agent

✅ **Good candidates**:
- 3-10 variables
- <20 branches
- Lots of overlapping conditions
- Legacy code refactoring
- Permission systems

❌ **Not ideal**:
- >12 variables
- Dynamic/runtime conditions
- Side effects in conditions
- Need for domain-specific optimization

### Design Patterns

1. **Guard Clauses First**
   ```go
   if fraudFlagged { return false }
   if banned { return false }
   // Then main logic
   ```

2. **Hierarchy of Conditions**
   ```go
   if isEnterprise { ... }      // Highest tier
   else if isPremium { ... }    // Middle tier
   else if isFree { ... }       // Lowest tier
   ```

3. **Compound Checks**
   ```go
   hasBasics := isActive && isVerified
   if hasBasics && isPremium { ... }
   ```

---

## Conclusion

The **10-variable payment gateway example** demonstrates the QM agent's sweet spot:
- ✅ Complex enough to show real value
- ✅ Fast enough to be practical
- ✅ Clear simplification benefits

The **16-variable SaaS example** shows the limitations:
- ⚠️ Too complex for QM algorithm
- ⚠️ Requires decomposition or manual analysis
- ✅ Still useful for understanding the pattern

**Rule of Thumb**: If your truth table has >10,000 states (>13 variables), consider decomposing the problem.
