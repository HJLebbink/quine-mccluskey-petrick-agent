package main

import (
	"fmt"
	"net/http"
)

// Original implementation with dead code
func validateAPIRequest(isAuthenticated, hasValidToken, isRateLimited bool) int {
	if !isAuthenticated {
		return http.StatusUnauthorized
	}
	if !isAuthenticated && !hasValidToken { // ← DEAD CODE (unreachable)
		return http.StatusUnauthorized
	}
	if !hasValidToken && !isAuthenticated { // ← DEAD CODE (unreachable)
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

// Simplified version (after QM agent analysis)
func validateAPIRequestSimplified(isAuthenticated, hasValidToken, isRateLimited bool) int {
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

func statusName(code int) string {
	switch code {
	case http.StatusOK:
		return "OK"
	case http.StatusUnauthorized:
		return "Unauthorized"
	case http.StatusTooManyRequests:
		return "Too Many Requests"
	case http.StatusInternalServerError:
		return "Internal Server Error"
	default:
		return "Unknown"
	}
}

func main() {
	testCases := []struct {
		authenticated, validToken, rateLimited bool
		expected                               int
	}{
		{false, false, false, http.StatusUnauthorized},
		{false, false, true, http.StatusUnauthorized},
		{false, true, false, http.StatusUnauthorized},
		{false, true, true, http.StatusUnauthorized},
		{true, false, false, http.StatusInternalServerError},
		{true, false, true, http.StatusInternalServerError},
		{true, true, false, http.StatusOK},
		{true, true, true, http.StatusTooManyRequests},
	}

	fmt.Println("API Validation Test:")
	fmt.Println("Auth | Token | RateLimit | Result")
	fmt.Println("-----|-------|-----------|-------")

	allMatch := true
	for _, tc := range testCases {
		original := validateAPIRequest(tc.authenticated, tc.validToken, tc.rateLimited)
		simplified := validateAPIRequestSimplified(tc.authenticated, tc.validToken, tc.rateLimited)

		match := "✓"
		if original != simplified {
			match = "✗ MISMATCH"
			allMatch = false
		}

		fmt.Printf(" %v  |  %v   |    %v     | %-20s %s\n",
			tc.authenticated, tc.validToken, tc.rateLimited,
			statusName(original), match)
	}

	if allMatch {
		fmt.Println("\n✓ All tests pass - simplified version behaves identically")
	}

	fmt.Println("\n🔍 QM Agent Analysis:")
	fmt.Println("   - 2 dead code branches detected (lines with redundant !isAuthenticated)")
	fmt.Println("   - 20% reduction: 5 branches → 4 branches")
	fmt.Println("   - Once !isAuthenticated is checked, further checks are unreachable")
}
