//! Lazy combination generation for picking k conditions from n.
//!
//! Uses a simple array-based counter that produces lexicographic order:
//! [0,1], [0,2], [0,3] → [1,2], [1,3] → [2,3].
//! Bit-twiddling bitmask optimization comes later once correctness is proven.

use std::mem::MaybeUninit;

/// Immutable state snapshot for a single combination query
#[derive(Clone, Copy, Debug)]
struct CombState {
    k: u32,
    n: u32,
    /// Bitmask of the *current* combination (indices set as bits)
    mask: u64,
}

/// Iterator producing one `[u32; 16]` snapshot per combination.
///
/// Each `[u32; 16]` contains indices sorted ascending in positions `0..k`
/// and zeros in positions `k..16`.
#[derive(Debug)]
pub struct CombinationIterator {
    /// The *previous* snapshot (the one we produced on the last `next()` call)
    state: CombState,
}

impl CombinationIterator {
    /// Create iterator over all k-element subsets of `{0..n}`.
    ///
    /// The first call to `next()` yields `{0, 1, …, k-1}`.
    /// Returns `None` if `k == 0` or `k > n` or `n > 64`.
    pub fn new(n: usize, k: usize) -> Option<Self> {
        let (n, k) = (n as u32, k as u32);
        if k == 0 || k > n || n > 64 {
            return None;
        }
        let mask = 0u64;  // start "before" first combination; next() produces {0,1,..} on first call
        Some(Self {
            state: CombState { n, k, mask },
        })
    }

    /// Extract the current indices into `out` (caller-allocated).
    /// Uses `ctz` on the mask for O(k) extraction.
    #[inline]
    pub fn indices(&self, out: &mut MaybeUninit<[u32; 16]>) {
        unsafe {
            let ptr = out.as_mut_ptr().cast::<u32>();
            let mut m = self.state.mask;
            let k = self.state.k;
            let mut idx = 0u32;
            while idx < k && m != 0 {
                let ctz = m.trailing_zeros();
                ptr.add(idx as usize).write(ctz);
                m &= m.wrapping_sub(1);
                idx += 1;
            }
            // Zero the rest
            while idx < 16 {
                ptr.add(idx as usize).write(0);
                idx += 1;
            }
        }
    }

    /// Advance `self.state` to the *next* combination (lexicographic).
    /// Returns true if a next combination exists, false if exhausted.
    ///
    /// This is the plain C++ `next_combination` port:
    ///   1. Find rightmost element that is not at its upper bound
    ///   2. Increment that element
    ///   3. Cascade all subsequent elements to the smallest values
    /// Advance to the next combination (lexicographic order).
    /// 
    /// The first call produces `{0, 1, …, k-1}`.
    /// The last successful call produces `{n-k, …, n-1}`; the following `next()` returns false.
    #[inline]
    pub fn next(&mut self) -> bool {
        let k = self.state.k as isize;
        if k == 0 {
            return false;
        }
        
        // First call: produce {0, 1, …, k-1}
        if self.state.mask == 0 {
            self.state.mask = if k == 64 { u64::MAX } else { (1u64 << k) - 1 };
            return true;
        }
        
        let n = self.state.n as isize;

        // Extract current indices
        let mut indices = [0u64; 16];
        {
            let mut m = self.state.mask;
            for j in 0..k {
                indices[j as usize] = m.trailing_zeros() as u64;
                m &= m.wrapping_sub(1);
            }
        }

        // Find rightmost index that can be incremented.
        // Upper bound for index i is: n - k + i (0-based)
        let mut i = k - 1;
        while i >= 0 {
            if indices[i as usize] < (n - k + i) as u64 {
                break;
            }
            i -= 1;
        }

        if i < 0 {
            return false; // exhausted
        }

        indices[i as usize] += 1;
        for j in ((i + 1) as usize)..(k as usize) {
            indices[j] = indices[(j - 1) as usize] + 1;
        }

        // Rebuild mask
        let mut new_mask = 0u64;
        for j in 0..k as usize {
            new_mask |= 1u64 << (indices[j] as u32);
        }
        self.state.mask = new_mask;
        true
    }

    /// Compute C(n, k).
    pub fn count(n: usize, k: usize) -> usize {
        choose(n, k)
    }
}

/// Compute binomial coefficient C(n, k)
#[inline]
pub fn choose(n: usize, k: usize) -> usize {
    if k > n { return 0; }
    if k == 0 || k == n { return 1; }
    if k == 1 || k == n - 1 { return n; }
    if k > n / 2 { return choose(n, n - k); }
    let mut r: u64 = 1;
    for i in 0..k {
        r = r * (n - i) as u64 / (i + 1) as u64;
    }
    r as usize
}

/// Collect all combinations into a vector (convenient, heap-allocates).
pub fn enumerate_all(n: usize, k: usize) -> Vec<[u32; 16]> {
    if k == 0 || k > n || n > 64 { return Vec::new(); }
    let mut result = Vec::with_capacity(choose(n, k));
    let mut iter = CombinationIterator::new(n, k).unwrap();
    let mut buf = MaybeUninit::<[u32; 16]>::uninit();
    loop {
        if !iter.next() { break; }
        iter.indices(&mut buf);
        result.push(unsafe { buf.assume_init() });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choose() {
        assert_eq!(choose(0, 0), 1);
        assert_eq!(choose(5, 2), 10);
        assert_eq!(choose(6, 3), 20);
        assert_eq!(choose(10, 5), 252);
        assert_eq!(choose(16, 8), 12870);
    }

    #[test]
    fn test_comb_4c2() {
        let mut iter = CombinationIterator::new(4, 2).unwrap();
        let mut buf = MaybeUninit::uninit();
        
        assert!(iter.next());
        iter.indices(&mut buf);
        assert_eq!(unsafe { buf.assume_init() }, [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        
        assert!(iter.next());
        iter.indices(&mut buf);
        assert_eq!(unsafe { buf.assume_init() }, [0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        
        assert!(iter.next());
        iter.indices(&mut buf);
        assert_eq!(unsafe { buf.assume_init() }, [0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        
        assert!(iter.next());
        iter.indices(&mut buf);
        assert_eq!(unsafe { buf.assume_init() }, [1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        
        assert!(iter.next());
        iter.indices(&mut buf);
        assert_eq!(unsafe { buf.assume_init() }, [1, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        
        assert!(iter.next());
        iter.indices(&mut buf);
        assert_eq!(unsafe { buf.assume_init() }, [2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        
        assert!(!iter.next());
    }

    #[test]
    fn test_comb_5c1() {
        let mut iter = CombinationIterator::new(5, 1).unwrap();
        let mut buf = MaybeUninit::uninit();
        for i in 0u32..5 {
            assert!(iter.next());
            iter.indices(&mut buf);
            assert_eq!(unsafe { buf.assume_init() }[0], i);
        }
        assert!(!iter.next());
    }

    #[test]
    fn test_count_8c3() {
        assert_eq!(CombinationIterator::count(8, 3), 56);
    }

    #[test]
    fn test_enumerate_all() {
        let c = enumerate_all(4, 2);
        assert_eq!(c.len(), 6);
        assert_eq!(c[0][0], 0); assert_eq!(c[0][1], 1);
        assert_eq!(c[5][0], 2); assert_eq!(c[5][1], 3);
    }

    #[test]
    fn test_boundary() {
        assert!(CombinationIterator::new(5, 0).is_none());
        assert!(CombinationIterator::new(5, 6).is_none());
        assert!(CombinationIterator::new(65, 1).is_none());

        // Exact boundary: n=64
        let c = enumerate_all(64, 1);
        assert_eq!(c.len(), 64);
    }
}
