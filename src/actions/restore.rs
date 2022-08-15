//! Restore sessions, windows and panes from the content of a backup.

use std::{collections::HashSet, iter::zip, path::Path};

use anyhow::Result;
use async_std::task;
use futures::future::join_all;

use crate::{
    error::ParseError,
    management::archive::v1,
    tmux::{self, pane::Pane, session::Session, window::Window},
};

pub async fn restore<P: AsRef<Path>>(backup_filepath: P) -> Result<v1::Overview> {
    tmux::server::start().await?;

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
        let related_panes: Vec<Vec<Pane>> = related_windows
            .iter()
            .map(|w| metadata.panes_related_to(w).into_iter().cloned().collect())
            .collect();

        let handle =
            task::spawn(
                async move { restore_session(session, related_windows, related_panes).await },
            );
        handles.push(handle);
    }

    if let Err(e) = join_all(handles)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, ParseError>>()
    {
        return Err(anyhow::anyhow!("error: {e}"));
    }

    tmux::server::kill_placeholder_session().await?; // created above by server::start()

    Ok(metadata.overview())
}

/// Associates a pane from the backup with a new target pane id.
#[derive(Debug, Clone)]
struct Pair {
    /// Pane definition from the backup.
    source: tmux::pane::Pane,
    /// Target pane id.
    target: tmux::pane_id::PaneId,
}

/// Create a session along with its windows and panes.
///
/// The session is created with the first window in order to give it the right name. The remainder
/// of windows are created in sequence, to preserve the order from the backup.
///
async fn restore_session(
    session: Session,
    related_windows: Vec<Window>,
    related_panes: Vec<Vec<Pane>>,
) -> Result<(), ParseError> {
    // 1a. Create the session and the first window (and the first pane as a side-effect).

    let first_window = related_windows
        .first()
        .expect("a session should have at least one window");
    let first_window_panes = related_panes
        .first()
        .expect("a window should have at least one pane");
    let first_pane = first_window_panes.first().unwrap();

    let (new_window_id, new_pane_id) = tmux::session::new_session(
        &session,
        first_pane.dirpath.as_path(),
        first_window.name.as_str(),
    )
    .await?;

    let mut pairs: Vec<Pair> = vec![];

    // 1b. Store the association between the original pane and this new pane.
    pairs.push(Pair {
        source: first_pane.clone(),
        target: new_pane_id,
    });

    // 1c. Create the other panes of the first window, storing their association with the original
    //     panes for this first window. Each new pane is configured as the original pane.
    for pane in first_window_panes.iter().skip(1) {
        let new_pane_id = tmux::pane::new_pane(pane, &new_window_id).await?;
        pairs.push(Pair {
            source: pane.clone(),
            target: new_pane_id,
        });
    }

    // 1d. Set the layout
    tmux::window::set_layout(&first_window.layout, new_window_id).await?;

    // 2. Create the other windows (and their first pane as a side-effect).
    for (window, panes) in zip(&related_windows, &related_panes).skip(1) {
        let first_pane = panes
            .first()
            .expect("a window should have at least one pane");
        let (new_window_id, new_pane_id) =
            tmux::window::new_window(window, first_pane.dirpath.as_path(), &session.name).await?;

        // 2b. Store the association between the original pane and this new pane.
        pairs.push(Pair {
            source: first_pane.clone(),
            target: new_pane_id,
        });

        // 2c. Then add the new panes in each window.
        for pane in panes.iter().skip(1) {
            let new_pane_id = tmux::pane::new_pane(pane, &new_window_id).await?;
            pairs.push(Pair {
                source: pane.clone(),
                target: new_pane_id,
            });
        }

        // 2d. Set the layout
        tmux::window::set_layout(&window.layout, new_window_id).await?;
    }

    Ok(())
}
