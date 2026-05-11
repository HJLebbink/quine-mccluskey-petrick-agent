// FIX: Trace OR(3) to find why "1" appears 3 times
use qm_agent::qm::encoding::BitOps;

fn decode(raw: u32, vars: usize) -> String {
    let dc_mask = raw >> vars;
    let data = raw & ((1u32 << vars) - 1);

    let mut result = String::new();
    for i in (0..vars).rev() {
        if dc_mask.get_bit(i) {
            result.push('X');
        } else if data.get_bit(i) {
            result.push('1');
        } else {
            result.push('0');
        }
    }
    result
}

fn main() {
    println!("=== Correct decode function ===\n");

    // All strings produced by 3-variable raw encodings
    let mut by_string: std::collections::HashMap<String, Vec<u32>> =
        std::collections::HashMap::new();

    for raw in 0..1024 {
        let s = decode(raw, 3);
        by_string.entry(s).or_default().push(raw);
    }

    // Find collisions
    println!("Collision groups (same string from multiple raw encodings):");
    let mut found = 0;
    for (s, raws) in &by_string {
        if raws.len() > 1 {
            found += 1;
            if found <= 30 {
                println!("  \"{}\" ← {:?}", s, raws);
            }
        }
    }
    println!("\n  Total collision groups: {}", found);

    // Show raw→string mapping for all don't-care encodings
    println!("\n=== All encodings that produce 'XX0', '1X0', etc. ===");
    for (s, raws) in &by_string {
        if s.chars().filter(|&c| c == 'X').count() > 0 {
            println!("  \"{}\" → raws: {:?}", s, raws);
        }
    }

    // Show raw=0, raw=8, raw=16, raw=24 etc. - all with dc=1 for different bit positions
    println!("\n=== Key raw encodings ===");
    for raw in [
        0, 8, 16, 24, 56, 120, 504, 1008, 1016, 1020, 1022, 1023, 1024, 1028, 1032, 1040, 1056,
        1088, 1152, 504, 632,
    ] {
        if raw < 1024 {
            let s = decode(raw, 3);
            let dc = raw >> 3;
            let data = raw & 7;
            println!(
                "  raw={:>4} ({:010b}) → data={:03b} dc={:03b} → \"{}\"",
                raw, raw, data, dc, s
            );
        }
    }
}
