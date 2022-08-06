//! Catalog of all backups.
//!

use std::path;

use anyhow::Result;
use async_std::fs;
use async_std::stream::StreamExt;
use regex::Regex;

use super::compaction::{Plan, Strategy};

/// Catalag of all backups.
pub struct Catalog {
    /// Location of the catalog.
    pub dirpath: path::PathBuf,

    /// Compaction strategy.
    pub strategy: Strategy,

    /// Sorted list of outdated backup files (oldest to newest).
    pub outdated_backups: Vec<path::PathBuf>,

    /// Sorted list of recent backup files (oldest to newest).
    pub recent_backups: Vec<path::PathBuf>,
}

impl Catalog {
    /// Return a new `Catalog` by listing the backups in `dirpath`.
    ///
    /// The catalog only manages backup files such as `backup-20220804T221153.tar.zst`.
    pub async fn new(dirpath: &path::Path, strategy: Strategy) -> Result<Catalog> {
        let mut backup_files: Vec<path::PathBuf> = vec![];

        let pattern = r#".*backup-\d{8}T\d{6}\.tar\.zst"#;
        let matcher = Regex::new(pattern).unwrap();

        let mut entries = fs::read_dir(dirpath).await?;
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let path = entry.path();
            if matcher.captures(&path.to_string_lossy()).is_some() {
                backup_files.push(path.into());
            }
        }

        backup_files.sort();

        // let strategy = Strategy::most_recent(10);
        let Plan { to_keep, to_remove } = strategy.plan(backup_files);

        Ok(Catalog {
            dirpath: dirpath.to_path_buf(),
            strategy,
            outdated_backups: to_remove,
            recent_backups: to_keep,
        })
    }

    /// Total number of backups in the catalog.
    pub fn size(&self) -> usize {
        self.outdated_backups.len() + self.recent_backups.len()
    }

    /// The catalog's dirpath as a string, for convenience in error messages.
    pub fn location(&self) -> String {
        self.dirpath.to_string_lossy().into()
    }

    /// Filepath of the current backup.
    ///
    /// This is usually the most recent backup.
    pub fn current(&self) -> Option<&path::Path> {
        self.recent_backups.last().map(|p| p.as_ref())
    }

    /// Apply the compaction strategy, by deleting old backup files, keeping only the `rotate_size`
    /// most recent ones.
    pub async fn compact(&mut self) -> Result<()> {
        Ok(())
    }
}
