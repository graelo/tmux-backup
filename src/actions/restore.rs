//! Restore sessions, windows and panes from the content of a backup.

use std::{
    collections::HashSet,
    iter::zip,
    path::{Path, PathBuf},
};

use async_std::task;
use futures::future::join_all;
use tempfile::TempDir;

use crate::{
    error::Error,
    management::archive::v1,
    tmux::{self, pane::Pane, session::Session, window::Window},
    Result,
};

/// Name of the placeholder session.
const PLACEHOLDER_SESSION_NAME: &str = "[placeholder]";

/// Restore all sessions, windows & panes from the backup file.
pub async fn restore<P: AsRef<Path>>(backup_filepath: P) -> Result<v1::Overview> {
    // Prepare the temp directory with the content of the backup.
    let temp_dir = TempDir::new()?;
    v1::unpack(backup_filepath.as_ref(), temp_dir.path()).await?;
    let panes_content_dir = temp_dir.path().join("panes-content");

    // Start tmux if needed.
    let not_in_tmux = std::env::var("TMUX").is_err();
    if not_in_tmux {
        tmux::server::start(PLACEHOLDER_SESSION_NAME).await?;
    }

    // Get the default command used to start panes.
    let default_command = tmux::server::default_command().await?;

    // Restore sessions, windows and panes.
    let metadata = v1::Metadata::read_file(backup_filepath).await?;

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
        let panes_content_dirpath = panes_content_dir.clone();
        let default_command = default_command.clone();

        let handle = task::spawn(async move {
            restore_session(
                session,
                related_windows,
                related_panes,
                panes_content_dirpath,
                &default_command,
            )
            .await
        });
        handles.push(handle);
    }

    join_all(handles)
        .await
        .into_iter()
        .collect::<Result<()>>()?;

    // Delete the temp restore directory.
    temp_dir.close()?;

    // Set the client last and current session.
    tmux::client::switch_client(&metadata.client.last_session_name).await?;
    tmux::client::switch_client(&metadata.client.session_name).await?;

    // Kill the session used to start the server.
    if not_in_tmux {
        tmux::server::kill_session(PLACEHOLDER_SESSION_NAME).await?;
        println!(
            "Attach to your last session with `tmux attach -t {}`",
            &metadata.client.session_name
        );

        // Return an overview of the archived tmux environment, which is identical, in principle,
        // with the new one. We cannot do more because the client metadata cannot be fetched.
        Ok(metadata.overview())
    } else {
        if tmux::server::kill_session("0").await.is_err() {
            let message = "
            Unusual start conditions:
            - you started from outside tmux but no existing session named `0` was found
            - check the state of your session
           ";
            return Err(Error::ConfigError(message.to_string()));
        }

        // Return an overview of the restored tmux environment.
        let metadata = v1::Metadata::new().await?;
        Ok(metadata.overview())
    }
}

/// Association between a pane from the backup with a new target pane id.
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
/// # Note
///
/// This strategy is faster than creating a placeholder window and removing it at the end (checked
/// multiple times).
async fn restore_session(
    session: Session,
    session_windows: Vec<Window>,
    panes_per_window: Vec<Vec<Pane>>,
    panes_content_dir: PathBuf,
    default_command: &str,
) -> Result<()> {
    let mut pairs: Vec<Pair> = vec![];

    // Create the session (first window and first pane as side-effects) or only windows & panes.

    for (index, (src_window, src_panes)) in zip(&session_windows, &panes_per_window).enumerate() {
        let first_pane = src_panes.first().unwrap(); // guaranteed
        let content_filepath = panes_content_dir.join(format!("pane-{}.txt", first_pane.id));
        let pane_command = format!(
            "cat {} ; exec {}",
            content_filepath.to_string_lossy(),
            &default_command
        );

        let (new_window_id, new_pane_id) = {
            if index == 0 {
                let (_new_session_id, new_window_id, new_pane_id) = tmux::session::new_session(
                    &session,
                    src_window,
                    first_pane,
                    Some(&pane_command),
                )
                .await?;
                (new_window_id, new_pane_id)
            } else {
                tmux::window::new_window(&session, src_window, first_pane, Some(&pane_command))
                    .await?
            }
        };

        // 1b. Store the association between the original pane and this new pane.
        pairs.push(Pair {
            source: first_pane.clone(),
            target: new_pane_id,
        });

        // 1c. Create the other panes of the first window, storing their association with the original
        //     panes for this first window. Each new pane is configured as the original pane.
        for pane in src_panes.iter().skip(1) {
            let content_filepath = panes_content_dir.join(format!("pane-{}.txt", pane.id));
            let pane_command = format!(
                "cat {} ; exec {}",
                content_filepath.to_string_lossy(),
                &default_command
            );

            let new_pane_id =
                tmux::pane::new_pane(pane, Some(&pane_command), &new_window_id).await?;
            pairs.push(Pair {
                source: pane.clone(),
                target: new_pane_id,
            });
        }

        // 1d. Set the layout
        tmux::window::set_layout(&src_window.layout, &new_window_id).await?;

        if src_window.is_active {
            tmux::window::select_window(&new_window_id).await?;
        }
    }

    for pair in &pairs {
        if pair.source.is_active {
            tmux::pane::select_pane(&pair.target).await?;
        }
    }

    Ok(())
}
