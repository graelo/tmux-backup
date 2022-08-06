//! Catalog of all backups.
//!

use std::path::{Path, PathBuf};

use anyhow::Result;
use async_std::fs;
use async_std::stream::StreamExt;
use regex::Regex;

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
        fs::create_dir_all(&dirpath).await?;

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

        Ok(Catalog {
            dirpath: dirpath.to_path_buf(),
            strategy,
            backup_files: backups,
        })
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

    /// Apply the compaction strategy
    ///
    /// # Important
    ///
    /// This will probably delete files in the `dirpath` folder.
    pub async fn compact(&mut self) -> Result<()> {
        Ok(())
    }
}
