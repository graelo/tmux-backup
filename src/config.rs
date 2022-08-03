//! Simple configuration with only `save` and `restore` commands.

use std::env;
use std::path::PathBuf;

use clap::{ArgEnum, Parser};

/// Indicate whether to save (resp. restore) the Tmux sessions to (resp. from) an archive.
#[derive(Debug, Clone, ArgEnum)]
pub enum Command {
    /// Save the Tmux sessions.
    Save,
    /// Restore the Tmux sessions.
    Restore,
}

/// Save or restore Tmux sessions.
#[derive(Parser, Debug)]
#[clap(author, about, version)]
pub struct Config {
    /// Optional directory where to save the archive.
    ///
    /// By default, `$XDG_STATE_HOME` is used, otherwise `$HOME/.local/state`, otherwise `/tmp`.
    /// In that path, the archive name is expected to be `archive-20220531T123456.tar.zst`.
    #[clap(short = 'd', long = "dirpath", default_value_os_t = default_archive_dirpath())]
    pub archive_dirpath: PathBuf,

    /// Indicate whether to save (resp. restore) the Tmux sessions to (resp. from) an archive.
    #[clap(value_parser, arg_enum)]
    pub command: Command,
}

fn default_archive_dirpath() -> PathBuf {
    let state_home = match env::var("XDG_STATE_HOME") {
        Ok(v) => PathBuf::from(v),
        Err(_) => match env::var("HOME") {
            Ok(v) => PathBuf::from(v).join(".local").join("state"),
            Err(_) => PathBuf::from("/tmp").join("state"),
        },
    };

    state_home.join("tmux-revive")
}
