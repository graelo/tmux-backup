//! Catalog of all backups.

use std::borrow::Cow;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::{env, iter};

use async_std::stream::StreamExt;
use async_std::{fs, task};
use chrono::{Local, NaiveDateTime};
use futures::future::join_all;
use regex::Regex;
use si_scale::helpers::bytes2;

use crate::{
    management::{
        archive::v1,
        backup::{Backup, BackupStatus},
        compaction::{Plan, Strategy},
    },
    Result,
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
        let dirpath = dirpath.as_ref();
        fs::create_dir_all(dirpath).await?;

        let backup_files = Self::parse_backup_filenames(dirpath).await?;

        let catalog = Catalog {
            dirpath: dirpath.to_path_buf(),
            strategy,
            backups: backup_files,
        };

        Ok(catalog)
    }

    /// Update the catalog's list of backups with the current content of `dirpath`.
    ///
    /// This returns a new catalog with the updated content.
    pub async fn refresh(self) -> Result<Catalog> {
        let backups = Self::parse_backup_filenames(self.dirpath.as_path()).await?;
        Ok(Catalog {
            dirpath: self.dirpath,
            strategy: self.strategy,
            backups,
        })
    }

    /// Update the catalog's list of backups with the current content of `dirpath`.
    pub async fn refresh_mut(&mut self) -> Result<()> {
        self.backups = Self::parse_backup_filenames(self.dirpath.as_path()).await?;
        Ok(())
    }

    /// Total number of backups in the catalog.
    pub fn len(&self) -> usize {
        self.backups.len()
    }

    /// Return `true` if the catalog has no backups.
    pub fn is_empty(&self) -> bool {
        self.backups.is_empty()
    }

    /// Filepath of the most recent backup.
    ///
    /// Because backups are sorted from oldest to most recent, both strategies agree on this.
    pub fn latest(&self) -> Option<&Backup> {
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
            purgeable,
            retainable: _retainable,
            ..
        } = self.plan();

        let n = purgeable.len();
        for backup in purgeable {
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
    /// By default, this prints a table of backups, age and status with colors. If `details_flag`
    /// is `true`, the table has additional columns:
    ///
    /// - version of the archive's format
    /// - number of sessions
    /// - number of windows
    /// - number of panes
    ///
    /// but this requires to read partially each backup file.
    ///
    /// If `filepaths_flag` is `true`, only absolute filepaths are printed. This can be used in
    /// scripting scenarios.
    ///
    /// If `only_status` is a `Some(..)`, this lists only the corresponding backup filepaths,
    /// acting as if `filepaths_flag` is `true`.
    pub async fn list(
        &self,
        details_flag: bool,
        only_status: Option<BackupStatus>,
        filepaths_flag: bool,
    ) {
        if filepaths_flag || only_status.is_some() {
            match only_status {
                Some(BackupStatus::Purgeable) => {
                    let Plan { purgeable, .. } = self.plan();
                    for backup in purgeable {
                        println!("{}", backup.filepath.to_string_lossy());
                    }
                }
                Some(BackupStatus::Retainable) => {
                    let Plan { retainable, .. } = self.plan();
                    for backup in retainable {
                        println!("{}", backup.filepath.to_string_lossy());
                    }
                }
                None => {
                    for backup in self.backups.iter() {
                        println!("{}", backup.filepath.to_string_lossy());
                    }
                }
            }
        } else {
            self.print_table(details_flag).await;
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

    async fn print_table(&self, details_flag: bool) {
        println!("Strategy: {}", &self.strategy);

        // Try to strip the HOME prefix from self.dirpath, otherwise return self.dirpath.
        let location: Cow<Path> = {
            if let Some(remainder) = env::var("HOME")
                .ok()
                .and_then(|home_dir| self.dirpath.strip_prefix(home_dir).ok())
            {
                Cow::Owned(PathBuf::from("$HOME").join(remainder))
            } else {
                Cow::Borrowed(&self.dirpath)
            }
        };
        println!("Location: `{}`\n", location.to_string_lossy());

        let Plan {
            purgeable,
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
                "{:4} {:32} {:11} {:12} {:11} {:8} {:8}",
                "", "NAME", "AGE", "STATUS", "FILESIZE", "VERSION", "CONTENT"
            );

            // Read all metadata concurrently
            let tasks: Vec<_> = statuses
                .iter()
                .map(|&(backup, _)| {
                    let backup_filepath = backup.filepath.clone();
                    task::spawn(async move { v1::Metadata::read_file(backup_filepath).await })
                })
                .collect();
            let metadatas: Result<Vec<_>> = join_all(tasks).await.into_iter().collect();
            let metadatas = metadatas.expect("Cannot read metadata files");

            // Build & print table rows
            for (index, ((backup, status), metadata)) in
                iter::zip(indices, iter::zip(statuses, metadatas))
            {
                let filename = backup.filepath.file_name().unwrap().to_string_lossy();
                let filesize = fs::metadata(backup.filepath.as_path()).await.unwrap().len();
                let filesize = bytes2(filesize as f64);

                let color = match status {
                    BackupStatus::Purgeable => yellow,
                    BackupStatus::Retainable => green,
                };
                let age = backup.age(now);

                let overview = metadata.overview();
                let version = &metadata.version;

                println!(
                        "{index:3}. {color}{filename:32}{reset} {age:11} {color}{status:12}{reset} {filesize:11} {version:8} {overview:8}"
                    );
            }
        } else {
            // Table header
            println!("{:4} {:32} {:11} {:11}", "", "NAME", "AGE", "STATUS");

            // Build & print table rows
            for (index, (backup, status)) in iter::zip(indices, statuses) {
                let filename = backup.filepath.file_name().unwrap().to_string_lossy();
                let color = match status {
                    BackupStatus::Purgeable => yellow,
                    BackupStatus::Retainable => green,
                };
                let age = backup.age(now);

                println!(
                    "{index:3}. {color}{filename:32}{reset} {age:11} {color}{status:6}{reset}"
                );
            }
        }

        println!(
            "\n{} backups: {} retainable, {} purgeable",
            self.len(),
            retainable.len(),
            purgeable.len(),
        );
    }
}
