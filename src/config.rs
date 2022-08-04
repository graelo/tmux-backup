//! Simple configuration with only `save` and `restore` commands.

use std::env;
use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};

/// Indicate whether to save (resp. restore) the Tmux sessions to (resp. from) an archive.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Save the Tmux sessions to an archive.
    ///
    /// Sessions, windows, and panes geometry + content are saved in an archive format inside
    /// `ARCHIVE_DIRPATH`. In that path, the archive name is expected to be similar to
    /// `archive-20220531T123456.tar.zst`.
    ///
    /// You can specify the max. number of archives to keep around.
    ///
    /// If you run this command from the terminal, consider using the `--stdout` flag in order to
    /// print the report in the terminal. Otherwise, if you run it via a Tmux keybinding, the
    /// one-line report is printed with `tmux display-message`.
    Save {
        /// Print the report (num. sessions, windows & panes) on stdout,
        /// otherwise send to Tmux.
        #[clap(long = "stdout", action = ArgAction::SetTrue, default_value = "false")]
        stdout: bool,

        /// How many archive files to keep in `ARCHIVE_DIRPATH`.
        #[clap(long = "history", default_value = "10")]
        num_archives: u16,
    },

    /// Restore the Tmux sessions.
    ///
    /// Sessions, windows and panes geometry + content are read from the most recent archive inside
    /// `ARCHIVE_DIRPATH`. In that path, the archive name is expected to be similar to
    /// `archive-20220531T123456.tar.zst`.
    ///
    /// If you run this command from the terminal, consider using the `--stdout` flag in order to
    /// print the report in the terminal. Otherwise, if you run it via a Tmux keybinding, the
    /// one-line report is printed with `tmux display-message`.
    Restore {
        /// Print the report (num. sessions, windows & panes) on stdout,
        /// otherwise send to Tmux.
        #[clap(long = "stdout", action = ArgAction::SetTrue, default_value = "false")]
        stdout: bool,
    },
}

/// Save or restore Tmux sessions.
#[derive(Debug, Parser)]
#[clap(author, about, version)]
#[clap(propagate_version = true)]
pub struct Config {
    /// Location of archives.
    ///
    /// If unspecified, it falls back on: `$XDG_STATE_HOME/tmux-revive`, then on
    /// `$HOME/.local/state/tmux-revive`.
    #[clap(short = 'd', long = "dirpath", default_value_os_t = default_archive_dirpath())]
    pub archive_dirpath: PathBuf,

    /// Selection of commands.
    #[clap(subcommand)]
    pub command: Command,
}

/// Determine the folder where to save archives.
///
/// The following is tried:
///
/// - `$XDG_STATE_HOME`
/// - `$HOME/.local/state`
///
/// # Panics
///
/// This function panics if even `$HOME` cannot be obtained from the environment.
fn default_archive_dirpath() -> PathBuf {
    let state_home = match env::var("XDG_STATE_HOME") {
        Ok(v) => PathBuf::from(v),
        Err(_) => match env::var("HOME") {
            Ok(v) => PathBuf::from(v).join(".local").join("state"),
            Err(_) => panic!("Cannot find `$HOME` in the environment"),
        },
    };

    state_home.join("tmux-revive")
}
