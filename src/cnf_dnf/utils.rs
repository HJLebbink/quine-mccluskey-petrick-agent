/// Convert CNF to string representation
pub fn cnf_to_string(cnf: &[u64]) -> String {
    to_string(cnf, true)
}

/// Convert DNF to string representation
pub fn dnf_to_string(dnf: &[u64]) -> String {
    to_string(dnf, false)
}

/// Convert CNF or DNF to string representation
fn to_string(terms: &[u64], is_cnf: bool) -> String {
    let mut terms_vec: Vec<u64> = terms.to_vec();
    terms_vec.sort_unstable();

    let mut result = String::new();
    let mut first_disj = true;

    for disj in terms_vec {
        if first_disj {
            first_disj = false;
        } else {
            result.push_str(if is_cnf { " & " } else { " | " });
        }

        result.push('(');
        let mut first_e = true;

        for i in 0..u64::BITS as usize {
            if test_bit(disj, i) {
                if first_e {
                    first_e = false;
                } else {
                    result.push_str(if is_cnf { "|" } else { "&" });
                }
                result.push_str(&i.to_string());
            }
        }

        result.push(')');
    }

    result
}

/// Test if a bit is set at a given position
#[inline]
pub fn test_bit(data: u64, pos: usize) -> bool {
    (data >> pos) & 1 == 1
}
