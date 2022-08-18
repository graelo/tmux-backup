//! Simple configuration with only `save` and `restore` commands.

use std::env;
use std::path::PathBuf;

use clap::{ArgAction, ArgGroup, Parser, Subcommand, ValueHint};
use clap_complete::Shell;

use crate::management::{backup::BackupStatus, compaction::Strategy};

/// Catalog subcommands.
#[derive(Debug, Subcommand)]
pub enum CatalogSubcommand {
    /// Print a list of backups to stdout.
    ///
    /// By default, this prints a table of backups, age and status with colors. The flag `--details`
    /// prints additional columns.
    ///
    /// If the flag `--filepaths` is set, only absolute filepaths are printed. This can be used in
    /// scripting scenarios.
    ///
    /// Options `--only purgeable` or `--only retainable` will list only the corresponding backups.
    /// They will activate the flag `--filepaths` automatically.
    List {
        /// Add details columns to the table.
        ///
        /// Print number of sessions, windows and panes in the backup and the backup's format
        /// version. This is slightly slower because it requires each backup file to be partially
        /// read.
        #[clap(long = "details", action = ArgAction::SetTrue)]
        details_flag: bool,

        /// List only backups having this status.
        #[clap(long = "only", value_enum, value_parser)]
        only_backup_status: Option<BackupStatus>,

        /// Print filepaths instead of the table format.
        #[clap(long = "filepaths", action = ArgAction::SetTrue)]
        filepaths_flag: bool,
    },
    /// Apply the catalog's compaction strategy: this deletes all purgable backups.
    Compact,
}

/// Indicate whether to save (resp. restore) the Tmux sessions to (resp. from) a backup.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Save the Tmux sessions to a new backup file.
    ///
    /// Sessions, windows, and panes geometry + content are saved in an archive format inside the
    /// backup folder. In that folder, the backup name is expected to be similar to
    /// `backup-20220531T123456.tar.zst`.
    ///
    /// If you run this command via a Tmux keybinding, use the `--to-tmux` flag in order to send a
    /// one-line report to the Tmux status bar. If you run this command from the terminal, ignore
    /// this flag in order to print the one-line report in the terminal.
    Save {
        /// Print a one-line report in the Tmux status bar, otherwise print to stdout.
        #[clap(long, action = ArgAction::SetTrue)]
        to_tmux: bool,

        /// Delete purgeable backups after saving.
        #[clap(long, action = ArgAction::SetTrue)]
        compact: bool,
    },

    /// Restore the Tmux sessions from a backup file.
    ///
    /// Sessions, windows and panes geometry + content are read from the backup marked as "current"
    /// (often the most recent backup) inside the backup folder. In that folder, the backup name is
    /// expected to be similar to `backup-20220531T123456.tar.zst`.
    ///
    /// If you run this command via a Tmux keybinding, use the `--to-tmux` flag in order to send a
    /// one-line report to the Tmux status bar. If you run this command from the terminal, ignore
    /// this flag in order to print the one-line report in the terminal.
    Restore {
        /// Print a one-line report in the Tmux status bar, otherwise print to stdout.
        #[clap(long, action = ArgAction::SetTrue)]
        to_tmux: bool,

        /// Filepath of the backup to restore, by default, pick latest.
        #[clap(value_parser)]
        backup_filepath: Option<PathBuf>,
    },

    /// Catalog commands.
    Catalog {
        /// Catalog commands.
        #[clap(subcommand)]
        command: CatalogSubcommand,
    },

    /// Describe the content of a backup file.
    Describe {
        /// Path to the backup file.
        #[clap(value_parser, value_hint = ValueHint::FilePath)]
        backup_filepath: PathBuf,
    },

    /// Print a shell completion script to stdout.
    GenerateCompletion {
        /// Shell for which you want completion.
        #[clap(value_parser = clap::value_parser!(Shell))]
        shell: Shell,
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
    /// If unspecified, it falls back on: `$XDG_STATE_HOME/tmux-backup`, then on
    /// `$HOME/.local/state/tmux-backup`.
    #[clap(short = 'd', long = "dirpath", value_hint = ValueHint::DirPath,
        default_value_os_t = default_backup_dirpath())]
    pub backup_dirpath: PathBuf,

    /// Number of recent backups to keep, for instance 10.
    #[clap(group="strategy",
        short = 'k',
        long="strategy-most-recent",
        value_name = "NUMBER",
        value_parser = clap::value_parser!(u16).range(1..),
        env = "TMUX_BACKUP_STRATEGY_MOST_RECENT"
    )]
    strategy_most_recent: Option<u16>,

    /// Apply a classic backup strategy.
    ///
    /// Keep
    /// the lastest per hour for the past 24 hours,
    /// the lastest per day for the past 7 days,
    /// the lastest per week of the past 4 weeks,
    /// the lastest per month of this year.
    #[clap(
        group = "strategy",
        short = 'l',
        long = "strategy-classic",
        value_parser,
        env = "TMUX_BACKUP_STRATEGY_CLASSIC"
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
/// If `$XDG_STATE_HOME` is defined, the function returns `$XDG_STATE_HOME/tmux-backup`, otherwise,
/// it returns `$HOME/.local/state/tmux-backup`.
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

    state_home.join("tmux-backup")
}
