//! Allows to keep the number of backup files under control.

use std::fmt;

use chrono::{Datelike, Timelike};
use chrono::{Duration, Local};
use itertools::Itertools;

use super::backup::{Backup, BackupStatus};

/// Backups compaction strategy.
///
/// Determines if a backup can be kept (retainable) or purged (purgeable).
#[derive(Debug, Clone)]
pub enum Strategy {
    /// Keep the `k` most recent backups.
    KeepMostRecent {
        /// Number of recent backup files to keep.
        k: usize,
    },

    /// Classic backup strategy.
    ///
    /// This is only useful if you save _very_ often, probably in an automated manner. See
    /// the method [`Strategy::plan`] for details.
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
    ///
    /// # KeepMostRecent strategy
    ///
    /// Simply splits the list of all backups into 2 lists: the `k` recent ones (or less if the
    /// catalog does not contain as much) and the remaining ones are considered outdated
    /// (purgeable).
    ///
    /// # Classic strategy
    ///
    /// Its goal is to keep
    ///
    /// - the lastest backup per hour for the past 24 hours (max 23 backups - exclude the past hour),
    /// - the lastest backup per day for the past 7 days (max 6 backups - exclude the past 24 hours),
    /// - the lastest backup per week of the past 4 weeks (max 3 backups - exclude the past week),
    /// - the lastest backup per month of this year (max 11 backups - exclude the past month).
    ///
    /// The time windows above are a partition; they do not overlap. Within each partition,
    /// only the most recent backup is kept.
    ///
    pub fn plan<'a>(&self, backups: &'a [Backup]) -> Plan<'a> {
        match self {
            Strategy::KeepMostRecent { k } => {
                let k = std::cmp::min(backups.len(), *k);
                let index = std::cmp::max(0, backups.len() - k);
                let (outdated_backups, recent_backups) = backups.split_at(index);

                let mut statuses = vec![];
                statuses.extend(
                    outdated_backups
                        .iter()
                        .map(|backup| (backup, BackupStatus::Purgeable)),
                );
                statuses.extend(
                    recent_backups
                        .iter()
                        .map(|backup| (backup, BackupStatus::Retainable)),
                );

                Plan {
                    purgeable: outdated_backups.iter().collect(),
                    retainable: recent_backups.iter().collect(),
                    statuses,
                }
            }

            Strategy::Classic => {
                let now = Local::now().naive_local();
                let _24h_ago = now - Duration::days(1);
                let _7d_ago = now - Duration::days(7);
                let _4w_ago = now - Duration::weeks(4);
                let _year_ago = now - Duration::days(365);

                // Last 24 h, grouped by hour
                let last_24h_per_hour: Vec<_> = backups
                    .iter()
                    .filter(|&b| b.creation_date > _24h_ago)
                    .group_by(|&b| b.creation_date.hour())
                    .into_iter()
                    .map(|(_key, group)| group.collect::<Vec<_>>())
                    .filter_map(|group| group.last().cloned())
                    .collect();

                // Last 7 days excluding the last 24 h, grouped by day
                let last_7d_per_day: Vec<_> = backups
                    .iter()
                    .filter(|&b| _24h_ago > b.creation_date && b.creation_date >= _7d_ago)
                    .group_by(|&b| b.creation_date.day())
                    .into_iter()
                    .map(|(_key, group)| group.collect::<Vec<_>>())
                    .filter_map(|group| group.last().cloned())
                    .collect();

                // Last 4 weeks excluding the last 7 days, grouped by week number
                let last_4w_per_isoweek: Vec<_> = backups
                    .iter()
                    .filter(|&b| _7d_ago > b.creation_date && b.creation_date >= _4w_ago)
                    .group_by(|&b| b.creation_date.iso_week())
                    .into_iter()
                    .map(|(_key, group)| group.collect::<Vec<_>>())
                    .filter_map(|group| group.last().cloned())
                    .collect();

                // Last year (365 days) excluding the last 4 weeks, grouped by month
                let last_year_per_month: Vec<_> = backups
                    .iter()
                    .filter(|&b| _4w_ago > b.creation_date && b.creation_date >= _year_ago)
                    .group_by(|&b| b.creation_date.month())
                    .into_iter()
                    .map(|(_key, group)| group.collect::<Vec<_>>())
                    .filter_map(|group| group.last().cloned())
                    .collect();

                let retainable: Vec<_> = vec![
                    last_year_per_month,
                    last_4w_per_isoweek,
                    last_7d_per_day,
                    last_24h_per_hour,
                ]
                .into_iter()
                .flatten()
                .collect();

                let retain_set: std::collections::HashSet<&Backup> =
                    retainable.iter().copied().collect();

                let purgeable: Vec<_> = backups
                    .iter()
                    .filter(|&b| !retain_set.contains(b))
                    .collect();

                let statuses: Vec<_> = backups
                    .iter()
                    .map(|b| {
                        if retain_set.contains(b) {
                            (b, BackupStatus::Retainable)
                        } else {
                            (b, BackupStatus::Purgeable)
                        }
                    })
                    .collect();

                Plan {
                    purgeable,
                    retainable,
                    statuses,
                }
            }
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
    /// List of backup files that should be purged.
    pub purgeable: Vec<&'a Backup>,

    /// List of backup files that should be kept.
    pub retainable: Vec<&'a Backup>,

    /// Sorted list of backup files along with their status (purgeable/retainable).
    pub statuses: Vec<(&'a Backup, BackupStatus)>,
}
