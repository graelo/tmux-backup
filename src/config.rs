//! Configuration.

use std::env;
use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum, ValueHint};
use clap_complete::Shell;

use crate::management::{backup::BackupStatus, compaction::Strategy};

/// Save or restore Tmux sessions.
#[derive(Debug, Parser)]
#[clap(author, about, version)]
#[clap(propagate_version = true)]
pub struct Config {
    /// Location of backups.
    ///
    /// If unspecified, it falls back on: `$XDG_STATE_HOME/tmux-backup`, then on
    /// `$HOME/.local/state/tmux-backup`.
    #[arg(short = 'd', long = "dirpath", value_hint = ValueHint::DirPath,
        default_value_os_t = default_backup_dirpath())]
    pub backup_dirpath: PathBuf,

    /// Selection of commands.
    #[command(subcommand)]
    pub command: Command,
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
        /// Choose a strategy for managing backups.
        #[command(flatten)]
        strategy: StrategyConfig,

        /// Print a one-line report in the Tmux status bar, otherwise print to stdout.
        #[arg(long, action = ArgAction::SetTrue)]
        to_tmux: bool,

        /// Delete purgeable backups after saving.
        #[arg(long, action = ArgAction::SetTrue)]
        compact: bool,

        /// Number of lines to ignore during capture if the active command is a shell.
        ///
        /// At the time of saving, for each pane where the active command is one of (`zsh`, `bash`,
        /// `fish`), the shell prompt is waiting for input. If tmux-backup naively captures the
        /// entire history, on restoring that backup, a new shell prompt will also appear. This
        /// obviously pollutes history with repeated shell prompts.
        ///
        /// If you know the number of lines your shell prompt occupies on screen, set this option
        /// to that number (simply `1` in my case). These last lines will not be captured. On
        /// restore, this gives the illusion of history continuity without repetition.
        #[arg(
            short = 'i',
            long = "ignore-last-lines",
            value_name = "NUMBER",
            default_value_t = 0
        )]
        num_lines_to_drop: u8,
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
        /// Choose a strategy for managing backups.
        #[command(flatten)]
        strategy: StrategyConfig,

        /// Print a one-line report in the Tmux status bar, otherwise print to stdout.
        #[arg(long, action = ArgAction::SetTrue)]
        to_tmux: bool,

        /// Filepath of the backup to restore, by default, pick latest.
        #[arg(value_parser)]
        backup_filepath: Option<PathBuf>,
    },

    /// Catalog commands.
    Catalog {
        /// Choose a strategy for managing backups.
        #[command(flatten)]
        strategy: StrategyConfig,

        /// Catalog commands.
        #[command(subcommand)]
        command: CatalogSubcommand,
    },

    /// Describe the content of a backup file.
    Describe {
        /// Path to the backup file.
        #[arg(value_parser, value_hint = ValueHint::FilePath)]
        backup_filepath: PathBuf,
    },

    /// Print a shell completion script to stdout.
    GenerateCompletion {
        /// Shell for which you want completion.
        #[arg(value_enum, value_parser = clap::value_parser!(Shell))]
        shell: Shell,
    },

    /// Outputs the default tmux plugin config to stdout.
    ///
    /// Similar to shell completions, this is done once when installing tmux-backup. Type
    /// `tmux-backup init > ~/.tmux/plugins/tmux-backup.tmux`. and source it
    /// from your `~/.tmux.conf`. See the README for details.
    Init,
}

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
        #[arg(long = "details", action = ArgAction::SetTrue)]
        details_flag: bool,

        /// List only backups having this status.
        #[arg(long = "only", value_enum, value_parser)]
        only_backup_status: Option<BackupStatus>,

        /// Print filepaths instead of the table format.
        #[arg(long = "filepaths", action = ArgAction::SetTrue)]
        filepaths_flag: bool,
    },

    /// Apply the catalog's compaction strategy: this deletes all purgable backups.
    Compact,
}

/// Strategy values
#[derive(Debug, Clone, ValueEnum)]
enum StrategyValues {
    /// Apply a most-recent strategy, keeping only n backups.
    MostRecent,

    /// Apply a classic backup strategy.
    ///
    /// Keep
    /// the lastest per hour for the past 24 hours,
    /// the lastest per day for the past 7 days,
    /// the lastest per week of the past 4 weeks,
    /// the lastest per month of this year.
    Classic,
}

/// Strategy configuration.
#[derive(Debug, clap::Args)]
pub struct StrategyConfig {
    #[arg(short = 's', long = "strategy", value_enum, default_value_t = StrategyValues::MostRecent)]
    strategy: StrategyValues,

    /// Number of recent backups to keep, for instance 10.
    #[arg(
        short = 'n',
        long,
        value_name = "NUMBER",
        value_parser = clap::value_parser!(u16).range(1..),
        default_value_t = 10,
    )]
    num_backups: u16,
}

//
// Helpers
//

impl StrategyConfig {
    /// Compaction Strategy corresponding to the CLI arguments.
    pub fn strategy(&self) -> Strategy {
        match self.strategy {
            StrategyValues::MostRecent => Strategy::most_recent(self.num_backups as usize),
            StrategyValues::Classic => Strategy::Classic,
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    mod strategy_config {
        use super::*;

        // Helper to parse a save command and extract its strategy
        // Note: strategy flags (-s, -n) belong to the subcommand, not the root
        fn parse_save_strategy(subcommand_args: &[&str]) -> Strategy {
            let mut full_args = vec!["tmux-backup", "save"];
            full_args.extend(subcommand_args);

            let config = Config::try_parse_from(full_args).unwrap();
            match config.command {
                Command::Save { strategy, .. } => strategy.strategy(),
                _ => panic!("Expected Save command"),
            }
        }

        #[test]
        fn default_strategy_is_most_recent_with_10() {
            let strategy = parse_save_strategy(&[]);

            match strategy {
                Strategy::KeepMostRecent { k } => assert_eq!(k, 10),
                _ => panic!("Expected KeepMostRecent"),
            }
        }

        #[test]
        fn explicit_most_recent_strategy() {
            let strategy = parse_save_strategy(&["-s", "most-recent"]);

            match strategy {
                Strategy::KeepMostRecent { k } => assert_eq!(k, 10), // default n
                _ => panic!("Expected KeepMostRecent"),
            }
        }

        #[test]
        fn most_recent_with_custom_count() {
            let strategy = parse_save_strategy(&["-s", "most-recent", "-n", "25"]);

            match strategy {
                Strategy::KeepMostRecent { k } => assert_eq!(k, 25),
                _ => panic!("Expected KeepMostRecent"),
            }
        }

        #[test]
        fn classic_strategy() {
            let strategy = parse_save_strategy(&["-s", "classic"]);

            assert!(matches!(strategy, Strategy::Classic));
        }

        #[test]
        fn long_form_arguments_work() {
            let strategy =
                parse_save_strategy(&["--strategy", "most-recent", "--num-backups", "42"]);

            match strategy {
                Strategy::KeepMostRecent { k } => assert_eq!(k, 42),
                _ => panic!("Expected KeepMostRecent"),
            }
        }

        #[test]
        fn num_backups_ignored_for_classic() {
            // -n is accepted but ignored for classic strategy
            let strategy = parse_save_strategy(&["-s", "classic", "-n", "99"]);

            assert!(matches!(strategy, Strategy::Classic));
        }
    }

    mod cli_parsing {
        use super::*;

        #[test]
        fn save_command_parses() {
            let config = Config::try_parse_from(["tmux-backup", "save"]).unwrap();
            assert!(matches!(config.command, Command::Save { .. }));
        }

        #[test]
        fn save_with_compact_flag() {
            let config = Config::try_parse_from(["tmux-backup", "save", "--compact"]).unwrap();
            match config.command {
                Command::Save { compact, .. } => assert!(compact),
                _ => panic!("Expected Save command"),
            }
        }

        #[test]
        fn save_with_to_tmux_flag() {
            let config = Config::try_parse_from(["tmux-backup", "save", "--to-tmux"]).unwrap();
            match config.command {
                Command::Save { to_tmux, .. } => assert!(to_tmux),
                _ => panic!("Expected Save command"),
            }
        }

        #[test]
        fn save_with_ignore_lines() {
            let config = Config::try_parse_from(["tmux-backup", "save", "-i", "2"]).unwrap();
            match config.command {
                Command::Save {
                    num_lines_to_drop, ..
                } => assert_eq!(num_lines_to_drop, 2),
                _ => panic!("Expected Save command"),
            }
        }

        #[test]
        fn restore_command_parses() {
            let config = Config::try_parse_from(["tmux-backup", "restore"]).unwrap();
            assert!(matches!(config.command, Command::Restore { .. }));
        }

        #[test]
        fn restore_with_specific_file() {
            let config =
                Config::try_parse_from(["tmux-backup", "restore", "/path/to/backup.tar.zst"])
                    .unwrap();
            match config.command {
                Command::Restore {
                    backup_filepath, ..
                } => {
                    assert_eq!(
                        backup_filepath,
                        Some(PathBuf::from("/path/to/backup.tar.zst"))
                    );
                }
                _ => panic!("Expected Restore command"),
            }
        }

        #[test]
        fn catalog_list_command() {
            let config = Config::try_parse_from(["tmux-backup", "catalog", "list"]).unwrap();
            match config.command {
                Command::Catalog { command, .. } => {
                    assert!(matches!(command, CatalogSubcommand::List { .. }));
                }
                _ => panic!("Expected Catalog command"),
            }
        }

        #[test]
        fn catalog_list_with_details() {
            let config =
                Config::try_parse_from(["tmux-backup", "catalog", "list", "--details"]).unwrap();
            match config.command {
                Command::Catalog { command, .. } => match command {
                    CatalogSubcommand::List { details_flag, .. } => {
                        assert!(details_flag);
                    }
                    _ => panic!("Expected List subcommand"),
                },
                _ => panic!("Expected Catalog command"),
            }
        }

        #[test]
        fn catalog_list_with_only_purgeable() {
            let config =
                Config::try_parse_from(["tmux-backup", "catalog", "list", "--only", "purgeable"])
                    .unwrap();
            match config.command {
                Command::Catalog { command, .. } => match command {
                    CatalogSubcommand::List {
                        only_backup_status, ..
                    } => {
                        assert!(matches!(only_backup_status, Some(BackupStatus::Purgeable)));
                    }
                    _ => panic!("Expected List subcommand"),
                },
                _ => panic!("Expected Catalog command"),
            }
        }

        #[test]
        fn catalog_compact_command() {
            let config = Config::try_parse_from(["tmux-backup", "catalog", "compact"]).unwrap();
            match config.command {
                Command::Catalog { command, .. } => {
                    assert!(matches!(command, CatalogSubcommand::Compact));
                }
                _ => panic!("Expected Catalog command"),
            }
        }

        #[test]
        fn custom_backup_dirpath() {
            let config =
                Config::try_parse_from(["tmux-backup", "-d", "/custom/path", "save"]).unwrap();
            assert_eq!(config.backup_dirpath, PathBuf::from("/custom/path"));
        }

        #[test]
        fn describe_command() {
            let config =
                Config::try_parse_from(["tmux-backup", "describe", "/path/to/backup.tar.zst"])
                    .unwrap();
            match config.command {
                Command::Describe { backup_filepath } => {
                    assert_eq!(backup_filepath, PathBuf::from("/path/to/backup.tar.zst"));
                }
                _ => panic!("Expected Describe command"),
            }
        }

        #[test]
        fn generate_completion_command() {
            let config =
                Config::try_parse_from(["tmux-backup", "generate-completion", "bash"]).unwrap();
            match config.command {
                Command::GenerateCompletion { shell } => {
                    assert!(matches!(shell, Shell::Bash));
                }
                _ => panic!("Expected GenerateCompletion command"),
            }
        }

        #[test]
        fn init_command() {
            let config = Config::try_parse_from(["tmux-backup", "init"]).unwrap();
            assert!(matches!(config.command, Command::Init));
        }

        #[test]
        fn rejects_invalid_num_backups_zero() {
            let result = Config::try_parse_from(["tmux-backup", "-n", "0", "save"]);
            assert!(result.is_err());
        }

        #[test]
        fn rejects_negative_num_backups() {
            let result = Config::try_parse_from(["tmux-backup", "-n", "-5", "save"]);
            assert!(result.is_err());
        }
    }

    // Note: Testing `default_backup_dirpath()` would require manipulating
    // environment variables (XDG_STATE_HOME, HOME), which can interfere with
    // other tests running in parallel. Consider using a test harness like
    // `temp_env` or running these tests serially with `#[serial]` if needed.
}
