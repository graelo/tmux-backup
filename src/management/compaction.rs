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
                    .chunk_by(|&b| b.creation_date.hour())
                    .into_iter()
                    .map(|(_key, group)| group.collect::<Vec<_>>())
                    .filter_map(|group| group.last().cloned())
                    .collect();

                // Last 7 days excluding the last 24 h, grouped by day
                let last_7d_per_day: Vec<_> = backups
                    .iter()
                    .filter(|&b| _24h_ago > b.creation_date && b.creation_date >= _7d_ago)
                    .chunk_by(|&b| b.creation_date.day())
                    .into_iter()
                    .map(|(_key, group)| group.collect::<Vec<_>>())
                    .filter_map(|group| group.last().cloned())
                    .collect();

                // Last 4 weeks excluding the last 7 days, grouped by week number
                let last_4w_per_isoweek: Vec<_> = backups
                    .iter()
                    .filter(|&b| _7d_ago > b.creation_date && b.creation_date >= _4w_ago)
                    .chunk_by(|&b| b.creation_date.iso_week())
                    .into_iter()
                    .map(|(_key, group)| group.collect::<Vec<_>>())
                    .filter_map(|group| group.last().cloned())
                    .collect();

                // Last year (365 days) excluding the last 4 weeks, grouped by month
                let last_year_per_month: Vec<_> = backups
                    .iter()
                    .filter(|&b| _4w_ago > b.creation_date && b.creation_date >= _year_ago)
                    .chunk_by(|&b| b.creation_date.month())
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
                write!(f, "KeepMostRecent: {k}")
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::path::PathBuf;

    /// Create a backup at the given date/time. The path encodes the datetime for easy debugging.
    fn backup_at(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> Backup {
        let dt = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, min, sec)
            .unwrap();
        Backup {
            filepath: PathBuf::from(format!(
                "/backups/backup-{}.tar.zst",
                dt.format("%Y%m%dT%H%M%S")
            )),
            creation_date: dt,
        }
    }

    /// Generate a sequence of backups, one per hour, starting from a base datetime.
    fn generate_hourly_backups(count: usize) -> Vec<Backup> {
        (0..count)
            .map(|i| {
                let hour = i % 24;
                let day = 1 + (i / 24);
                backup_at(2024, 6, day as u32, hour as u32, 0, 0)
            })
            .collect()
    }

    mod keep_most_recent_strategy {
        use super::*;

        #[test]
        fn empty_catalog_produces_empty_plan() {
            let strategy = Strategy::most_recent(5);
            let backups: Vec<Backup> = vec![];

            let plan = strategy.plan(&backups);

            assert!(plan.purgeable.is_empty());
            assert!(plan.retainable.is_empty());
            assert!(plan.statuses.is_empty());
        }

        #[test]
        fn single_backup_when_k_is_one() {
            let strategy = Strategy::most_recent(1);
            let backups = vec![backup_at(2024, 6, 15, 10, 0, 0)];

            let plan = strategy.plan(&backups);

            assert!(plan.purgeable.is_empty());
            assert_eq!(plan.retainable.len(), 1);
        }

        #[test]
        fn single_backup_when_k_exceeds_count() {
            let strategy = Strategy::most_recent(10);
            let backups = vec![backup_at(2024, 6, 15, 10, 0, 0)];

            let plan = strategy.plan(&backups);

            // Should keep the one backup we have, not fail
            assert!(plan.purgeable.is_empty());
            assert_eq!(plan.retainable.len(), 1);
        }

        #[test]
        fn keeps_exactly_k_most_recent() {
            let strategy = Strategy::most_recent(3);
            let backups = vec![
                backup_at(2024, 6, 15, 8, 0, 0),  // oldest - purgeable
                backup_at(2024, 6, 15, 9, 0, 0),  // purgeable
                backup_at(2024, 6, 15, 10, 0, 0), // retainable
                backup_at(2024, 6, 15, 11, 0, 0), // retainable
                backup_at(2024, 6, 15, 12, 0, 0), // newest - retainable
            ];

            let plan = strategy.plan(&backups);

            assert_eq!(plan.purgeable.len(), 2);
            assert_eq!(plan.retainable.len(), 3);

            // The oldest two should be purgeable
            assert_eq!(plan.purgeable[0].creation_date.hour(), 8);
            assert_eq!(plan.purgeable[1].creation_date.hour(), 9);

            // The newest three should be retainable
            assert_eq!(plan.retainable[0].creation_date.hour(), 10);
            assert_eq!(plan.retainable[1].creation_date.hour(), 11);
            assert_eq!(plan.retainable[2].creation_date.hour(), 12);
        }

        #[test]
        fn statuses_preserve_original_order() {
            let strategy = Strategy::most_recent(2);
            let backups = vec![
                backup_at(2024, 6, 15, 8, 0, 0),
                backup_at(2024, 6, 15, 9, 0, 0),
                backup_at(2024, 6, 15, 10, 0, 0),
                backup_at(2024, 6, 15, 11, 0, 0),
            ];

            let plan = strategy.plan(&backups);

            // Statuses should be in the same order as input
            assert_eq!(plan.statuses.len(), 4);
            assert!(matches!(plan.statuses[0].1, BackupStatus::Purgeable));
            assert!(matches!(plan.statuses[1].1, BackupStatus::Purgeable));
            assert!(matches!(plan.statuses[2].1, BackupStatus::Retainable));
            assert!(matches!(plan.statuses[3].1, BackupStatus::Retainable));
        }

        #[test]
        fn k_equals_count_keeps_all() {
            let strategy = Strategy::most_recent(3);
            let backups = vec![
                backup_at(2024, 6, 15, 8, 0, 0),
                backup_at(2024, 6, 15, 9, 0, 0),
                backup_at(2024, 6, 15, 10, 0, 0),
            ];

            let plan = strategy.plan(&backups);

            assert!(plan.purgeable.is_empty());
            assert_eq!(plan.retainable.len(), 3);
        }

        #[test]
        fn k_zero_purges_all() {
            let strategy = Strategy::most_recent(0);
            let backups = vec![
                backup_at(2024, 6, 15, 8, 0, 0),
                backup_at(2024, 6, 15, 9, 0, 0),
            ];

            let plan = strategy.plan(&backups);

            assert_eq!(plan.purgeable.len(), 2);
            assert!(plan.retainable.is_empty());
        }

        #[test]
        fn handles_large_catalog() {
            let strategy = Strategy::most_recent(10);
            let backups = generate_hourly_backups(100);

            let plan = strategy.plan(&backups);

            assert_eq!(plan.purgeable.len(), 90);
            assert_eq!(plan.retainable.len(), 10);

            // Verify the retained ones are the most recent
            for retained in &plan.retainable {
                // The last 10 backups (indices 90-99)
                assert!(backups[90..].contains(retained));
            }
        }
    }

    mod strategy_display {
        use super::*;

        #[test]
        fn keep_most_recent_shows_count() {
            let strategy = Strategy::most_recent(42);
            assert_eq!(format!("{strategy}"), "KeepMostRecent: 42");
        }

        #[test]
        fn classic_shows_name() {
            let strategy = Strategy::Classic;
            assert_eq!(format!("{strategy}"), "Classic");
        }
    }

    mod strategy_constructors {
        use super::*;

        #[test]
        fn most_recent_stores_k() {
            let strategy = Strategy::most_recent(7);
            match strategy {
                Strategy::KeepMostRecent { k } => assert_eq!(k, 7),
                _ => panic!("Expected KeepMostRecent variant"),
            }
        }
    }

    // Note: The Classic strategy uses `Local::now()` internally, making it
    // non-deterministic and difficult to unit test reliably. To properly test
    // Classic, consider refactoring `plan()` to accept a `now` parameter,
    // or create an integration test with a controlled time environment.
    //
    // The Classic strategy logic groups backups by:
    // - Hour (last 24h)
    // - Day (last 7 days, excluding last 24h)
    // - Week (last 4 weeks, excluding last 7 days)
    // - Month (last year, excluding last 4 weeks)
    //
    // Each group keeps only the most recent backup within that time window.
}
