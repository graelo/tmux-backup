//! Simple configuration with only `save` and `restore` commands.

use std::env;
use std::path::PathBuf;

use clap::{ArgAction, ArgGroup, Parser, Subcommand, ValueEnum};

use crate::management::Strategy;

/// Which sublist to print.
#[derive(Debug, Clone, ValueEnum)]
pub enum SubList {
    /// Retainable backups only.
    Retainable,
    /// Disposable backups only.
    Disposable,
}

/// Catalog subcommands.
#[derive(Debug, Subcommand)]
pub enum CatalogSubcommand {
    /// List backups in the catalog to stdout.
    ///
    /// If `--only disposable` or `--only retainable` are passed, print the corresponding list,
    /// otherwise print all details in colored output.
    List {
        /// Only list disposable backups.
        #[clap(long = "only", value_enum, value_parser)]
        sublist: Option<SubList>,
    },
    /// Delete outdated backups by applying the catalog's compaction strategy.
    Compact,
}

/// Indicate whether to save (resp. restore) the Tmux sessions to (resp. from) a backup.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Save the Tmux sessions to a new backup file.
    ///
    /// Sessions, windows, and panes geometry + content are saved in an archive format inside
    /// the backup folder. In that folder, the backup name is expected to be similar to
    /// `backup-20220531T123456.tar.zst`.
    ///
    /// If you run this command via a Tmux keybinding, use the `--to-tmux` flag in order to display
    /// the one-line report in the Tmux status bar. Otherwise, if you run it from the terminal,
    /// ignore this flag in order to print the report in the terminal.
    Save {
        /// Send a report (num. sessions, etc.) to the Tmux status bar, otherwise print to stdout.
        #[clap(long, action = ArgAction::SetTrue)]
        to_tmux: bool,
    },

    /// Restore the Tmux sessions from a backup file.
    ///
    /// Sessions, windows and panes geometry + content are read from the backup marked as "current"
    /// (often the most recent backup) inside the backup folder. In that folder, the backup name is
    /// expected to be similar to `backup-20220531T123456.tar.zst`.
    ///
    /// If you run this command via a Tmux keybinding, use the `--to-tmux` flag in order to display
    /// the one-line report in the Tmux status bar. Otherwise, if you run it from the terminal,
    /// ignore this flag in order to print the report in the terminal.
    Restore {
        /// Send a report (num. sessions, etc.) to the Tmux status bar, otherwise print to stdout.
        #[clap(long, action = ArgAction::SetTrue)]
        to_tmux: bool,
    },

    /// Operations on the catalog of backups.
    Catalog {
        /// List the backups in the catalog, indicating the disposable ones.
        #[clap(subcommand)]
        command: CatalogSubcommand,
    },
}

/// Save or restore Tmux sessions.
#[derive(Debug, Parser)]
#[clap(author, about, version)]
#[clap(propagate_version = true)]
#[clap(group(
            ArgGroup::new("strategy")
                .required(true)
        ))]
pub struct Config {
    /// Location of backups.
    ///
    /// If unspecified, it falls back on: `$XDG_STATE_HOME/tmux-revive`, then on
    /// `$HOME/.local/state/tmux-revive`.
    #[clap(short = 'd', long = "dirpath", default_value_os_t = default_backup_dirpath())]
    pub backup_dirpath: PathBuf,

    /// Number of recent backups to keep, for instance 10.
    #[clap(group="strategy",
        short = 'k',
        long="strategy-most-recent",
        value_name = "NUMBER",
        value_parser = clap::value_parser!(u16).range(1..)
    )]
    strategy_most_recent: Option<u16>,

    /// Apply a classic backup strategy (keep last hour, then last day, then last week, then last month).
    #[clap(
        group = "strategy",
        short = 'l',
        long = "strategy-classic",
        value_parser
    )]
    strategy_classic: bool,

    /// Selection of commands.
    #[clap(subcommand)]
    pub command: Command,
}

//
// Helpers
//

impl Config {
    /// Compaction Strategy corresponding to the CLI arguments.
    pub fn strategy(&self) -> Strategy {
        if let Some(k) = self.strategy_most_recent {
            Strategy::most_recent(k as usize)
        } else {
            Strategy::Classic
        }
    }
}
/// Determine the folder where to save backups.
///
/// If `$XDG_STATE_HOME` is defined, the function returns `$XDG_STATE_HOME/tmux-revive`,
/// otherwise, it returns `$HOME/.local/state/tmux-revive`.
///
/// # Panics
///
/// This function panics if even `$HOME` cannot be obtained from the environment.
fn default_backup_dirpath() -> PathBuf {
    let state_home = match env::var("XDG_STATE_HOME") {
        Ok(v) => PathBuf::from(v),
        Err(_) => match env::var("HOME") {
            Ok(v) => PathBuf::from(v).join(".local").join("state"),
            Err(_) => panic!("Cannot find `$HOME` in the environment"),
        },
    };

    state_home.join("tmux-revive")
}
