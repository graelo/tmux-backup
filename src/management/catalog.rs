//! Catalog of all backups.

use std::iter;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use anyhow::Result;
use async_std::stream::StreamExt;
use async_std::{fs, task};
use chrono::{Duration, Local, NaiveDateTime};
use futures::future::join_all;
use regex::Regex;

use crate::management::{
    archive::v1,
    backup::{Backup, BackupStatus},
    compaction::{Plan, Strategy},
};

/// Catalog of all backups.
pub struct Catalog {
    /// Location of the catalog.
    pub dirpath: PathBuf,

    /// Compaction strategy.
    pub strategy: Strategy,

    /// Sorted list of all backups (oldest to newest).
    pub backups: Vec<Backup>,
}

// Public API

impl Catalog {
    /// Return a new `Catalog` by listing the backups in `dirpath`.
    ///
    /// # Notes
    ///
    /// - The folder is created if missing.
    /// - The catalog only manages backup files such as `backup-20220804T221153.tar.zst`, other
    /// files are simply ignored (and in principle, should not be present).
    pub async fn new<P: AsRef<Path>>(dirpath: P, strategy: Strategy) -> Result<Catalog> {
        fs::create_dir_all(dirpath.as_ref()).await?;

        let backup_files = Self::parse_backup_filenames(dirpath.as_ref()).await?;

        let catalog = Catalog {
            dirpath: dirpath.as_ref().to_path_buf(),
            strategy,
            backups: backup_files,
        };

        Ok(catalog)
    }

    /// Update the catalog's list of backups with current content of `dirpath`.
    pub async fn refresh(self) -> Result<Catalog> {
        let backups = Self::parse_backup_filenames(self.dirpath.as_path()).await?;
        Ok(Catalog {
            dirpath: self.dirpath,
            strategy: self.strategy,
            backups,
        })
    }

    /// Update the catalog's list of backups with current content of `dirpath`.
    pub async fn refresh_mut(&mut self) -> Result<()> {
        self.backups = Self::parse_backup_filenames(self.dirpath.as_path()).await?;
        Ok(())
    }

    /// Total number of backups in the catalog.
    pub fn size(&self) -> usize {
        self.backups.len()
    }

    /// Filepath of the current backup.
    ///
    /// This is usually the most recent backup.
    pub fn current(&self) -> Option<&Backup> {
        match self.strategy {
            Strategy::KeepMostRecent { .. } => self.backups.last(),
            Strategy::Classic => unimplemented!(),
        }
    }

    /// Simulate the compaction strategy: list the backup files to delete, and the ones to keep.
    pub fn plan(&self) -> Plan {
        self.strategy.plan(&self.backups)
    }

    /// Apply the compaction strategy.
    ///
    /// # Important
    ///
    /// This will probably delete files in the `dirpath` folder.
    pub async fn compact(&self) -> Result<usize> {
        let Plan {
            disposable,
            retainable: _retainable,
            ..
        } = self.plan();

        let n = disposable.len();
        for backup in disposable {
            fs::remove_file(&backup.filepath).await?;
        }

        Ok(n)
    }

    /// Apply the compaction strategy and update the catalog.
    ///
    /// # Important
    ///
    /// This will probably delete files in the `dirpath` folder.
    pub async fn compact_mut(&mut self) {
        self.compact()
            .await
            .expect("Error when compacting the catalog");
        self.refresh_mut()
            .await
            .expect("Error when refreshing the catalog");
    }

    /// List backups.
    ///
    /// If a specific backup status is passed (`status` is `Some(..)`), then the function prints
    /// only the absolute paths of the corresponding backups, otherwise it prints a
    /// Docker/Podman-like table.
    ///
    /// If `details_flag` is `true`, the function prints an overview of the content of the
    /// backup:
    ///
    /// - version of the archive's format
    /// - number of sessions
    /// - number of windows
    /// - number of panes
    ///
    /// but this is slower because it needs to read partially each backup file.
    pub async fn list(&self, status: Option<BackupStatus>, details_flag: bool) {
        if let Some(status) = status {
            match status {
                BackupStatus::Disposable => {
                    let Plan { disposable, .. } = self.plan();
                    for backup in disposable.iter() {
                        println!("{}", backup.filepath.to_string_lossy());
                    }
                }
                BackupStatus::Retainable => {
                    let Plan { retainable, .. } = self.plan();
                    for backup in retainable.iter() {
                        println!("{}", backup.filepath.to_string_lossy());
                    }
                }
            }
        } else {
            self.full_list(details_flag).await;
        }
    }
}

// Private functions

impl Catalog {
    /// Return the list of `Backup` in `dirpath`.
    async fn parse_backup_filenames<P: AsRef<Path>>(dirpath: P) -> Result<Vec<Backup>> {
        let mut backups: Vec<Backup> = vec![];

        let pattern = r#".*backup-(\d{8}T\d{6})\.tar\.zst"#;
        let matcher = Regex::new(pattern).unwrap();

        let mut entries = fs::read_dir(dirpath.as_ref()).await?;
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let path = entry.path();
            if let Some(captures) = matcher.captures(&path.to_string_lossy()) {
                let date_str = &captures[1];
                let creation_date =
                    NaiveDateTime::parse_from_str(date_str, "%Y%m%dT%H%M%S").unwrap();
                let backup = Backup {
                    filepath: path.into(),
                    creation_date,
                };
                backups.push(backup);
            }
        }

        backups.sort_unstable_by_key(|b| b.creation_date);

        Ok(backups)
    }

    /// Return a string representing the duration since the backup file was created.
    ///
    // This function can only receive properly formatted files
    fn time_ago(now: NaiveDateTime, creation_date: NaiveDateTime) -> String {
        let duration = now.signed_duration_since(creation_date);
        let duration_secs = duration.num_seconds();

        // Month scale -> "n months ago"
        let month = Duration::weeks(4).num_seconds();
        if duration_secs >= 2 * month {
            return format!("{} months ago", duration_secs / month);
        }
        if duration_secs >= month {
            return "1 month ago".into();
        }

        // Week scale -> "n weeks ago"
        let week = Duration::weeks(1).num_seconds();
        if duration_secs >= 2 * week {
            return format!("{} weeks ago", duration_secs / week);
        }
        if duration_secs >= week {
            return "1 week ago".into();
        }

        // Day scale -> "n days ago"
        let day = Duration::days(1).num_seconds();
        if duration_secs >= 2 * day {
            return format!("{} days ago", duration_secs / day);
        }
        if duration_secs >= day {
            return "1 day ago".into();
        }

        // Hour scale -> "n hours ago"
        let hour = Duration::hours(1).num_seconds();
        if duration_secs >= 2 * hour {
            return format!("{} hours ago", duration_secs / hour);
        }
        if duration_secs >= hour {
            return "1 hour ago".into();
        }

        // Minute scale -> "n minutes ago"
        let minute = Duration::minutes(1).num_seconds();
        if duration_secs >= 2 * minute {
            return format!("{} minutes ago", duration_secs / minute);
        }
        if duration_secs >= minute {
            return "1 minute ago".into();
        }

        return format!("{} seconds ago", duration_secs);
    }

    async fn full_list(&self, details_flag: bool) {
        println!("Strategy: {}", &self.strategy);
        println!("Location: `{}`\n", self.dirpath.to_string_lossy());

        let Plan {
            disposable,
            retainable,
            statuses,
        } = self.plan();

        let now = Local::now().naive_local();

        let reset = "\u{001b}[0m";
        let green = "\u{001b}[32m";
        let yellow = "\u{001b}[33m";

        // 45, 44, ..., 1
        let indices = RangeInclusive::new(1, statuses.len()).into_iter().rev();

        if details_flag {
            // Table header
            println!(
                "{:4} {:32} {:17} {:12} {:8} {:8}",
                "", "NAME", "CREATED", "STATUS", "VERSION", "CONTENT"
            );

            // Read all metadata concurrently
            let tasks: Vec<_> = statuses
                .iter()
                .map(|&(backup, _)| {
                    let backup_filepath = backup.filepath.clone();
                    task::spawn(async move { v1::read_metadata(backup_filepath).await })
                })
                .collect();
            let metadatas: Result<Vec<_>, _> = join_all(tasks).await.into_iter().collect();
            let metadatas = metadatas.expect("Cannot read metadata files");

            // Build & print table rows
            for (index, ((backup, status), metadata)) in
                iter::zip(indices, iter::zip(statuses, metadatas))
            {
                let filename = backup.filepath.file_name().unwrap().to_string_lossy();
                let color = match status {
                    BackupStatus::Disposable => yellow,
                    BackupStatus::Retainable => green,
                };
                let status_str = match status {
                    BackupStatus::Disposable => "disposable",
                    BackupStatus::Retainable => "retainable",
                };
                let time_ago = Self::time_ago(now, backup.creation_date);

                let overview = metadata.overview();
                let version = &metadata.version;

                println!(
                        "{index:3}. {color}{filename:32}{reset} {time_ago:17} {color}{status_str:12}{reset} {version:8} {overview:8}"
                    );
            }
        } else {
            // Table header
            println!("{:4} {:32} {:17} {:6}", "", "NAME", "CREATED", "STATUS");

            // Build & print table rows
            for (index, (backup, status)) in iter::zip(indices, statuses) {
                let filename = backup.filepath.file_name().unwrap().to_string_lossy();
                let color = match status {
                    BackupStatus::Disposable => yellow,
                    BackupStatus::Retainable => green,
                };
                let status_str = match status {
                    BackupStatus::Disposable => "disposable",
                    BackupStatus::Retainable => "retainable",
                };
                let time_ago = Self::time_ago(now, backup.creation_date);

                println!(
                        "{index:3}. {color}{filename:32}{reset} {time_ago:17} {color}{status_str:6}{reset}"
                    );
            }
        }

        println!(
            "\n{} backups: {} retainable, {} disposable",
            self.size(),
            retainable.len(),
            disposable.len(),
        );
    }
}
