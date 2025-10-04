# Go Code Examples

These are **actual runnable Go programs** demonstrating the code patterns that the QM agent can optimize.

## Running the Examples

### Basic Examples (3-4 variables)
```bash
# Feature access control (50% reduction)
go run feature_access.go

# API validation (dead code detection)
go run api_validation.go

# Document access (71% reduction!)
go run document_access.go
```

### Large Example (10 variables) ✨
```bash
# Payment gateway (60% reduction)
go run payment_gateway.go
```

See **[LARGE_EXAMPLES.md](LARGE_EXAMPLES.md)** for details on handling 10-16 variable problems.

## Example 1: feature_access.go

**Problem**: Feature access with overlapping permission checks

```go
func canAccessFeature(isPremium, isBeta, isAdmin bool) bool {
    if isPremium && isBeta { return true }
    if isPremium && isAdmin { return true }
    if isAdmin && isBeta { return true }
    if isAdmin { return true }  // ← Makes above redundant!
    return false
}
```

**Result**: 50% reduction (4 branches → 2)

## Example 2: api_validation.go

**Problem**: API request validation with dead code

```go
func validateAPIRequest(isAuthenticated, hasValidToken, isRateLimited bool) int {
    if !isAuthenticated { return Unauthorized }
    if !isAuthenticated && !hasValidToken { return Unauthorized }  // ← DEAD CODE
    if !hasValidToken && !isAuthenticated { return Unauthorized }  // ← DEAD CODE
    ...
}
```

**Result**: 2 dead code branches detected, 20% reduction

**Simplified**:
```go
func validateAPIRequest(isAuthenticated, hasValidToken, isRateLimited bool) int {
    if !isAuthenticated { return Unauthorized }
    // Dead code removed - above condition makes them unreachable
    if isAuthenticated && hasValidToken && isRateLimited { return TooManyRequests }
    if isAuthenticated && hasValidToken && !isRateLimited { return OK }
    return InternalServerError
}
```

## Example 3: document_access.go (Most Dramatic!)

**Problem**: Document permissions with massive redundancy

```go
func canAccessDocument(isOwner, isAdmin, isEditor, isPublic bool) bool {
    if isOwner { return true }
    if isAdmin { return true }
    if isOwner && isEditor { return true }  // ← DEAD CODE
    if isAdmin && isEditor { return true }  // ← DEAD CODE
    if isPublic && isEditor { return true }
    if isOwner && isPublic { return true }  // ← DEAD CODE
    if isAdmin && isPublic { return true }  // ← DEAD CODE
    return false
}
```

**Result**: 57% reduction (7 branches → 3 branches)

**Simplified**:
```go
func canAccessDocument(isOwner, isAdmin, isEditor, isPublic bool) bool {
    if isOwner || isAdmin {
        return true
    }
    if isPublic && isEditor {
        return true
    }
    return false
}
```

**Why the dead code?**
- Since `isOwner` and `isAdmin` **alone** grant access
- Any combination including them (like `isOwner && isEditor`) is redundant
- Once `isOwner` is true, the function already returned!
- The original logic requires **both** `isPublic && isEditor`, not separately

## Testing

Each program includes test cases covering all input combinations and verifies that:
1. The original version produces correct output
2. The simplified version produces identical output
3. All edge cases are handled

Run them to see the analysis output:

```bash
$ go run document_access.go

Document Access Control Test:
Owner | Admin | Editor | Public | Access
------|-------|--------|--------|-------
  false   |  false   |   false   |   false   | false ✓
  true   |  false   |   false   |   false   | true ✓
  ...

✓ All tests pass - simplified versions behave identically

🔍 QM Agent Analysis:
   - 71.4% reduction: 7 branches → 2 branches → 1 expression
   - 4 dead code branches detected
   - Key insight: isOwner and isAdmin alone grant access,
     making all combinations with them redundant
```

## How to Use QM Agent on These

To analyze any of these functions with the QM agent:

1. Extract the conditions and variables
2. Create a JSON request (see corresponding `.json` files in parent directory)
3. Run: `qm-agent simplify -i go_*.json`

The agent will:
- Detect dead code
- Show complexity reduction percentage
- Generate simplified code with original preserved as comments
- Flag overlapping conditions

## When to Use This Pattern

✅ **Refactoring legacy code** with accumulated conditionals
✅ **Code reviews** to find redundant checks
✅ **Security audits** to ensure coverage
✅ **Permission systems** with overlapping roles
✅ **State validation** with complex rules

## Corresponding JSON Files

Each `.go` file has a corresponding `.json` file in the parent directory:
- `feature_access.go` ↔ `go_feature_access.json`
- `api_validation.go` ↔ `go_api_validation.json`
- `document_access.go` ↔ `go_document_access.json`

These JSON files are what Claude would send to the QM agent for analysis.
