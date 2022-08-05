//! Catalog of all archives.
//!

use std::path;

use anyhow::Result;
use async_std::fs;
use async_std::stream::StreamExt;
use regex::Regex;

/// Catalag of all archives.
pub struct Catalog {
    /// Location of the catalog.
    pub dirpath: path::PathBuf,

    /// Sorted list of outdated archive files (oldest to newest).
    pub outdated: Vec<path::PathBuf>,

    /// Sorted list of recent archive files (oldest to newest).
    pub recent: Vec<path::PathBuf>,
}

impl Catalog {
    /// Return a new `Catalog` by listing the archives in `dirpath`.
    ///
    /// Only archive files such as `archive-20220804T221153.tar.zst` are added to the catalog.
    pub async fn new(dirpath: &path::Path, rotate_size: usize) -> Result<Catalog> {
        let mut archives: Vec<path::PathBuf> = vec![];

        let pattern = r#".*archive-\d{8}T\d{6}\.tar\.zst"#;
        let matcher = Regex::new(pattern).unwrap();

        let mut entries = fs::read_dir(dirpath).await?;
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            let path = entry.path();
            if matcher.captures(&path.to_string_lossy()).is_some() {
                archives.push(path.into());
            }
        }

        archives.sort();

        let index = std::cmp::max(0, archives.len() - rotate_size);
        let recent = archives.split_off(index);

        Ok(Catalog {
            dirpath: dirpath.to_path_buf(),
            outdated: archives,
            recent,
        })
    }

    /// Total number of archives in the catalog.
    pub fn size(&self) -> usize {
        self.outdated.len() + self.recent.len()
    }

    /// Filepath of the current archive.
    ///
    /// This is usually the most recent archive.
    pub fn current(&self) -> Option<&path::Path> {
        self.recent.last().map(|p| p.as_ref())
    }

    /// Compact the catalog by deleting old archives files, keeping only the `rotate_size` most
    /// recent ones.
    pub async fn compact(&mut self, rotate_size: usize) -> Result<()> {
        Ok(())
    }
}
