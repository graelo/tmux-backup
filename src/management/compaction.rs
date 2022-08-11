//! Compaction allows to keep the number of backup files under control.

use std::fmt;

use super::backup::Backup;

/// Backups compaction strategy.
///
/// Determines if a backup should be kept or deleted.
#[derive(Debug, Clone)]
pub enum Strategy {
    /// Keep the `k` most recent backups.
    KeepMostRecent {
        /// Number of recent backup files to keep.
        k: usize,
    },

    /// Classic backup strategy.
    ///
    /// This keeps
    /// - the latest backup in the hour,
    /// - the latest backup of the previous day,
    /// - the latest backup of the previous week,
    /// - the latest backup of the previous month,
    Classic,
}

impl Strategy {
    /// Return a new simple strategy.
    pub fn most_recent(k: usize) -> Self {
        Self::KeepMostRecent { k }
    }

    /// Determine which backup files should be kept.
    ///
    /// The `backup_files` are assumed to be sorted from oldest to newest.
    pub fn plan<'a>(&self, backup_files: &'a [Backup]) -> Plan<'a> {
        match self {
            Strategy::KeepMostRecent { k } => {
                let index = std::cmp::max(0, backup_files.len() - k);
                let (outdated_backups, recent_backups) = backup_files.split_at(index);

                Plan {
                    disposable: outdated_backups,
                    retainable: recent_backups,
                }
            }

            Strategy::Classic => unimplemented!(),
        }
    }
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Strategy::KeepMostRecent { k } => {
                write!(f, "KeepMostRecent: {}", k)
            }
            Strategy::Classic => write!(f, "Classic"),
        }
    }
}

/// Describes what the strategy would do.
pub struct Plan<'a> {
    /// List of backup files to delete.
    pub disposable: &'a [Backup],

    /// List of backup files to keep.
    pub retainable: &'a [Backup],
}
