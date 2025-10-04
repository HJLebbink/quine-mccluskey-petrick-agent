# Real-World Go Examples

These examples demonstrate the QM agent on realistic code patterns with significant complexity reduction.

## Example 1: Feature Access Control (50% reduction)

**File**: `go_feature_access.json`

**Scenario**: Multi-tier feature access system with overlapping permission checks.

**Original Code** (4 branches):
```go
func canAccessFeature(isPremium, isBeta, isAdmin bool) bool {
    if isPremium && isBeta {
        return true
    }
    if isPremium && isAdmin {
        return true
    }
    if isAdmin && isBeta {
        return true
    }
    if isAdmin {
        return true  // ← This makes branches 2 & 3 partially redundant
    }
    return false
}
```

**Result**:
- **50% reduction**: 4 branches → 2 branches
- **3 overlaps detected**: Branches have overlapping conditions
- **Simplified**: `isAdmin && isBeta || isPremium` → return true

**Key Insight**: The `isAdmin` alone branch makes other combinations with `isAdmin` partially redundant.

---

## Example 2: API Request Validation (20% reduction + dead code)

**File**: `go_api_validation.json`

**Scenario**: HTTP API authentication and rate limiting validation.

**Original Code** (5 branches):
```go
func validateAPIRequest(isAuthenticated, hasValidToken, isRateLimited bool) int {
    if !isAuthenticated {
        return http.StatusUnauthorized
    }
    if !isAuthenticated && !hasValidToken {  // ← DEAD CODE
        return http.StatusUnauthorized
    }
    if !hasValidToken && !isAuthenticated {  // ← DEAD CODE
        return http.StatusUnauthorized
    }
    if isAuthenticated && hasValidToken && isRateLimited {
        return http.StatusTooManyRequests
    }
    if isAuthenticated && hasValidToken && !isRateLimited {
        return http.StatusOK
    }
    return http.StatusInternalServerError
}
```

**Result**:
- **20% reduction**: 5 branches → 4 branches
- **2 dead code branches**: Lines 48 and 51 are unreachable
- **Analysis**: Branches 2 and 3 are fully covered by branch 1

**Key Insight**: Once `!isAuthenticated` is checked first, any additional checks that include `!isAuthenticated` are unreachable.

**Better Refactored Version**:
```go
func validateAPIRequest(isAuthenticated, hasValidToken, isRateLimited bool) int {
    if !isAuthenticated {
        return http.StatusUnauthorized
    }
    if isAuthenticated && hasValidToken && isRateLimited {
        return http.StatusTooManyRequests
    }
    if isAuthenticated && hasValidToken && !isRateLimited {
        return http.StatusOK
    }
    return http.StatusInternalServerError
}
```

---

## Example 3: Document Access Permissions (71.4% reduction!)

**File**: `go_document_access.json`

**Scenario**: Document management system with role-based access control.

**Original Code** (7 branches):
```go
func canAccessDocument(isOwner, isAdmin, isEditor, isPublic bool) bool {
    if isOwner {
        return true
    }
    if isAdmin {
        return true
    }
    if isOwner && isEditor {       // ← DEAD CODE
        return true
    }
    if isAdmin && isEditor {       // ← DEAD CODE
        return true
    }
    if isPublic && isEditor {
        return true
    }
    if isOwner && isPublic {       // ← DEAD CODE
        return true
    }
    if isAdmin && isPublic {       // ← DEAD CODE
        return true
    }
    return false
}
```

**Result**:
- **71.4% reduction**: 7 branches → 2 branches
- **4 dead code branches**: Lines 29, 32, 38, 41 are unreachable
- **Coverage**: 81.25%
- **All branches overlap** with earlier conditions

**Simplified Logic**:
```go
// QM-AGENT-ORIGINAL:
// [original 7-branch code preserved as comments]

// QM-AGENT-SIMPLIFIED
if (isAdmin && isOwner || isPublic) || isEditor {
    return true
}
return false
```

**Even Cleaner Refactor** (human-readable):
```go
func canAccessDocument(isOwner, isAdmin, isEditor, isPublic bool) bool {
    return isOwner || isAdmin || isEditor || isPublic
}
```

**Key Insights**:
1. Branches 3, 4, 6, 7 combine `isOwner` or `isAdmin` with other conditions
2. Since `isOwner` and `isAdmin` **alone** grant access (branches 1 & 2), any combination including them is redundant
3. The logic reduces to a simple OR of all four variables

---

## Summary Comparison

| Example | Original Branches | Simplified | Reduction | Dead Code | Key Issue |
|---------|------------------|------------|-----------|-----------|-----------|
| Feature Access | 4 | 2 | **50%** | 0 | Overlapping permissions |
| API Validation | 5 | 4 | **20%** | 2 | Redundant auth checks |
| Document Access | 7 | 2 | **71.4%** | 4 | Redundant role combinations |

## Common Patterns Found

1. **Single-condition branches making compound conditions redundant**
   - `if isAdmin { ... }` makes `if isAdmin && X { ... }` redundant

2. **Duplicate conditions in different order**
   - `!isAuthenticated && !hasValidToken` vs `!hasValidToken && !isAuthenticated`

3. **Overcomplicated permission checks**
   - Multiple OR conditions that could be single boolean expression

## Running These Examples

```bash
# Feature access (50% reduction)
cargo run -- simplify -i examples/agent/go_feature_access.json

# API validation (dead code detection)
cargo run -- simplify -i examples/agent/go_api_validation.json

# Document access (71% reduction!)
cargo run -- simplify -i examples/agent/go_document_access.json
```

## When QM Agent is Most Useful

1. ✅ **Refactoring legacy code** with accumulated conditionals
2. ✅ **Code reviews** to find redundant checks
3. ✅ **Security audits** to ensure all paths are tested (coverage analysis)
4. ✅ **Permission systems** with overlapping roles
5. ✅ **State machines** with complex transitions
6. ✅ **Input validation** with redundant checks

## Limitations Demonstrated

- The agent produces **minimal SOP (Sum of Products)** form
- Human-readable refactoring may differ from mathematical minimum
- Complex conditions may need manual simplification after QM analysis
- The agent doesn't understand domain semantics (just boolean algebra)

## Best Practice

1. Run QM agent to find dead code and overlaps
2. Use complexity reduction as a guide
3. Apply domain knowledge to make final refactoring decision
4. Always include original code in markers for easy reversion
