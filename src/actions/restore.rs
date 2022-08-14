//! Restore sessions, windows and panes from the content of a backup.

use std::{collections::HashSet, path::Path};

use anyhow::Result;
use async_std::task;
use futures::future::join_all;

use crate::{management::archive::v1, tmux};

pub async fn restore<P: AsRef<Path>>(backup_filepath: P) -> Result<v1::Overview> {
    tmux::server::start().await?;

    println!("restoring `{}`", backup_filepath.as_ref().to_string_lossy());

    let metadata = v1::read_metadata(backup_filepath).await?;

    let existing_sessions_names: HashSet<_> = tmux::session::available_sessions()
        .await?
        .into_iter()
        .map(|s| s.name)
        .collect();

    let mut restore_session_tasks = vec![];
    for session in &metadata.sessions {
        if existing_sessions_names.contains(&session.name) {
            println!("skip creating existing session {}", session.name);
            continue;
        }
        let session = session.clone();
        let handle = task::spawn(async move { tmux::session::new_session(&session).await });
        restore_session_tasks.push(handle);
    }
    join_all(restore_session_tasks).await;

    Ok(metadata.overview())
}
