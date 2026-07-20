//! System wall clock adapter (`wall_clock` domain, ADR-0005).
//!
//! Produces canonical RFC 3339 UTC timestamps
//! (`docs/standards/canonical-encoding-and-digest.md` section 6): uppercase
//! `T`/`Z`, no offset, millisecond fraction with trailing zeros trimmed and
//! zero fraction omitted. A reading the OS cannot supply (or that predates
//! the epoch) fails closed instead of guessing.

use cognitive_domain::WallTimestamp;
use cognitive_kernel::ports::{Clock, PortFailure};
use std::time::{SystemTime, UNIX_EPOCH};

/// OS wall clock producing canonical UTC timestamps.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

/// Days-to-civil-date conversion (Howard Hinnant's `civil_from_days`,
/// public-domain algorithm), exact for the proleptic Gregorian calendar.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let year_of_era = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if month <= 2 {
        year_of_era + 1
    } else {
        year_of_era
    };
    (year, month as u32, day as u32)
}

/// Format seconds + milliseconds since the Unix epoch as a canonical UTC
/// timestamp string.
pub(crate) fn format_canonical_utc(epoch_seconds: u64, millis: u32) -> String {
    let days = (epoch_seconds / 86_400) as i64;
    let seconds_of_day = epoch_seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    let mut out = format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}");
    if millis > 0 {
        let mut fraction = format!("{millis:03}");
        while fraction.ends_with('0') {
            fraction.pop();
        }
        out.push('.');
        out.push_str(&fraction);
    }
    out.push('Z');
    out
}

impl Clock for SystemClock {
    fn now(&self) -> Result<WallTimestamp, PortFailure> {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| PortFailure {
                detail: format!("system clock before unix epoch: {err}"),
            })?;
        let text = format_canonical_utc(elapsed.as_secs(), elapsed.subsec_millis());
        WallTimestamp::parse(&text).map_err(|err| PortFailure {
            detail: format!("system clock produced non-canonical timestamp: {err}"),
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn known_instants_format_exactly() {
        assert_eq!(format_canonical_utc(0, 0), "1970-01-01T00:00:00Z");
        assert_eq!(format_canonical_utc(951_782_400, 0), "2000-02-29T00:00:00Z");
        // 2026-07-20T05:00:00Z == 1784523600 seconds since the epoch.
        assert_eq!(
            format_canonical_utc(1_784_523_600, 0),
            "2026-07-20T05:00:00Z"
        );
        assert_eq!(
            format_canonical_utc(1_784_523_600, 120),
            "2026-07-20T05:00:00.12Z",
            "trailing zeros trimmed"
        );
        assert_eq!(
            format_canonical_utc(1_784_523_600, 7),
            "2026-07-20T05:00:00.007Z"
        );
    }

    #[test]
    fn readings_are_canonical_form() {
        let now = SystemClock.now().unwrap();
        // WallTimestamp::parse already enforces the canonical form; assert
        // the reading is in a plausible year range as a sanity check.
        assert!(now.as_str().starts_with("20"), "{now}");
    }

    #[test]
    fn sampled_days_through_2101_round_trip_through_validation() {
        // Stride a prime number of days from 1970 through two century
        // boundaries (2000 leap, 2100 non-leap) to hit varied offsets.
        let mut days = 0u64;
        while days < 48_300 {
            let text = format_canonical_utc(days * 86_400 + 3_661, 999);
            assert!(
                cognitive_domain::WallTimestamp::parse(&text).is_ok(),
                "{text}"
            );
            days += 97;
        }
    }
}
