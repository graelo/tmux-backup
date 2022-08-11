//! High-level representation of a backup for catalog operations and reporting.

use std::path::PathBuf;

use chrono::NaiveDateTime;
use clap::ValueEnum;

/// Quick access, high-level representation of a backup.
///
/// This is sufficient for the [`Catalog`](crate::management::catalog::Catalog) to list backups
/// and decide whether or not a backup should be deleted or kept.
///
/// `Backup` provides only information which can be derived from the file name. This avoids to open
/// the file, deal with the format, parse the metadata, etc.
pub struct Backup {
    /// Path to the backup file.
    pub filepath: PathBuf,

    /// Backup date.
    pub creation_date: NaiveDateTime,
}

/// Which subset of backups to print.
#[derive(Debug, Clone, ValueEnum)]
pub enum BackupStatus {
    /// Retainable backups only.
    Retainable,
    /// Disposable backups only.
    Disposable,
}
