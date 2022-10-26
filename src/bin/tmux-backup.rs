//! Main runner

use std::path::Path;

use async_std::task;
use clap::{CommandFactory, Parser};
use clap_complete::generate;

use tmux_backup::{
    actions::{restore, save},
    config::{CatalogSubcommand, Command, Config, StrategyConfig},
    management::{archive::v1, catalog::Catalog},
    tmux,
};

async fn init_catalog<P: AsRef<Path>>(
    backup_dirpath: P,
    strategy_config: StrategyConfig,
) -> Catalog {
    match Catalog::new(&backup_dirpath.as_ref(), strategy_config.strategy()).await {
        Ok(catalog) => catalog,
        Err(e) => {
            failure_message(
                format!(
                    "🛑 Catalog error at `{}`: {}",
                    backup_dirpath.as_ref().to_string_lossy(),
                    e
                ),
                Output::Both,
            );
            std::process::exit(1);
        }
    }
}

async fn run(config: Config) {
    match config.command {
        Command::Catalog { strategy, command } => {
            let catalog = init_catalog(&config.backup_dirpath, strategy).await;

            match command {
                CatalogSubcommand::List {
                    details_flag,
                    only_backup_status,
                    filepaths_flag,
                } => {
                    catalog
                        .list(details_flag, only_backup_status, filepaths_flag)
                        .await
                }
                CatalogSubcommand::Compact => match catalog.compact().await {
                    Ok(n) => {
                        let message = format!("✅ deleted {n} outdated backups");
                        success_message(message, Output::Stdout)
                    }
                    Err(e) => failure_message(
                        format!("🛑 Could not compact backups: {}", e),
                        Output::Stdout,
                    ),
                },
            }
        }

        Command::Describe { backup_filepath } => {
            v1::print_description(backup_filepath).await.unwrap()
        }

        Command::Save {
            strategy,
            to_tmux,
            compact,
            num_lines_to_drop,
        } => {
            let catalog = init_catalog(&config.backup_dirpath, strategy).await;

            match save(&catalog.dirpath, num_lines_to_drop as usize).await {
                Ok((backup_filepath, archive_overview)) => {
                    if compact {
                        // In practice this should never fail: write to the catalog already ensures
                        // the catalog's dirpath is writable.
                        catalog
                            .refresh()
                            .await
                            .expect("Success saving but could not refresh")
                            .compact()
                            .await
                            .expect("Success saving but could not compact");
                    }
                    let message = format!(
                        "✅ {archive_overview}, persisted to `{}`",
                        backup_filepath.to_string_lossy()
                    );
                    success_message(message, to_tmux);
                }
                Err(e) => {
                    failure_message(format!("🛑 Could not save sessions: {}", e), to_tmux);
                }
            };
        }

        Command::Restore {
            strategy,
            to_tmux,
            backup_filepath,
        } => {
            let catalog = init_catalog(&config.backup_dirpath, strategy).await;

            // Either the provided filepath, or catalog.latest(), or failure message
            let backup_to_restore = {
                if let Some(ref backup_filepath) = backup_filepath {
                    backup_filepath.as_path()
                } else if let Some(backup) = catalog.latest() {
                    &backup.filepath
                } else {
                    failure_message("🛑 No available backup to restore".to_string(), to_tmux);
                    return;
                }
            };
            match restore(backup_to_restore).await {
                Ok(overview) => {
                    let message = format!(
                        "✅ restored {overview} from `{}`",
                        backup_to_restore.to_string_lossy()
                    );
                    success_message(message, to_tmux)
                }
                Err(e) => {
                    failure_message(format!("🛑 Could not restore sessions: {}", e), to_tmux);
                }
            }
        }

        Command::GenerateCompletion { shell } => {
            let mut app = Config::command();
            let name = app.get_name().to_string();
            generate(shell, &mut app, name, &mut std::io::stdout());
        }
    }
}

fn main() {
    let config = Config::parse();
    task::block_on(run(config));
}

enum Output {
    ToTmux,
    Stdout,
    Both,
}

impl From<bool> for Output {
    fn from(to_tmux: bool) -> Self {
        if to_tmux {
            Output::ToTmux
        } else {
            Output::Stdout
        }
    }
}

fn success_message<O: Into<Output>>(message: String, output: O) {
    match output.into() {
        Output::ToTmux => tmux::display_message(&message),
        Output::Stdout => println!("{message}"),
        Output::Both => {
            println!("{message}");
            tmux::display_message(&message)
        }
    }
}

fn failure_message<O: Into<Output>>(message: String, output: O) {
    match output.into() {
        Output::ToTmux => tmux::display_message(&message),
        Output::Stdout => eprintln!("{message}"),
        Output::Both => {
            eprintln!("{message}");
            tmux::display_message(&message)
        }
    };
    std::process::exit(1);
}
