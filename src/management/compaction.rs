//! Allows to keep the number of backup files under control.

use std::fmt;

use super::backup::{Backup, BackupStatus};

/// Backups compaction strategy.
///
/// Determines if a backup can be kept (retainable) or deleted (disposable).
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

                let mut statuses = vec![];
                statuses.extend(
                    outdated_backups
                        .iter()
                        .map(|backup| (backup, BackupStatus::Disposable)),
                );
                statuses.extend(
                    recent_backups
                        .iter()
                        .map(|backup| (backup, BackupStatus::Retainable)),
                );

                Plan {
                    disposable: outdated_backups,
                    retainable: recent_backups,
                    statuses,
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
    /// List of backup files that should be deleted.
    pub disposable: &'a [Backup],

    /// List of backup files that should be kept.
    pub retainable: &'a [Backup],

    /// Sorted list of backup files along with their status (disposable/retainable).
    pub statuses: Vec<(&'a Backup, BackupStatus)>,
}
