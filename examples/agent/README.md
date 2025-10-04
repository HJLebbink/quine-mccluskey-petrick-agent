# Agent API Examples

This directory contains example JSON files for the Claude integration API.

## Overview

The QM agent can simplify if-then-else conditions from any programming language via a simple JSON API. Claude handles language parsing, and the agent provides boolean algebra optimization.

## 🌟 Real-World Examples (NEW!)

See **[REALWORLD_EXAMPLES.md](REALWORLD_EXAMPLES.md)** for dramatic examples showing 50-71% complexity reduction:

- **`go_feature_access.json`** - Feature access control (50% reduction)
- **`go_api_validation.json`** - API validation with dead code detection (20% reduction)
- **`go_document_access.json`** - Document permissions (71.4% reduction!)

These demonstrate the agent's power on realistic code patterns with overlapping conditions and redundant checks.

## Example Files

### 1. `simple.json`
Basic boolean simplification showing `(a && b) || (a && !b)` simplifying to just `a`.

**Run:**
```bash
qm-agent simplify -i examples/agent/simple.json
```

**Expected result:**
- Simplifies from 2 branches to 1
- Detects that both conditions lead to same output when `a` is true

### 2. `go_example.json`
Real-world Go access control logic with three boolean variables.

**Run:**
```bash
qm-agent simplify -i examples/agent/go_example.json
```

**Features:**
- Multiple boolean combinations
- Generates Go-style code suggestions
- Shows complexity reduction metrics

### 3. `integer_comparison.json`
Demonstrates integer variable support with status codes 0-7.

**Run:**
```bash
qm-agent simplify -i examples/agent/integer_comparison.json
```

**Shows:**
- Integer variables with bounded domains
- Comparison operators (==, >=)
- Range simplification

### 4. `dead_code.json`
Example with unreachable code for dead code detection.

**Run:**
```bash
qm-agent simplify -i examples/agent/dead_code.json
```

**Detects:**
- Branch 2 is fully covered by branch 1
- Provides warning with reason and covered_by information

## JSON Format

### Request Structure

```json
{
  "variables": {
    "var_name": "boolean",  // or
    "int_var": {"type": "integer", "min": 0, "max": 10}
  },
  "branches": [
    {
      "condition": "var_name && int_var > 5",
      "output": "result_value",
      "metadata": {
        "line": 42,
        "has_side_effects": false,
        "source": "original source code"
      }
    }
  ],
  "default": "default_value",
  "context": {
    "language": "go",  // "rust", "cpp", "python", etc.
    "preserve_order": false,
    "style": "guard_clauses"
  }
}
```

### Response Structure

```json
{
  "simplified_branches": [
    {
      "condition": "simplified_expr",
      "output": "result",
      "original_lines": [10, 12],
      "is_default": false
    }
  ],
  "analysis": {
    "dead_code": [
      {
        "branch_index": 1,
        "line": 12,
        "reason": "FullyCovered",
        "covered_by": [0]
      }
    ],
    "coverage_gaps": ["!a && !b", "!a && b"],
    "coverage_percent": 75.0,
    "overlaps": []
  },
  "suggestions": [
    {
      "kind": "simplification",
      "message": "Simplified from 3 to 2 branches (33.3% reduction)",
      "code": "if a {\n\treturn 1\n}\nreturn 0\n",
      "lines": [10, 12]
    }
  ],
  "metrics": {
    "original_branches": 3,
    "simplified_branches": 2,
    "complexity_reduction": 33.3,
    "variables_used": ["a", "b"]
  }
}
```

## Using with Claude

### Example Workflow

**User shows code to Claude:**
```go
func check(isValid, hasPermission, isAdmin bool) string {
    if isValid && hasPermission && isAdmin {
        return "full_access"
    } else if isValid && hasPermission && !isAdmin {
        return "read_access"
    } else if isValid && !hasPermission {
        return "no_access"
    }
    return "error"
}
```

**Claude extracts conditions and calls agent:**
```bash
echo '{"variables": {...}, "branches": [...]}' | qm-agent simplify
```

**Claude receives optimization and suggests:**
> "Your access control logic can be simplified. The conditions `hasPermission && isAdmin` and `hasPermission && !isAdmin` when both require `isValid` can be restructured..."

## Supported Features

- ✅ Boolean variables
- ✅ Integer variables with bounded domains (0-255)
- ✅ Comparison operators: ==, !=, <, >, <=, >=
- ✅ Boolean operators: &&, ||, !
- ✅ Dead code detection
- ✅ Coverage analysis
- ✅ Multi-language code generation
- ✅ Complexity metrics

## Limitations

- Maximum 16 variables (truth table size limit)
- Integer domains limited to reasonable ranges
- No function calls or complex expressions
- No variable-to-variable comparisons (only constant comparisons)

## Tips for Claude

1. **Extract pure conditions** - Filter out side effects first
2. **Infer variable domains** - Use type info to set min/max for integers
3. **Preserve order** - Set `preserve_order: true` if conditions have side effects
4. **Use metadata** - Include line numbers and source for better suggestions
5. **Choose language** - Set context.language for idiomatic code generation

## Running Tests

```bash
# Test all examples
for f in examples/agent/*.json; do
    echo "Testing $f"
    qm-agent simplify -i "$f"
done

# Test via stdin
cat examples/agent/simple.json | qm-agent simplify

# Integration with jq for filtering
qm-agent simplify -i examples/agent/simple.json | jq '.suggestions[0].code'
```
