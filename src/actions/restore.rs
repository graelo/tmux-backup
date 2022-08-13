//! Restore sessions, windows and panes from the content of a backup.

use std::path::Path;

use anyhow::Result;

use crate::management::archive::v1;

pub async fn restore<P: AsRef<Path>>(backup_filepath: P) -> Result<v1::Overview> {
    println!("restoring `{}`", backup_filepath.as_ref().to_string_lossy());

    Ok(v1::Overview {
        version: "1.0".into(),
        num_sessions: 0,
        num_windows: 0,
        num_panes: 0,
    })
}
