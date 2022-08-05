//! Catalog of all backups.
//!

use std::path;

use anyhow::Result;
use async_std::fs;
use async_std::stream::StreamExt;
use regex::Regex;

/// Catalag of all backups.
pub struct Catalog {
    /// Location of the catalog.
    pub dirpath: path::PathBuf,

    /// Sorted list of outdated backup files (oldest to newest).
    pub outdated: Vec<path::PathBuf>,

    /// Sorted list of recent backup files (oldest to newest).
    pub recent: Vec<path::PathBuf>,
}

impl Catalog {
    /// Return a new `Catalog` by listing the backups in `dirpath`.
    ///
    /// Only backup files such as `backup-20220804T221153.tar.zst` are added to the catalog.
    pub async fn new(dirpath: &path::Path, rotate_size: usize) -> Result<Catalog> {
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

        let index = std::cmp::max(0, backup_files.len() - rotate_size);
        let recent = backup_files.split_off(index);

        Ok(Catalog {
            dirpath: dirpath.to_path_buf(),
            outdated: backup_files,
            recent,
        })
    }

    /// Total number of backups in the catalog.
    pub fn size(&self) -> usize {
        self.outdated.len() + self.recent.len()
    }

    /// Filepath of the current backup.
    ///
    /// This is usually the most recent backup.
    pub fn current(&self) -> Option<&path::Path> {
        self.recent.last().map(|p| p.as_ref())
    }

    /// Compact the catalog by deleting old backup files, keeping only the `rotate_size` most
    /// recent ones.
    pub async fn compact(&mut self, rotate_size: usize) -> Result<()> {
        Ok(())
    }
}
