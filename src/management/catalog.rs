//! Catalog of all backups.

use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use anyhow::Result;
use async_std::fs;
use async_std::stream::StreamExt;
use chrono::{Duration, Local, NaiveDateTime};
use regex::Regex;

use crate::config::SubList;
use crate::management::archive::Metadata;

use super::compaction::{Plan, Strategy};

/// Useful backup metadata.
pub struct Backup {
    /// Path to the backup file.
    pub filepath: PathBuf,

    /// Backup date.
    pub creation_date: NaiveDateTime,
}

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
    /// - The catalog only manages backup files such as `backup-20220804T221153.tar.zst`.
    pub async fn new<P: AsRef<Path>>(dirpath: P, strategy: Strategy) -> Result<Catalog> {
        fs::create_dir_all(dirpath.as_ref()).await?;

        let backup_files = Self::read_files(dirpath.as_ref()).await?;

        let catalog = Catalog {
            dirpath: dirpath.as_ref().to_path_buf(),
            strategy,
            backups: backup_files,
        };

        Ok(catalog)
    }

    /// Update the catalog's list of backups with current content of `dirpath`.
    pub async fn refresh_mut(&mut self) -> Result<()> {
        self.backups = Self::read_files(self.dirpath.as_path()).await?;
        Ok(())
    }

    /// Total number of backups in the catalog.
    pub fn size(&self) -> usize {
        self.backups.len()
    }

    /// Return the catalog's dirpath as a string, for convenience in error messages.
    pub fn location(&self) -> String {
        self.dirpath.to_string_lossy().into()
    }

    /// Filepath of the current backup.
    ///
    /// This is usually the most recent backup.
    pub fn current(&self) -> Option<&Backup> {
        self.backups.last()
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
    pub fn list(&self, sublist: Option<SubList>) {
        let Plan {
            disposable,
            retainable,
        } = self.plan();

        if let Some(sublist) = sublist {
            match sublist {
                SubList::Disposable => {
                    for backup in disposable.iter() {
                        println!("{}", backup.filepath.to_string_lossy());
                    }
                }
                SubList::Retainable => {
                    for backup in retainable.iter() {
                        println!("{}", backup.filepath.to_string_lossy());
                    }
                }
            }
        } else {
            println!("Strategy: {}", &self.strategy);
            println!("Location: `{}`\n", self.location());

            let reset = "\u{001b}[0m";
            let yellow = "\u{001b}[33m";
            let green = "\u{001b}[32m";

            let n_retainable = retainable.len();
            let n_disposable = disposable.len();

            let now = Local::now().naive_local();

            println!("{:4} {:32} {:24} {:6}", "", "NAME", "CREATED", "STATUS");

            let iter = RangeInclusive::new(n_retainable + 1, n_retainable + n_disposable)
                .into_iter()
                .rev();
            for (index, backup) in std::iter::zip(iter, disposable) {
                let filename = backup.filepath.file_name().unwrap().to_string_lossy();
                println!(
                    "{:3}. {yellow}{:32}{reset} {:24} {:6}",
                    index,
                    filename,
                    Self::time_ago(now, backup.creation_date),
                    "disposable",
                );
            }

            let iter = RangeInclusive::new(1, n_retainable).into_iter().rev();
            for (index, backup) in std::iter::zip(iter, retainable) {
                let filename = backup.filepath.file_name().unwrap().to_string_lossy();
                println!(
                    "{:3}. {green}{:32}{reset} {:24} {:6}",
                    index,
                    filename,
                    Self::time_ago(now, backup.creation_date),
                    "retainable",
                );
            }

            println!(
                "\n{} backups: {} retainable, {} disposable",
                self.size(),
                retainable.len(),
                disposable.len(),
            );
        }
    }

    pub fn describe<P>(&self, backup_filepath: P)
    where
        P: AsRef<Path>,
    {
        match Metadata::read(backup_filepath) {
            Ok(metadata) => {
                let report = metadata.get_report();
                println!("{report}");
            }
            Err(e) => eprintln!("{}", e),
        }
    }
}

// Private functions

impl Catalog {
    /// Return the list of `Backup` in `dirpath`.
    async fn read_files<P: AsRef<Path>>(dirpath: P) -> Result<Vec<Backup>> {
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
}
