package main

import "fmt"

// Original implementation with massive redundancy
func canAccessDocument(isOwner, isAdmin, isEditor, isPublic bool) bool {
	if isOwner {
		return true
	}
	if isAdmin {
		return true
	}
	if isOwner && isEditor { // ← DEAD CODE (isOwner alone grants access)
		return true
	}
	if isAdmin && isEditor { // ← DEAD CODE (isAdmin alone grants access)
		return true
	}
	if isPublic && isEditor {
		return true
	}
	if isOwner && isPublic { // ← DEAD CODE (isOwner alone grants access)
		return true
	}
	if isAdmin && isPublic { // ← DEAD CODE (isAdmin alone grants access)
		return true
	}
	return false
}

// Simplified version (after QM agent analysis)
func canAccessDocumentSimplified(isOwner, isAdmin, isEditor, isPublic bool) bool {
	// Dead code removed: isOwner and isAdmin alone grant access
	// The combinations with them are redundant
	if isOwner || isAdmin {
		return true
	}
	if isPublic && isEditor {
		return true
	}
	return false
}

// Alternative simplified form (guard clause style)
func canAccessDocumentGuards(isOwner, isAdmin, isEditor, isPublic bool) bool {
	if isOwner {
		return true
	}
	if isAdmin {
		return true
	}
	if isPublic && isEditor {
		return true
	}
	return false
}

func main() {
	// Test all 16 combinations
	testCases := []struct {
		owner, admin, editor, public bool
		expected                      bool
	}{
		{false, false, false, false, false},
		{true, false, false, false, true},  // isOwner
		{false, true, false, false, true},  // isAdmin
		{true, true, false, false, true},
		{false, false, true, false, false}, // isEditor alone not enough
		{true, false, true, false, true},   // isOwner
		{false, true, true, false, true},   // isAdmin
		{true, true, true, false, true},
		{false, false, false, true, false}, // isPublic alone not enough
		{true, false, false, true, true},   // isOwner
		{false, true, false, true, true},   // isAdmin
		{true, true, false, true, true},
		{false, false, true, true, true},   // isPublic && isEditor
		{true, false, true, true, true},    // isOwner
		{false, true, true, true, true},    // isAdmin
		{true, true, true, true, true},
	}

	fmt.Println("Document Access Control Test:")
	fmt.Println("Owner | Admin | Editor | Public | Access")
	fmt.Println("------|-------|--------|--------|-------")

	allMatch := true
	for _, tc := range testCases {
		original := canAccessDocument(tc.owner, tc.admin, tc.editor, tc.public)
		simplified := canAccessDocumentSimplified(tc.owner, tc.admin, tc.editor, tc.public)
		guards := canAccessDocumentGuards(tc.owner, tc.admin, tc.editor, tc.public)

		match := "✓"
		if original != simplified || original != guards {
			match = "✗ MISMATCH"
			allMatch = false
		}

		fmt.Printf("  %v   |  %v   |   %v   |   %v   | %v %s\n",
			tc.owner, tc.admin, tc.editor, tc.public, original, match)
	}

	if allMatch {
		fmt.Println("\n✓ All tests pass - simplified versions behave identically")
	}

	fmt.Println("\n🔍 QM Agent Analysis:")
	fmt.Println("   - 71.4% reduction: 7 branches → 2 branches")
	fmt.Println("   - 4 dead code branches detected")
	fmt.Println("   - Key insight: isOwner and isAdmin alone grant access,")
	fmt.Println("     making all combinations with them redundant")
	fmt.Println("\n📊 Comparison:")
	fmt.Println("   Original:   7 branches, 4 are dead code")
	fmt.Println("   Simplified: if isOwner || isAdmin { return true }")
	fmt.Println("              if isPublic && isEditor { return true }")
	fmt.Println("              return false")
	fmt.Println("   Savings:    3 branches instead of 7, much clearer logic")
}
