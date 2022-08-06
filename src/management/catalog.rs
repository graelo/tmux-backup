//! Catalog of all backups.
//!

use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use anyhow::Result;
use async_std::fs;
use async_std::stream::StreamExt;
use regex::Regex;

use crate::config::SubList;

use super::compaction::{Plan, Strategy};

/// Catalag of all backups.
pub struct Catalog {
    /// Location of the catalog.
    pub dirpath: PathBuf,

    /// Compaction strategy.
    pub strategy: Strategy,

    /// Sorted list of all backup files (oldest to newest).
    pub backup_files: Vec<PathBuf>,
}

impl Catalog {
    /// Return a new `Catalog` by listing the backups in `dirpath`.
    ///
    /// # Notes
    ///
    /// - The folder is created if missing.
    /// - The catalog only manages backup files such as `backup-20220804T221153.tar.zst`.
    pub async fn new(dirpath: &Path, strategy: Strategy) -> Result<Catalog> {
        fs::create_dir_all(dirpath).await?;

        let backup_files = Self::read_files(dirpath).await?;

        let catalog = Catalog {
            dirpath: dirpath.to_path_buf(),
            strategy,
            backup_files,
        };

        Ok(catalog)
    }

    /// Update the catalog's list of backups with current content of `dirpath`.
    pub async fn refresh(&mut self) -> Result<()> {
        self.backup_files = Self::read_files(self.dirpath.as_path()).await?;
        Ok(())
    }

    /// Update the catalog with the files in `dirpath`.
    async fn read_files(dirpath: &Path) -> Result<Vec<PathBuf>> {
        let mut backups: Vec<PathBuf> = vec![];

        let pattern = r#".*backup-\d{8}T\d{6}\.tar\.zst"#;
        let matcher = Regex::new(pattern).unwrap();

        let mut entries = fs::read_dir(dirpath).await?;
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let path = entry.path();
            if matcher.captures(&path.to_string_lossy()).is_some() {
                backups.push(path.into());
            }
        }

        backups.sort();

        Ok(backups)
    }

    /// Total number of backups in the catalog.
    pub fn size(&self) -> usize {
        self.backup_files.len()
    }

    /// The catalog's dirpath as a string, for convenience in error messages.
    pub fn location(&self) -> String {
        self.dirpath.to_string_lossy().into()
    }

    /// Filepath of the current backup.
    ///
    /// This is usually the most recent backup.
    pub fn current(&self) -> Option<&Path> {
        self.backup_files.last().map(|p| p.as_ref())
    }

    /// Simulate the compaction strategy: list the backup files to delete, and the ones to keep.
    pub fn plan(&self) -> Plan {
        self.strategy.plan(&self.backup_files)
    }

    /// Apply the compaction strategy.
    ///
    /// # Important
    ///
    /// This will probably delete files in the `dirpath` folder.
    pub async fn compact(&self) -> Result<usize> {
        let Plan {
            deletable,
            retainable: _retainable,
        } = self.plan();

        let n = deletable.len();
        for path in deletable {
            fs::remove_file(path).await?;
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
        self.refresh()
            .await
            .expect("Error when refreshing the catalog");
    }

    /// List backups.
    pub fn list(&self, sublist: Option<SubList>) {
        let Plan {
            deletable,
            retainable,
        } = self.plan();

        if let Some(sublist) = sublist {
            match sublist {
                SubList::Deletable => {
                    for backup_path in deletable.iter() {
                        println!("{}", backup_path.to_string_lossy());
                    }
                }
                SubList::Retainable => {
                    for backup_path in retainable.iter() {
                        println!("{}", backup_path.to_string_lossy());
                    }
                }
            }
        } else {
            println!("Catalog");
            println!("- location: `{}`:", self.location());
            println!("- strategy: {}", &self.strategy);

            let reset = "\u{001b}[0m";
            let magenta = "\u{001b}[35m";
            let green = "\u{001b}[32m";

            println!("- deletable:");
            let iter =
                RangeInclusive::new(retainable.len() + 1, retainable.len() + deletable.len())
                    .into_iter()
                    .rev();
            for (index, backup_path) in std::iter::zip(iter, deletable) {
                println!(
                    "    {:3}. {magenta}{}{reset}",
                    index,
                    backup_path.file_name().unwrap().to_string_lossy()
                );
            }

            println!("- keep:");
            let iter = RangeInclusive::new(1, retainable.len()).into_iter().rev();
            for (index, backup_path) in std::iter::zip(iter, retainable) {
                println!(
                    "    {:3}. {green}{}{reset}",
                    index,
                    backup_path.file_name().unwrap().to_string_lossy()
                );
            }
            println!(
                "\n{} backups: {} retainable, {} deletable",
                self.size(),
                retainable.len(),
                deletable.len(),
            );
        }
    }
}
