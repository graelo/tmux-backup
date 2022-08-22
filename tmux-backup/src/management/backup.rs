//! High-level representation of a backup for catalog operations and reporting.

use std::fmt;
use std::path::PathBuf;

use chrono::{Duration, NaiveDateTime};
use clap::ValueEnum;

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
    pub creation_date: NaiveDateTime,
}

impl Backup {
    /// Return a string representing the duration since the backup file was created.
    ///
    // This function can only receive properly formatted files
    pub fn age(&self, now: NaiveDateTime) -> String {
        let duration = now.signed_duration_since(self.creation_date);
        let duration_secs = duration.num_seconds();

        // Month scale -> "n months ago"
        let month = Duration::weeks(4).num_seconds();
        if duration_secs >= 2 * month {
            return format!("{} months", duration_secs / month);
        }
        if duration_secs >= month {
            return "1 month".into();
        }

        // Week scale -> "n weeks ago"
        let week = Duration::weeks(1).num_seconds();
        if duration_secs >= 2 * week {
            return format!("{} weeks", duration_secs / week);
        }
        if duration_secs >= week {
            return "1 week".into();
        }

        // Day scale -> "n days ago"
        let day = Duration::days(1).num_seconds();
        if duration_secs >= 2 * day {
            return format!("{} days", duration_secs / day);
        }
        if duration_secs >= day {
            return "1 day".into();
        }

        // Hour scale -> "n hours ago"
        let hour = Duration::hours(1).num_seconds();
        if duration_secs >= 2 * hour {
            return format!("{} hours", duration_secs / hour);
        }
        if duration_secs >= hour {
            return "1 hour".into();
        }

        // Minute scale -> "n minutes ago"
        let minute = Duration::minutes(1).num_seconds();
        if duration_secs >= 2 * minute {
            return format!("{} minutes", duration_secs / minute);
        }
        if duration_secs >= minute {
            return "1 minute".into();
        }

        format!("{} seconds", duration_secs)
    }
}

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
