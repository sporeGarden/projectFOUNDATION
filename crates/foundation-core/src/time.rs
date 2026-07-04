// SPDX-License-Identifier: AGPL-3.0-or-later
//! Minimal UTC time formatting — no external crate dependency.
//!
//! Implements the Howard Hinnant civil-date algorithm for converting
//! UNIX epoch seconds to `(year, month, day)` without any C library calls.

use std::time::SystemTime;

/// Civil date components (UTC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CivilDate {
    /// Year (4-digit).
    pub year: u64,
    /// Month (1-12).
    pub month: u64,
    /// Day of month (1-31).
    pub day: u64,
}

/// Civil datetime components (UTC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CivilDateTime {
    /// Date portion.
    pub date: CivilDate,
    /// Hour (0-23).
    pub hour: u64,
    /// Minute (0-59).
    pub minute: u64,
    /// Second (0-59).
    pub second: u64,
}

impl CivilDate {
    /// Compute civil date from a `SystemTime` (falls back to epoch on pre-1970 clocks).
    #[must_use]
    pub fn from_system_time(time: SystemTime) -> Self {
        let secs = time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self::from_epoch_secs(secs)
    }

    /// Compute civil date from seconds since UNIX epoch.
    #[must_use]
    pub const fn from_epoch_secs(secs: u64) -> Self {
        let days = secs / 86_400;
        let z = days + 719_468;
        let era = z / 146_097;
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };

        Self {
            year: y,
            month: m,
            day: d,
        }
    }

    /// Format as `YYYY-MM-DD`.
    #[must_use]
    pub fn to_iso_date(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl std::fmt::Display for CivilDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl CivilDateTime {
    /// Compute civil datetime from a `SystemTime`.
    #[must_use]
    pub fn from_system_time(time: SystemTime) -> Self {
        let secs = time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self::from_epoch_secs(secs)
    }

    /// Compute civil datetime from seconds since UNIX epoch.
    #[must_use]
    pub const fn from_epoch_secs(secs: u64) -> Self {
        let time_of_day = secs % 86_400;
        Self {
            date: CivilDate::from_epoch_secs(secs),
            hour: time_of_day / 3600,
            minute: (time_of_day % 3600) / 60,
            second: time_of_day % 60,
        }
    }

    /// Format as compact timestamp (`YYYYMMDD-HHMMSS`).
    #[must_use]
    pub fn to_compact(&self) -> String {
        format!(
            "{:04}{:02}{:02}-{:02}{:02}{:02}",
            self.date.year, self.date.month, self.date.day, self.hour, self.minute, self.second
        )
    }

    /// Format as display timestamp (`YYYY-MM-DD HH:MM:SS UTC`).
    #[must_use]
    pub fn to_display(&self) -> String {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
            self.date.year, self.date.month, self.date.day, self.hour, self.minute, self.second
        )
    }

    /// Format as ISO 8601 (`YYYY-MM-DDTHH:MM:SSZ`).
    #[must_use]
    pub fn to_iso(&self) -> String {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            self.date.year, self.date.month, self.date.day, self.hour, self.minute, self.second
        )
    }
}

impl std::fmt::Display for CivilDateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            self.date.year, self.date.month, self.date.day, self.hour, self.minute, self.second
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_1970_01_01() {
        let date = CivilDate::from_epoch_secs(0);
        assert_eq!(date.year, 1970);
        assert_eq!(date.month, 1);
        assert_eq!(date.day, 1);
    }

    #[test]
    fn known_date_2026_06_03() {
        // 2026-06-03 = day 20607 since epoch
        let secs = 20607 * 86_400;
        let date = CivilDate::from_epoch_secs(secs);
        assert_eq!(date.to_iso_date(), "2026-06-03");
    }

    #[test]
    fn datetime_formatting() {
        // 2026-06-03 14:30:45 UTC
        let secs = 20607 * 86_400 + 14 * 3600 + 30 * 60 + 45;
        let dt = CivilDateTime::from_epoch_secs(secs);
        assert_eq!(dt.to_compact(), "20260603-143045");
        assert_eq!(dt.to_display(), "2026-06-03 14:30:45 UTC");
        assert_eq!(dt.to_iso(), "2026-06-03T14:30:45Z");
    }

    #[test]
    fn display_trait_matches_iso() {
        let secs = 20607 * 86_400 + 3661;
        let dt = CivilDateTime::from_epoch_secs(secs);
        assert_eq!(dt.to_string(), dt.to_iso());
    }

    #[test]
    fn civil_date_display_matches_iso_date() {
        let date = CivilDate::from_epoch_secs(20607 * 86_400);
        assert_eq!(date.to_string(), date.to_iso_date());
    }
}
