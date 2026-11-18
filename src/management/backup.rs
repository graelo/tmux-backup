//! High-level representation of a backup for catalog operations and reporting.

use std::fmt;
use std::path::PathBuf;

use clap::ValueEnum;
use jiff::civil::DateTime;

/// Quick access, high-level representation of a backup.
///
/// `Backup` provides only information which can be derived from the file name, avoiding to open
/// the file, deal with the format, parse the metadata, etc.
///
/// This is sufficient for the [`Catalog`](crate::management::catalog::Catalog) to list backups
/// and decide whether or not a backup should be deleted or kept.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Backup {
    /// Path to the backup file.
    pub filepath: PathBuf,

    /// Backup date.
    pub creation_date: DateTime,
}

impl Backup {
    /// Return a string representing the duration since the backup file was created.
    ///
    // This function can only receive properly formatted files
    pub fn age(&self, now: DateTime) -> String {
        format_age(self.creation_date, now)
    }
}

/// The single rolling autosave archive, tracked separately from ordinary backups.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Autosave {
    /// Path to the autosave archive.
    pub filepath: PathBuf,

    /// Last modification time of the autosave archive.
    pub modified_at: DateTime,
}

impl Autosave {
    /// Return a string representing the duration since the autosave was last written.
    pub fn age(&self, now: DateTime) -> String {
        format_age(self.modified_at, now)
    }
}

fn format_age(creation_date: DateTime, now: DateTime) -> String {
    let duration_secs = now.duration_since(creation_date).as_secs();

    // Month scale -> "n months ago"
    let month = 4 * WEEK_SECS;
    if duration_secs >= 2 * month {
        return format!("{} months", duration_secs / month);
    }
    if duration_secs >= month {
        return "1 month".into();
    }

    // Week scale -> "n weeks ago"
    let week = WEEK_SECS;
    if duration_secs >= 2 * week {
        return format!("{} weeks", duration_secs / week);
    }
    if duration_secs >= week {
        return "1 week".into();
    }

    // Day scale -> "n days ago"
    let day = DAY_SECS;
    if duration_secs >= 2 * day {
        return format!("{} days", duration_secs / day);
    }
    if duration_secs >= day {
        return "1 day".into();
    }

    // Hour scale -> "n hours ago"
    let hour = HOUR_SECS;
    if duration_secs >= 2 * hour {
        return format!("{} hours", duration_secs / hour);
    }
    if duration_secs >= hour {
        return "1 hour".into();
    }

    // Minute scale -> "n minutes ago"
    let minute = MINUTE_SECS;
    if duration_secs >= 2 * minute {
        return format!("{} minutes", duration_secs / minute);
    }
    if duration_secs >= minute {
        return "1 minute".into();
    }

    format!("{duration_secs} seconds")
}

/// Number of seconds in one minute.
const MINUTE_SECS: i64 = 60;

/// Number of seconds in one hour.
const HOUR_SECS: i64 = 60 * MINUTE_SECS;

/// Number of seconds in one day.
const DAY_SECS: i64 = 24 * HOUR_SECS;

/// Number of seconds in one week.
const WEEK_SECS: i64 = 7 * DAY_SECS;

/// Which subset of backups to print.
#[derive(Debug, Clone, ValueEnum)]
pub enum BackupStatus {
    /// Retainable backups only.
    Retainable,

    /// Purgeable backups only.
    Purgeable,
}

impl fmt::Display for BackupStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackupStatus::Retainable => write!(f, "{:12}", "retainable"),
            BackupStatus::Purgeable => write!(f, "{:12}", "purgeable"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    /// Helper to create a backup with a given datetime for testing.
    fn backup_at(year: i16, month: i8, day: i8, hour: i8, min: i8, sec: i8) -> Backup {
        Backup {
            filepath: PathBuf::from("/tmp/backup.tar.zst"),
            creation_date: datetime(year, month, day, hour, min, sec),
        }
    }

    fn datetime(year: i16, month: i8, day: i8, hour: i8, min: i8, sec: i8) -> DateTime {
        date(year, month, day).at(hour, min, sec, 0)
    }

    mod age_formatting {
        use super::*;

        #[test]
        fn fresh_backup_shows_seconds() {
            let backup = backup_at(2024, 6, 15, 10, 30, 0);
            let now = datetime(2024, 6, 15, 10, 30, 45);

            assert_eq!(backup.age(now), "45 seconds");
        }

        #[test]
        fn long_spans_keep_whole_second_truncation() {
            // 112 days (four 28-day "months") minus one nanosecond: a floating-point total
            // would round this up to the next whole second and report 4 months.
            let backup = backup_at(2024, 1, 1, 0, 0, 0);
            let now = date(2024, 4, 21).at(23, 59, 59, 999_999_999);

            assert_eq!(backup.age(now), "3 months");
        }

        #[test]
        fn subsecond_durations_are_truncated_to_whole_seconds() {
            let backup = backup_at(2024, 6, 15, 10, 30, 0);
            let now = date(2024, 6, 15).at(10, 30, 59, 900_000_000);

            assert_eq!(backup.age(now), "59 seconds");
        }

        #[test]
        fn zero_seconds_is_still_seconds() {
            let backup = backup_at(2024, 6, 15, 10, 30, 0);
            let now = datetime(2024, 6, 15, 10, 30, 0);

            assert_eq!(backup.age(now), "0 seconds");
        }

        #[test]
        fn exactly_one_minute() {
            let backup = backup_at(2024, 6, 15, 10, 30, 0);
            let now = datetime(2024, 6, 15, 10, 31, 0);

            assert_eq!(backup.age(now), "1 minute");
        }

        #[test]
        fn just_under_two_minutes_is_still_one_minute() {
            let backup = backup_at(2024, 6, 15, 10, 30, 0);
            let now = datetime(2024, 6, 15, 10, 31, 59);

            assert_eq!(backup.age(now), "1 minute");
        }

        #[test]
        fn two_minutes_uses_plural() {
            let backup = backup_at(2024, 6, 15, 10, 30, 0);
            let now = datetime(2024, 6, 15, 10, 32, 0);

            assert_eq!(backup.age(now), "2 minutes");
        }

        #[test]
        fn fifty_nine_minutes_before_hour_threshold() {
            let backup = backup_at(2024, 6, 15, 10, 0, 0);
            let now = datetime(2024, 6, 15, 10, 59, 59);

            assert_eq!(backup.age(now), "59 minutes");
        }

        #[test]
        fn exactly_one_hour() {
            let backup = backup_at(2024, 6, 15, 10, 0, 0);
            let now = datetime(2024, 6, 15, 11, 0, 0);

            assert_eq!(backup.age(now), "1 hour");
        }

        #[test]
        fn just_under_two_hours_is_still_one_hour() {
            let backup = backup_at(2024, 6, 15, 10, 0, 0);
            let now = datetime(2024, 6, 15, 11, 59, 59);

            assert_eq!(backup.age(now), "1 hour");
        }

        #[test]
        fn two_hours_uses_plural() {
            let backup = backup_at(2024, 6, 15, 10, 0, 0);
            let now = datetime(2024, 6, 15, 12, 0, 0);

            assert_eq!(backup.age(now), "2 hours");
        }

        #[test]
        fn twenty_three_hours_before_day_threshold() {
            let backup = backup_at(2024, 6, 15, 0, 0, 0);
            let now = datetime(2024, 6, 15, 23, 59, 59);

            assert_eq!(backup.age(now), "23 hours");
        }

        #[test]
        fn exactly_one_day() {
            let backup = backup_at(2024, 6, 15, 10, 0, 0);
            let now = datetime(2024, 6, 16, 10, 0, 0);

            assert_eq!(backup.age(now), "1 day");
        }

        #[test]
        fn six_days_before_week_threshold() {
            let backup = backup_at(2024, 6, 15, 10, 0, 0);
            let now = datetime(2024, 6, 21, 9, 59, 59);

            assert_eq!(backup.age(now), "5 days");
        }

        #[test]
        fn exactly_one_week() {
            let backup = backup_at(2024, 6, 15, 10, 0, 0);
            let now = datetime(2024, 6, 22, 10, 0, 0);

            assert_eq!(backup.age(now), "1 week");
        }

        #[test]
        fn two_weeks() {
            let backup = backup_at(2024, 6, 1, 10, 0, 0);
            let now = datetime(2024, 6, 15, 10, 0, 0);

            assert_eq!(backup.age(now), "2 weeks");
        }

        #[test]
        fn three_weeks_exactly() {
            let backup = backup_at(2024, 6, 1, 10, 0, 0);
            let now = datetime(2024, 6, 22, 10, 0, 0); // exactly 21 days = 3 weeks

            assert_eq!(backup.age(now), "3 weeks");
        }

        #[test]
        fn just_under_four_weeks_still_shows_weeks() {
            let backup = backup_at(2024, 6, 1, 10, 0, 0);
            let now = datetime(2024, 6, 29, 9, 59, 59); // just under 28 days

            assert_eq!(backup.age(now), "3 weeks");
        }

        #[test]
        fn exactly_one_month_four_weeks() {
            let backup = backup_at(2024, 6, 1, 10, 0, 0);
            let now = datetime(2024, 6, 29, 10, 0, 0);

            // 4 weeks = 28 days, which is the "month" threshold in this implementation
            assert_eq!(backup.age(now), "1 month");
        }

        #[test]
        fn two_months() {
            let backup = backup_at(2024, 1, 1, 10, 0, 0);
            let now = datetime(2024, 3, 1, 10, 0, 0);

            // ~60 days = 2 "months" (8 weeks)
            assert_eq!(backup.age(now), "2 months");
        }

        #[test]
        fn many_months_ago() {
            let backup = backup_at(2024, 1, 1, 0, 0, 0);
            let now = datetime(2024, 12, 1, 0, 0, 0);

            // ~11 months = roughly 48 weeks
            let age = backup.age(now);
            assert!(age.ends_with("months"), "Expected months, got: {age}");
        }
    }

    mod backup_status_display {
        use super::*;

        #[test]
        fn retainable_is_padded_to_12_chars() {
            let status = BackupStatus::Retainable;
            assert_eq!(format!("{status}"), "retainable  ");
        }

        #[test]
        fn purgeable_is_padded_to_12_chars() {
            let status = BackupStatus::Purgeable;
            assert_eq!(format!("{status}"), "purgeable   ");
        }
    }

    mod backup_equality {
        use super::*;

        #[test]
        fn same_path_and_date_are_equal() {
            let a = backup_at(2024, 6, 15, 10, 30, 0);
            let b = backup_at(2024, 6, 15, 10, 30, 0);

            assert_eq!(a, b);
        }

        #[test]
        fn different_dates_are_not_equal() {
            let a = backup_at(2024, 6, 15, 10, 30, 0);
            let b = backup_at(2024, 6, 15, 10, 30, 1);

            assert_ne!(a, b);
        }

        #[test]
        fn different_paths_are_not_equal() {
            let a = Backup {
                filepath: PathBuf::from("/tmp/a.tar.zst"),
                creation_date: datetime(2024, 6, 15, 10, 30, 0),
            };
            let b = Backup {
                filepath: PathBuf::from("/tmp/b.tar.zst"),
                creation_date: datetime(2024, 6, 15, 10, 30, 0),
            };

            assert_ne!(a, b);
        }
    }
}
