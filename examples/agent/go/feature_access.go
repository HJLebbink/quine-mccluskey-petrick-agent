package main

import "fmt"

// Original implementation with redundant conditions
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
		return true
	}
	return false
}

func main() {
	// Test all combinations
	testCases := []struct {
		premium, beta, admin bool
		expected             bool
	}{
		{false, false, false, false},
		{true, false, false, false}, // premium alone is not enough
		{false, true, false, false},
		{true, true, false, true},   // premium + beta
		{false, false, true, true},  // admin (sufficient alone)
		{true, false, true, true},   // premium + admin
		{false, true, true, true},   // beta + admin
		{true, true, true, true},    // all three
	}

	fmt.Println("Feature Access Control Test:")
	fmt.Println("Premium | Beta  | Admin | Access")
	fmt.Println("--------|-------|-------|-------")
	for _, tc := range testCases {
		result := canAccessFeature(tc.premium, tc.beta, tc.admin)
		if result != tc.expected {
			fmt.Printf("FAIL: %v | %v | %v | %v (expected %v)\n",
				tc.premium, tc.beta, tc.admin, result, tc.expected)
		} else {
			fmt.Printf("  %v   |  %v  |  %v  | %v\n",
				tc.premium, tc.beta, tc.admin, result)
		}
	}

	fmt.Println("\n🔍 Analysis: 4 branches can be simplified to 2")
	fmt.Println("   Key insight: 'isAdmin' alone grants access,")
	fmt.Println("   making combinations with isAdmin redundant")
}
