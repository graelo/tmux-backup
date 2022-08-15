//! Restore sessions, windows and panes from the content of a backup.

use std::{collections::HashSet, path::Path};

use anyhow::Result;
use async_std::task;
use futures::future::join_all;

use crate::{
    error::ParseError,
    management::archive::v1,
    tmux::{self, session::Session, window::Window},
};

pub async fn restore<P: AsRef<Path>>(backup_filepath: P) -> Result<v1::Overview> {
    tmux::server::start().await?;

    println!("restoring `{}`", backup_filepath.as_ref().to_string_lossy());

    let metadata = v1::read_metadata(backup_filepath).await?;

    let existing_sessions_names: HashSet<_> = tmux::session::available_sessions()
        .await?
        .into_iter()
        .map(|s| s.name)
        .collect();

    let mut handles = vec![];

    for session in &metadata.sessions {
        if existing_sessions_names.contains(&session.name) {
            eprintln!("skip creating existing session {}", session.name);
            continue;
        }

        let session = session.clone();
        let related_windows = metadata.windows_related_to(&session);

        let handle = task::spawn(async move { restore_session(session, related_windows).await });
        handles.push(handle);
    }

    join_all(handles).await;

    tmux::server::kill_placeholder_session().await?; // created above by server::start()

    Ok(metadata.overview())
}

/// Create the session along with its windows and panes.
///
/// The session is created with the first window in order to give it the right name. The remainder
/// of windows are created in sequence, to preserve the order from the backup.
async fn restore_session(session: Session, windows: Vec<Window>) -> Result<(), ParseError> {
    // 1. Create the session and the windows (each has one empty pane).

    // A session is guaranteed to have at least one window.
    let first_window_name = windows.first().unwrap().name.as_str();
    tmux::session::new_session(&session, first_window_name).await?;

    for window in windows.iter().skip(1) {
        tmux::window::new_window(window, session.dirpath.as_path(), &session.name).await?;
    }

    // 2. Create panes in each window.

    Ok(())
}
