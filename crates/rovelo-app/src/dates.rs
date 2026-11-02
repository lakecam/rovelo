//! Civil dates without a date crate: today from the system clock, ISO parsing, overdue test.
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub y: i32,
    pub m: u32,
    pub d: u32,
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.y, self.m, self.d)
    }
}

/// Days since 1970-01-01 for a proleptic Gregorian date.
pub fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = i64::from(y) - i64::from(m <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (i64::from(m) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn civil_from_days(z: i64) -> Date {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    Date {
        y: (y + i64::from(m <= 2)) as i32,
        m,
        d,
    }
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
    }
}

impl Date {
    pub fn days(self) -> i64 {
        days_from_civil(self.y, self.m, self.d)
    }

    pub fn plus_days(self, n: i64) -> Date {
        civil_from_days(self.days() + n)
    }

    /// Strict `YYYY-MM-DD`; rejects impossible dates such as 2025-02-30.
    pub fn parse(s: &str) -> Option<Date> {
        let b = s.trim().as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return None;
        }
        let num = |r: std::ops::Range<usize>| -> Option<u32> {
            let t = std::str::from_utf8(&b[r]).ok()?;
            if t.bytes().all(|c| c.is_ascii_digit()) {
                t.parse().ok()
            } else {
                None
            }
        };
        let (y, m, d) = (num(0..4)? as i32, num(5..7)?, num(8..10)?);
        if !(1..=12).contains(&m) || d == 0 || d > days_in_month(y, m) {
            return None;
        }
        Some(Date { y, m, d })
    }
}

pub fn today() -> Date {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    civil_from_days(secs.div_euclid(86_400))
}

/// A task is overdue when it is open and its due date is before `today`.
pub fn is_overdue(due: Option<&str>, done: bool, today: Date) -> bool {
    !done && due.and_then(Date::parse).is_some_and(|d| d < today)
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

impl Date {
    /// "Nov 7"; the year is added only when it is not `today`'s.
    pub fn short(self, today: Date) -> String {
        let m = MONTHS[(self.m as usize).clamp(1, 12) - 1];
        if self.y == today.y {
            format!("{m} {}", self.d)
        } else {
            format!("{m} {}, {}", self.d, self.y)
        }
    }
}

/// "today", "tomorrow", "in 3 d", "2 d late".
pub fn relative(due: Date, today: Date) -> String {
    match due.days() - today.days() {
        0 => "today".into(),
        1 => "tomorrow".into(),
        n if n > 1 => format!("in {n} d"),
        n => format!("{} d late", -n),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_and_known_dates() {
        assert_eq!(
            civil_from_days(0),
            Date {
                y: 1970,
                m: 1,
                d: 1
            }
        );
        assert_eq!(days_from_civil(2000, 3, 1), 11_017);
        assert_eq!(
            civil_from_days(19_723),
            Date {
                y: 2024,
                m: 1,
                d: 1
            }
        );
        assert_eq!(
            civil_from_days(days_from_civil(2024, 2, 29)),
            Date {
                y: 2024,
                m: 2,
                d: 29
            }
        );
    }

    #[test]
    fn round_trips_across_centuries() {
        for z in (-800_000..800_000).step_by(997) {
            let d = civil_from_days(z);
            assert_eq!(days_from_civil(d.y, d.m, d.d), z);
        }
    }

    #[test]
    fn parse_is_strict() {
        assert!(Date::parse("2026-11-02").is_some());
        assert!(Date::parse("2024-02-29").is_some());
        assert!(Date::parse("2025-02-29").is_none());
        assert!(Date::parse("2026-13-01").is_none());
        assert!(Date::parse("2026-1-1").is_none());
        assert!(Date::parse("2026/11/02").is_none());
        assert!(Date::parse("").is_none());
        assert!(Date::parse("+026-11-02").is_none());
    }

    #[test]
    fn overdue_rules() {
        let t = Date {
            y: 2026,
            m: 11,
            d: 2,
        };
        assert!(is_overdue(Some("2026-11-01"), false, t));
        assert!(!is_overdue(Some("2026-11-02"), false, t));
        assert!(!is_overdue(Some("2026-11-01"), true, t));
        assert!(!is_overdue(Some("garbage"), false, t));
        assert!(!is_overdue(None, false, t));
    }

    #[test]
    fn plus_days_and_relative() {
        let t = Date {
            y: 2026,
            m: 12,
            d: 30,
        };
        assert_eq!(
            t.plus_days(3),
            Date {
                y: 2027,
                m: 1,
                d: 2
            }
        );
        assert_eq!(relative(t.plus_days(1), t), "tomorrow");
        assert_eq!(relative(t.plus_days(-2), t), "2 d late");
        assert_eq!(t.to_string(), "2026-12-30");
    }

    #[test]
    fn short_form() {
        let t = Date {
            y: 2026,
            m: 11,
            d: 2,
        };
        assert_eq!(
            Date {
                y: 2026,
                m: 11,
                d: 7
            }
            .short(t),
            "Nov 7"
        );
        assert_eq!(
            Date {
                y: 2027,
                m: 1,
                d: 3
            }
            .short(t),
            "Jan 3, 2027"
        );
    }

    #[test]
    fn today_is_plausible() {
        assert!(today().y >= 2024);
    }
}
