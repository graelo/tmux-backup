//! Compaction allows to keep the number of backup files under control.

use std::path::PathBuf;

/// Backups compaction strategy.
///
/// Determines if a backup should be kept or deleted.
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
    pub fn plan<'a>(&self, backup_files: &'a [PathBuf]) -> Plan<'a> {
        match self {
            Strategy::KeepMostRecent { k } => {
                // let backup_files = backup_files.map(|p| *p).collect();
                let index = std::cmp::max(0, backup_files.len() - k);
                let (outdated_backups, recent_backups) = backup_files.split_at(index);

                Plan {
                    to_remove: outdated_backups,
                    to_keep: recent_backups,
                }
            }

            Strategy::Classic => Plan {
                to_remove: backup_files,
                to_keep: backup_files,
            },
        }
    }
}

/// Describes what the strategy would do.
pub struct Plan<'a> {
    /// List of backup files to delete.
    pub to_remove: &'a [PathBuf],

    /// List of backup files to keep.
    pub to_keep: &'a [PathBuf],
}
