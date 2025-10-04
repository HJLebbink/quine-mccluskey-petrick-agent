package main

import "fmt"

// Complex SaaS feature access with 16 boolean flags
// Real-world scenario: Enterprise SaaS platform with complex feature gating
func canAccessAdvancedAnalytics(
	// Subscription tier flags
	isPremium, isEnterprise, isTrial bool,
	// Payment flags
	hasPaymentMethod, isPaymentVerified bool,
	// Verification flags
	isEmailVerified, isPhoneVerified bool,
	// Role flags
	isAdmin, isOwner, isModerator bool,
	// Feature flags
	hasAPIAccess, hasBulkExport bool,
	// Permission flags
	canInviteUsers, canCreateTeams bool,
	// Geographic flags
	isRegionEU, isRegionUS bool,
) bool {
	// Trial admin explicitly blocked (check first!)
	if isTrial && isAdmin {
		return false
	}

	// Original messy implementation with lots of redundancy
	if isEnterprise {
		return true
	}
	if isEnterprise && isEmailVerified {
		return true
	}
	if isEnterprise && isPaymentVerified {
		return true
	}
	if isEnterprise && hasPaymentMethod {
		return true
	}
	if isOwner && isPremium {
		return true
	}
	if isOwner && isEnterprise {
		return true
	}
	if isAdmin && isEnterprise {
		return true
	}
	if isAdmin && isPremium && isEmailVerified {
		return true
	}
	if isPremium && isEmailVerified && isPaymentVerified {
		return true
	}
	if isPremium && hasPaymentMethod && isEmailVerified {
		return true
	}
	if isOwner && isEmailVerified && hasPaymentMethod {
		return true
	}
	if isAdmin && hasBulkExport && isEnterprise {
		return true
	}
	if isPremium && canCreateTeams && isEmailVerified {
		return true
	}
	// Trial users have limited access
	if isTrial && isEmailVerified && isPhoneVerified && isRegionUS {
		return true
	}
	return false
}

// Simplified version after QM agent analysis
func canAccessAdvancedAnalyticsSimplified(
	isPremium, isEnterprise, isTrial bool,
	hasPaymentMethod, isPaymentVerified bool,
	isEmailVerified, isPhoneVerified bool,
	isAdmin, isOwner, isModerator bool,
	hasAPIAccess, hasBulkExport bool,
	canInviteUsers, canCreateTeams bool,
	isRegionEU, isRegionUS bool,
) bool {
	// Trial admin explicitly blocked (check first!)
	if isTrial && isAdmin {
		return false
	}

	// After QM analysis, we find:
	// 1. isEnterprise alone grants access (branches 2-4, 6-7, 11-12 redundant)
	// 2. isOwner + isPremium grants access
	// 3. Many conditions are subsumed by simpler ones

	// Enterprise access (any enterprise user except trial admins)
	if isEnterprise {
		return true
	}

	// Owner access (premium or with payment)
	if isOwner && (isPremium || hasPaymentMethod && isEmailVerified) {
		return true
	}

	// Admin access (premium with verification)
	if isAdmin && isPremium && isEmailVerified {
		return true
	}

	// Premium verified users
	if isPremium && isEmailVerified && (isPaymentVerified || hasPaymentMethod || canCreateTeams) {
		return true
	}

	// Trial users (very specific conditions, admins already blocked above)
	if isTrial && isEmailVerified && isPhoneVerified && isRegionUS {
		return true
	}

	return false
}

type TestCase struct {
	name string
	// Subscription
	isPremium, isEnterprise, isTrial bool
	// Payment
	hasPaymentMethod, isPaymentVerified bool
	// Verification
	isEmailVerified, isPhoneVerified bool
	// Roles
	isAdmin, isOwner, isModerator bool
	// Features
	hasAPIAccess, hasBulkExport bool
	// Permissions
	canInviteUsers, canCreateTeams bool
	// Region
	isRegionEU, isRegionUS bool
	// Expected result
	expected bool
}

func main() {
	testCases := []TestCase{
		{
			name:       "Enterprise user (should always have access)",
			isEnterprise: true,
			isEmailVerified: false, // Even without verification!
			expected:   true,
		},
		{
			name:            "Enterprise + Admin (redundant check)",
			isEnterprise:    true,
			isAdmin:         true,
			isEmailVerified: true,
			expected:        true,
		},
		{
			name:            "Owner + Premium",
			isOwner:         true,
			isPremium:       true,
			isEmailVerified: true,
			expected:        true,
		},
		{
			name:             "Premium + Verified + Payment",
			isPremium:        true,
			isEmailVerified:  true,
			isPaymentVerified: true,
			expected:         true,
		},
		{
			name:            "Admin + Premium + Verified",
			isAdmin:         true,
			isPremium:       true,
			isEmailVerified: true,
			expected:        true,
		},
		{
			name:            "Trial + Verified (US only)",
			isTrial:         true,
			isEmailVerified: true,
			isPhoneVerified: true,
			isRegionUS:      true,
			expected:        true,
		},
		{
			name:            "Trial + Verified (EU - should fail)",
			isTrial:         true,
			isEmailVerified: true,
			isPhoneVerified: true,
			isRegionEU:      true,
			expected:        false,
		},
		{
			name:            "Trial Admin (explicitly blocked)",
			isTrial:         true,
			isAdmin:         true,
			isEmailVerified: true,
			isPhoneVerified: true,
			isRegionUS:      true,
			expected:        false,
		},
		{
			name:     "Basic user (no access)",
			expected: false,
		},
		{
			name:            "Premium without verification",
			isPremium:       true,
			isEmailVerified: false,
			expected:        false,
		},
		{
			name:              "Owner + Enterprise (double redundancy)",
			isOwner:           true,
			isEnterprise:      true,
			isEmailVerified:   true,
			hasPaymentMethod:  true,
			expected:          true,
		},
		{
			name:             "Premium + Teams + Verified",
			isPremium:        true,
			canCreateTeams:   true,
			isEmailVerified:  true,
			expected:         true,
		},
	}

	fmt.Println("🎯 SaaS Feature Flags Analysis (16 Variables)")
	fmt.Println("============================================")
	fmt.Println("\nTesting: Advanced Analytics Access")
	fmt.Println("Variables: Enterprise, Premium, Trial, Payment, Verification, Roles, etc.\n")

	mismatches := 0
	for i, tc := range testCases {
		original := canAccessAdvancedAnalytics(
			tc.isPremium, tc.isEnterprise, tc.isTrial,
			tc.hasPaymentMethod, tc.isPaymentVerified,
			tc.isEmailVerified, tc.isPhoneVerified,
			tc.isAdmin, tc.isOwner, tc.isModerator,
			tc.hasAPIAccess, tc.hasBulkExport,
			tc.canInviteUsers, tc.canCreateTeams,
			tc.isRegionEU, tc.isRegionUS,
		)

		simplified := canAccessAdvancedAnalyticsSimplified(
			tc.isPremium, tc.isEnterprise, tc.isTrial,
			tc.hasPaymentMethod, tc.isPaymentVerified,
			tc.isEmailVerified, tc.isPhoneVerified,
			tc.isAdmin, tc.isOwner, tc.isModerator,
			tc.hasAPIAccess, tc.hasBulkExport,
			tc.canInviteUsers, tc.canCreateTeams,
			tc.isRegionEU, tc.isRegionUS,
		)

		status := "✓"
		if original != tc.expected || simplified != original {
			status = "✗ FAIL"
			mismatches++
		}

		fmt.Printf("%2d. %-45s Result: %-5v %s\n", i+1, tc.name, original, status)
	}

	if mismatches == 0 {
		fmt.Println("\n✅ All tests pass - simplified version behaves identically")
	} else {
		fmt.Printf("\n❌ %d test(s) failed\n", mismatches)
	}

	fmt.Println("\n🔍 QM Agent Analysis:")
	fmt.Println("   Original: 14 branches with heavy redundancy")
	fmt.Println("   Variables: 16 boolean flags (2^16 = 65,536 possible states)")
	fmt.Println("   ")
	fmt.Println("   Key findings:")
	fmt.Println("   1. isEnterprise alone grants access → 3 branches redundant")
	fmt.Println("   2. isOwner + isEnterprise redundant (isEnterprise alone sufficient)")
	fmt.Println("   3. Many isPremium combinations overlap")
	fmt.Println("   4. Trial logic requires ALL conditions (email, phone, US region)")
	fmt.Println("   ")
	fmt.Println("   Expected reduction: ~50-60% (14 branches → 6-7 branches)")
	fmt.Println("   ")
	fmt.Println("   Real-world impact:")
	fmt.Println("   - Easier to maintain and understand")
	fmt.Println("   - Fewer bugs from overlapping conditions")
	fmt.Println("   - Clear hierarchy: Enterprise > Owner > Admin > Premium > Trial")
	fmt.Println("   - Explicit trial admin blocking")
}
