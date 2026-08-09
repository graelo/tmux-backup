//! Main runner

use std::path::Path;

use async_fs as fs;
use clap::{CommandFactory, Parser};
use clap_complete::generate;

use tmux_backup::{
    actions::{
        AutosaveContext, autosave, autosave_context, display_autosave_message, restore, save,
    },
    config::{AutosaveTmuxOutput, CatalogSubcommand, Command, Config, StrategyConfig},
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
                    "🛑 Catalog cannot be created from `{}`: {e}",
                    backup_dirpath.as_ref().to_string_lossy()
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
                        format!("🛑 Could not compact backups: {e}"),
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
                    failure_message(format!("🛑 Could not save sessions: {e}"), to_tmux);
                }
            };
        }

        Command::Autosave {
            to_tmux,
            num_lines_to_drop,
        } => {
            let context = match autosave_context(to_tmux.is_some()).await {
                Ok(context) => context,
                Err(e) => {
                    failure_message(
                        format!("🛑 Could not prepare autosave: {e}"),
                        Output::Stdout,
                    );
                    return;
                }
            };

            if let Err(e) = fs::create_dir_all(&config.backup_dirpath).await {
                autosave_failure(
                    format!("🛑 Could not create autosave directory: {e}"),
                    to_tmux,
                    &context,
                );
                return;
            }

            match autosave(
                &config.backup_dirpath,
                num_lines_to_drop as usize,
                context.clone(),
            )
            .await
            {
                Ok((backup_filepath, archive_overview)) => autosave_success(
                    format!(
                        "✅ {archive_overview}, auto-saved to `{}`",
                        backup_filepath.to_string_lossy()
                    ),
                    to_tmux,
                    &context,
                ),
                Err(e) => autosave_failure(
                    format!("🛑 Could not autosave sessions: {e}"),
                    to_tmux,
                    &context,
                ),
            }
        }

        Command::Restore {
            strategy,
            to_tmux,
            backup_filepath,
        } => {
            let catalog = init_catalog(&config.backup_dirpath, strategy).await;

            // Either the provided filepath, or newest ordinary backup/autosave, or failure.
            let backup_to_restore = {
                if let Some(ref backup_filepath) = backup_filepath {
                    backup_filepath.as_path()
                } else if let Some(backup_filepath) = catalog.latest_for_restore() {
                    backup_filepath
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
                    failure_message(format!("🛑 Could not restore sessions: {e}"), to_tmux);
                }
            }
        }

        Command::GenerateCompletion { shell } => {
            let mut app = Config::command();
            let name = app.get_name().to_string();
            generate(shell, &mut app, name, &mut std::io::stdout());
        }

        Command::Init => {
            let text = std::include_str!("../../tmux-backup.tmux");
            println!("{text}");
        }
    }
}

fn main() {
    let config = Config::parse();
    smol::block_on(run(config));
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

fn autosave_success(
    message: String,
    to_tmux: Option<AutosaveTmuxOutput>,
    context: &AutosaveContext,
) {
    match to_tmux {
        Some(AutosaveTmuxOutput::All) => display_autosave_message(context, &message),
        None | Some(AutosaveTmuxOutput::Errors) => println!("{message}"),
    }
}

fn autosave_failure(
    message: String,
    to_tmux: Option<AutosaveTmuxOutput>,
    context: &AutosaveContext,
) {
    eprintln!("{message}");
    if to_tmux.is_some() {
        display_autosave_message(context, &message);
    }
    std::process::exit(1);
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
