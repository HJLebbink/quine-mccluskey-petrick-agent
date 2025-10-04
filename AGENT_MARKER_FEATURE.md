# QM Agent Marker Feature

## Summary

The QM agent now includes a **marker system** to prevent re-analysis of already-optimized code. This feature saves computation time and respects previous optimization decisions.

## Key Features

### 1. Original Code Preservation

When the agent simplifies code, it preserves the original with comments:

```go
// QM-AGENT-ORIGINAL:
// func check(flag bool) int {
//     if flag {
//         return 1
//     }
//     if !flag {
//         return 1
//     }
//     return 0
// }

// QM-AGENT-SIMPLIFIED
if flag || !flag {
    return 1
}
```

### 2. Skip Re-Analysis

If code already contains QM-AGENT markers, Claude can skip re-analysis by setting `already_analyzed: true`:

**Request:**
```json
{
  "variables": {"flag": "boolean"},
  "branches": [{"condition": "flag", "output": "return 1"}],
  "context": {
    "already_analyzed": true
  }
}
```

**Response:**
```json
{
  "suggestions": [{
    "kind": "already_analyzed",
    "message": "Code was already analyzed by QM agent. Skipping re-analysis."
  }],
  "metrics": {
    "complexity_reduction": 0.0
  }
}
```

### 3. Conditional Code Generation

The agent now only generates code suggestions when there's **actual complexity reduction**:

- **complexity_reduction > 0.0**: Include code suggestion with original preserved
- **complexity_reduction == 0.0**: Return "No simplification possible. The logic is already optimal."
- **already_analyzed == true**: Skip analysis entirely

## API Changes

### New Request Fields

Added to `context` in JSON request:

```typescript
{
  "context": {
    "language": "go",                           // existing
    "preserve_order": false,                    // existing
    "already_analyzed": false,                  // NEW - skip re-analysis
    "original_code": "func original() { ... }"  // NEW - preserve as comments
  }
}
```

### New Response Types

New suggestion kinds:

1. `"no_change"` - When complexity_reduction == 0.0 and no issues found
2. `"already_analyzed"` - When already_analyzed flag is set

## Usage Examples

### Example 1: First-Time Analysis with Simplification

```bash
# Input JSON
{
  "variables": {"flag": "boolean"},
  "branches": [
    {"condition": "flag", "output": "return 1"},
    {"condition": "!flag", "output": "return 1"}
  ],
  "default": "return 0",
  "context": {
    "language": "go",
    "original_code": "func check(flag bool) int {\n\tif flag {\n\t\treturn 1\n\t}\n\tif !flag {\n\t\treturn 1\n\t}\n\treturn 0\n}"
  }
}
```

**Result:**
- 50% complexity reduction (2 branches → 1)
- Code suggestion includes commented original
- Markers added: `QM-AGENT-ORIGINAL` and `QM-AGENT-SIMPLIFIED`

### Example 2: Re-Analysis Check

```bash
# Claude detects markers in source code
# Sets already_analyzed: true

{
  "context": {
    "already_analyzed": true
  }
}
```

**Result:**
- Analysis skipped
- "Code was already analyzed" message
- No suggestions generated

### Example 3: Already Optimal Code

```bash
# Input: Simple code with no optimization possible
{
  "variables": {"flag": "boolean"},
  "branches": [
    {"condition": "flag", "output": "return true"}
  ],
  "default": "return false"
}
```

**Result:**
- complexity_reduction: 0.0
- Suggestion: "No simplification possible. The logic is already optimal."
- No code generated (no changes needed)

## Implementation Details

### File: `src/agent_api.rs`

1. **RequestContext** (line 113-135):
   - Added `already_analyzed: bool`
   - Added `original_code: Option<String>`

2. **process_request()** (line 246-270):
   - Early return if `already_analyzed == true`
   - Returns stub response with "already_analyzed" suggestion

3. **generate_suggestions()** (line 417-489):
   - Changed threshold from `> 5.0` to `> 0.0`
   - Added "no_change" suggestion when no improvements found
   - Passes `original_code` to code generator

4. **generate_code()** (line 492-514):
   - New signature: `fn generate_code(..., original_code: Option<&str>)`
   - Prepends commented original if provided
   - Adds marker comments

5. **comment_out_code()** (line 516-528):
   - Helper function to comment code by language
   - Python: `# ` prefix
   - C-style (Go, Rust, C++): `// ` prefix

### File: `CLAUDE.md`

Added comprehensive documentation:
- Section: "Avoiding Re-Analysis of Already-Optimized Code" (line 486-564)
- Updated "Tips for Claude" with marker checking (line 587-590)
- Updated JSON API documentation (line 343-344)

## Testing

Created test files:
- `examples/agent/test_already_analyzed.json` - Tests skip behavior
- `examples/agent/test_with_original.json` - Tests original_code field
- `examples/agent/test_true_simplification.json` - Tests code generation

All existing tests pass (8/8 passed, 1 ignored).

## Benefits

1. **Performance**: Skip redundant analysis on already-optimized code
2. **Reversibility**: Original code preserved as comments, easy to revert
3. **Clarity**: Clear markers indicate code was analyzed
4. **Efficiency**: Only suggest changes when actual improvement exists

## Workflow for Claude

1. Read source code
2. Check for `QM-AGENT-ORIGINAL` or `QM-AGENT-SIMPLIFIED` markers
3. If markers found:
   - Set `already_analyzed: true`
   - Skip calling agent (or call with flag set)
   - Inform user code was already analyzed
4. If no markers:
   - Extract conditions
   - Include `original_code` in request
   - Call agent
   - Apply suggestions if complexity_reduction > 0

## Language Support

Marker comments automatically use correct syntax:
- **Go, Rust, C++, C**: `// comment`
- **Python**: `# comment`
- **Generic**: `// comment` (default)
