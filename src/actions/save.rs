//! Retrieve session information and panes content save to a backup.

use std::path::{Path, PathBuf};

use async_fs as fs;
use futures::future::join_all;
use smol;
use tempfile::TempDir;

use crate::{management::archive::v1, tmux, Result};
use tmux_lib::utils;

/// Shell commands that are recognized for prompt line dropping.
///
/// When capturing pane content, if the active command is one of these shells,
/// we can optionally drop the last N lines to avoid capturing the shell prompt.
const DETECTED_SHELLS: &[&str] = &["zsh", "bash", "fish"];

/// Save the tmux sessions, windows and panes into a backup at `backup_dirpath`.
///
/// After saving, this function returns the path to the backup and the number of
/// sessions, windows and panes.
///
/// # Notes
///
/// - The `backup_dirpath` folder is assumed to exist (done during catalog initialization).
/// - Backups have a name similar to `backup-20220731T222948.tar.zst`.
///
pub async fn save<P: AsRef<Path>>(
    backup_dirpath: P,
    num_lines_to_drop: usize,
) -> Result<(PathBuf, v1::Overview)> {
    // Prepare the temp directory.
    let temp_dir = TempDir::new()?;

    // Save sessions & windows into `metadata.json` in the temp folder.
    let metadata_task: smol::Task<Result<(PathBuf, PathBuf, u16, u16)>> = {
        let temp_dirpath = temp_dir.path().to_path_buf();

        smol::spawn(async move {
            let temp_version_filepath = temp_dirpath.join(v1::VERSION_FILENAME);
            fs::write(&temp_version_filepath, v1::FORMAT_VERSION).await?;

            let metadata = v1::Metadata::new().await?;

            let json = serde_json::to_string(&metadata)?;

            let temp_metadata_filepath = temp_dirpath.join(v1::METADATA_FILENAME);
            fs::write(temp_metadata_filepath.as_path(), json).await?;

            Ok((
                temp_version_filepath,
                temp_metadata_filepath,
                metadata.sessions.len() as u16,
                metadata.windows.len() as u16,
            ))
        })
    };

    // Save pane contents in the temp folder.
    let (temp_panes_content_dir, num_panes) = {
        let temp_panes_content_dir = temp_dir.path().join(v1::PANES_DIR_NAME);
        fs::create_dir_all(&temp_panes_content_dir).await?;

        let panes = tmux::pane::available_panes().await?;
        let num_panes = panes.len() as u16;
        save_panes_content(panes, &temp_panes_content_dir, num_lines_to_drop).await?;

        (temp_panes_content_dir, num_panes)
    };
    let (temp_version_filepath, temp_metadata_filepath, num_sessions, num_windows) =
        metadata_task.await?;

    // Tar-compress content of temp folder into a new backup file in `backup_dirpath`.
    let new_backup_filepath = v1::new_backup_filepath(backup_dirpath.as_ref());

    v1::create_from_paths(
        &new_backup_filepath,
        &temp_version_filepath,
        &temp_metadata_filepath,
        &temp_panes_content_dir,
    )?;

    // Cleanup the entire temp folder.
    temp_dir.close()?;

    let overview = v1::Overview {
        version: v1::FORMAT_VERSION.to_string(),
        num_sessions,
        num_windows,
        num_panes,
    };

    Ok((new_backup_filepath, overview))
}

/// Determine if the given command is a recognized shell.
///
/// Used to decide whether to drop trailing lines (shell prompt) when capturing pane content.
fn is_shell_command(command: &str) -> bool {
    DETECTED_SHELLS.contains(&command)
}

/// Calculate how many lines to drop from pane capture based on the active command.
///
/// If the pane is running a recognized shell, we drop `num_lines_to_drop` lines
/// to avoid capturing the shell prompt. For other commands, we keep everything.
fn lines_to_drop_for_pane(pane_command: &str, num_lines_to_drop: usize) -> usize {
    if is_shell_command(pane_command) {
        num_lines_to_drop
    } else {
        0
    }
}

/// For each provided pane, retrieve the content and save it into `destination_dir`.
async fn save_panes_content<P: AsRef<Path>>(
    panes: Vec<tmux::pane::Pane>,
    destination_dir: P,
    num_lines_to_drop: usize,
) -> Result<()> {
    let mut handles = Vec::new();

    for pane in panes {
        let dest_dir = destination_dir.as_ref().to_path_buf();
        let drop_n_last_lines = lines_to_drop_for_pane(&pane.command, num_lines_to_drop);

        let handle = smol::spawn(async move {
            let stdout = pane.capture().await.unwrap();
            let cleaned_buffer = utils::cleanup_captured_buffer(&stdout, drop_n_last_lines);

            let filename = format!("pane-{}.txt", pane.id);
            let filepath = dest_dir.join(filename);
            fs::write(filepath, cleaned_buffer).await
        });
        handles.push(handle);
    }

    join_all(handles).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod shell_detection {
        use super::*;

        #[test]
        fn recognizes_zsh() {
            assert!(is_shell_command("zsh"));
        }

        #[test]
        fn recognizes_bash() {
            assert!(is_shell_command("bash"));
        }

        #[test]
        fn recognizes_fish() {
            assert!(is_shell_command("fish"));
        }

        #[test]
        fn rejects_vim() {
            assert!(!is_shell_command("vim"));
        }

        #[test]
        fn rejects_nvim() {
            assert!(!is_shell_command("nvim"));
        }

        #[test]
        fn rejects_python() {
            assert!(!is_shell_command("python"));
        }

        #[test]
        fn rejects_empty_command() {
            assert!(!is_shell_command(""));
        }

        #[test]
        fn rejects_similar_but_different() {
            // Shell name as substring shouldn't match
            assert!(!is_shell_command("zsh-5.9"));
            assert!(!is_shell_command("/bin/zsh"));
            assert!(!is_shell_command("bash-5.2"));
        }

        #[test]
        fn case_sensitive() {
            assert!(!is_shell_command("ZSH"));
            assert!(!is_shell_command("BASH"));
            assert!(!is_shell_command("Fish"));
        }
    }

    mod lines_to_drop {
        use super::*;

        #[test]
        fn drops_lines_for_shells() {
            assert_eq!(lines_to_drop_for_pane("zsh", 2), 2);
            assert_eq!(lines_to_drop_for_pane("bash", 3), 3);
            assert_eq!(lines_to_drop_for_pane("fish", 1), 1);
        }

        #[test]
        fn zero_drop_for_non_shells() {
            assert_eq!(lines_to_drop_for_pane("vim", 5), 0);
            assert_eq!(lines_to_drop_for_pane("python", 10), 0);
            assert_eq!(lines_to_drop_for_pane("htop", 3), 0);
        }

        #[test]
        fn zero_requested_means_zero_dropped() {
            assert_eq!(lines_to_drop_for_pane("zsh", 0), 0);
            assert_eq!(lines_to_drop_for_pane("bash", 0), 0);
        }
    }

    mod constants {
        use super::*;

        #[test]
        fn detected_shells_includes_common_shells() {
            assert!(DETECTED_SHELLS.contains(&"zsh"));
            assert!(DETECTED_SHELLS.contains(&"bash"));
            assert!(DETECTED_SHELLS.contains(&"fish"));
        }

        #[test]
        fn detected_shells_is_not_empty() {
            assert!(!DETECTED_SHELLS.is_empty());
        }
    }
}
