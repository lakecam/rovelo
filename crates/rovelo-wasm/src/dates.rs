//! Tiny civil-date helpers (no date crate) for template due dates.
pub fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = i64::from(y) - i64::from(m <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (i64::from(m) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((y + i64::from(m <= 2)) as i32, m, d)
}

/// Parse strict `YYYY-MM-DD`.
pub fn parse(s: &str) -> Option<(i32, u32, u32)> {
    let b = s.trim().as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let n = |r: std::ops::Range<usize>| -> Option<u32> {
        let t = std::str::from_utf8(&b[r]).ok()?;
        t.bytes()
            .all(|c| c.is_ascii_digit())
            .then(|| t.parse().ok())
            .flatten()
    };
    let (y, m, d) = (n(0..4)? as i32, n(5..7)?, n(8..10)?);
    if !(1..=12).contains(&m) || d == 0 || d > 31 {
        return None;
    }
    // Round-trip rejects impossible dates such as 2025-02-30.
    (civil_from_days(days_from_civil(y, m, d)) == (y, m, d)).then_some((y, m, d))
}

/// `today` plus `n` days as ISO text.
pub fn plus_days(today: &str, n: i64) -> String {
    let (y, m, d) = parse(today).unwrap_or((2026, 1, 1));
    let (y, m, d) = civil_from_days(days_from_civil(y, m, d) + n);
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_days_across_months() {
        assert_eq!(plus_days("2026-11-28", 5), "2026-12-03");
        assert_eq!(plus_days("2028-02-28", 1), "2028-02-29");
        assert_eq!(plus_days("garbage", 0), "2026-01-01");
    }

    #[test]
    fn rejects_impossible() {
        assert!(parse("2025-02-30").is_none());
        assert!(parse("2026-11-02").is_some());
    }
}
