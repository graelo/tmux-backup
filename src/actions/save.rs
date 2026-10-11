//! Retrieve session information and panes content save to a backup.

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use tempfile::TempDir;

use crate::{Result, management::archive::v1, tmux, tmux::Tmux};
use tmux_lib::utils;

/// Shell commands that are recognized for prompt line dropping.
///
/// When capturing pane content, if the active command is one of these shells,
/// we can optionally drop the last N lines to avoid capturing the shell prompt.
const DETECTED_SHELLS: &[&str] = &["zsh", "bash", "fish"];

/// Save the tmux sessions, windows and panes into a backup at `backup_dirpath`,
/// recording `client` as the one to switch back to on restore.
///
/// After saving, this function returns the path to the backup and the number of
/// sessions, windows and panes.
///
/// # Notes
///
/// - The `backup_dirpath` folder is assumed to exist (done during catalog initialization).
/// - `client` is taken rather than looked up: on a control handle, tmux would
///   name the control client itself. Resolve it on a spawning handle, before
///   attaching, with `current_client`.
/// - Backups have a name similar to `backup-20220731T222948.tar.zst`.
///
pub fn save<P: AsRef<Path>>(
    tmux: &Tmux,
    client: tmux::client::Client,
    backup_dirpath: P,
    num_lines_to_drop: usize,
) -> Result<(PathBuf, v1::Overview)> {
    // Prepare the temp directory.
    let temp_dir = TempDir::new()?;
    let temp_panes_content_dir = temp_dir.path().join(v1::PANES_DIR_NAME);
    fs::create_dir_all(&temp_panes_content_dir)?;

    // Query sessions & windows on their own thread, so that the pane captures
    // overlap with it.
    let (metadata, panes) = thread::scope(|scope| {
        let metadata = scope.spawn(move || v1::Metadata::new_with_client(tmux, client));
        let panes = save_panes(tmux, &temp_panes_content_dir, num_lines_to_drop);
        (metadata.join().expect("metadata query panicked"), panes)
    });
    let num_panes = panes?;
    let metadata = metadata?;

    // Save the format version & metadata into the temp folder.
    let temp_version_filepath = temp_dir.path().join(v1::VERSION_FILENAME);
    fs::write(&temp_version_filepath, v1::FORMAT_VERSION)?;

    let temp_metadata_filepath = temp_dir.path().join(v1::METADATA_FILENAME);
    fs::write(&temp_metadata_filepath, serde_json::to_string(&metadata)?)?;

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
        num_sessions: metadata.sessions.len() as u16,
        num_windows: metadata.windows.len() as u16,
        num_panes,
    };

    Ok((new_backup_filepath, overview))
}

/// List the panes and save the content of each into `destination_dir`, returning how many there
/// were.
pub(crate) fn save_panes(
    tmux: &Tmux,
    destination_dir: &Path,
    num_lines_to_drop: usize,
) -> Result<u16> {
    let panes = tmux.available_panes()?;
    let num_panes = panes.len() as u16;
    save_panes_content(tmux, &panes, destination_dir, num_lines_to_drop)?;
    Ok(num_panes)
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
fn save_panes_content(
    tmux: &Tmux,
    panes: &[tmux::pane::Pane],
    destination_dir: &Path,
    num_lines_to_drop: usize,
) -> Result<()> {
    // One thread per pane. On a control handle the captures take turns on the
    // connection, which tmux-lib measured faster than a client per pane; the
    // threads still overlap the trimming and the file writes.
    thread::scope(|scope| {
        let handles: Vec<_> = panes
            .iter()
            .map(|pane| {
                scope.spawn(move || {
                    save_pane_content(tmux, pane, destination_dir, num_lines_to_drop)
                })
            })
            .collect();

        handles
            .into_iter()
            .try_for_each(|handle| handle.join().expect("pane capture panicked"))
    })
}

/// Retrieve the content of `pane` and save it into `destination_dir`.
fn save_pane_content(
    tmux: &Tmux,
    pane: &tmux::pane::Pane,
    destination_dir: &Path,
    num_lines_to_drop: usize,
) -> Result<()> {
    let drop_n_last_lines = lines_to_drop_for_pane(&pane.command, num_lines_to_drop);
    let stdout = tmux.capture_pane(&pane.id)?;
    let cleaned_buffer = cleanup_captured_buffer(&stdout, drop_n_last_lines);

    let filepath = destination_dir.join(format!("pane-{}.txt", pane.id));
    fs::write(filepath, cleaned_buffer)?;
    Ok(())
}

/// Clean a captured pane buffer while safely limiting the number of trailing lines to drop.
fn cleanup_captured_buffer(buffer: &[u8], requested_drop_count: usize) -> Vec<u8> {
    let available_line_count = captured_line_count(buffer);
    utils::cleanup_captured_buffer(buffer, requested_drop_count.min(available_line_count))
}

/// Return the number of lines retained by tmux-lib before it drops requested trailing lines.
fn captured_line_count(buffer: &[u8]) -> usize {
    let lines: Vec<_> = buffer
        .split(|byte| *byte == b'\n')
        .map(|line| line.trim_ascii_end())
        .collect();

    lines
        .iter()
        .rposition(|line| !line.is_empty())
        .map_or(lines.len(), |last_nonempty| last_nonempty + 1)
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

    mod buffer_cleanup {
        use super::*;

        #[test]
        fn clamps_dropped_lines_to_short_buffer() {
            let cleaned = cleanup_captured_buffer(b"one line\n", 10);

            assert!(cleaned.is_empty());
        }

        #[test]
        fn handles_an_empty_buffer() {
            let cleaned = cleanup_captured_buffer(b"", 1);

            assert!(cleaned.is_empty());
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
    }
}
