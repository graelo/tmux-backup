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
    tmux::Tmux,
};

async fn init_catalog<P: AsRef<Path>>(
    tmux: &Tmux,
    backup_dirpath: P,
    strategy_config: StrategyConfig,
) -> Catalog {
    match Catalog::new(backup_dirpath.as_ref(), strategy_config.strategy()) {
        Ok(catalog) => catalog,
        Err(e) => {
            failure_message(
                format!(
                    "🛑 Catalog cannot be created from `{}`: {e}",
                    backup_dirpath.as_ref().to_string_lossy()
                ),
                Output::Both,
                tmux,
            );
            std::process::exit(1);
        }
    }
}

async fn run(config: Config) {
    // One handle for the whole run: tmux-backup always talks to the default
    // server, and the handle is cheap to clone into the tasks that need it.
    let tmux = Tmux::spawning();

    match config.command {
        Command::Catalog { strategy, command } => {
            let catalog = init_catalog(&tmux, &config.backup_dirpath, strategy).await;

            match command {
                CatalogSubcommand::List {
                    details_flag,
                    only_backup_status,
                    filepaths_flag,
                } => catalog.list(details_flag, only_backup_status, filepaths_flag),
                CatalogSubcommand::Compact => match catalog.compact() {
                    Ok(n) => {
                        let message = format!("✅ deleted {n} outdated backups");
                        success_message(message, Output::Stdout, &tmux)
                    }
                    Err(e) => failure_message(
                        format!("🛑 Could not compact backups: {e}"),
                        Output::Stdout,
                        &tmux,
                    ),
                },
            }
        }

        Command::Describe { backup_filepath } => v1::print_description(backup_filepath).unwrap(),

        Command::Save {
            strategy,
            to_tmux,
            compact,
            num_lines_to_drop,
        } => {
            let catalog = init_catalog(&tmux, &config.backup_dirpath, strategy).await;

            match save(&tmux, &catalog.dirpath, num_lines_to_drop as usize).await {
                Ok((backup_filepath, archive_overview)) => {
                    if compact {
                        // In practice this should never fail: write to the catalog already ensures
                        // the catalog's dirpath is writable.
                        catalog
                            .refresh()
                            .expect("Success saving but could not refresh")
                            .compact()
                            .expect("Success saving but could not compact");
                    }
                    let message = format!(
                        "✅ {archive_overview}, persisted to `{}`",
                        backup_filepath.to_string_lossy()
                    );
                    success_message(message, to_tmux, &tmux);
                }
                Err(e) => {
                    failure_message(format!("🛑 Could not save sessions: {e}"), to_tmux, &tmux);
                }
            };
        }

        Command::Autosave {
            to_tmux,
            num_lines_to_drop,
        } => {
            let context = match autosave_context(&tmux, to_tmux.is_some()) {
                Ok(context) => context,
                Err(e) => {
                    failure_message(
                        format!("🛑 Could not prepare autosave: {e}"),
                        Output::Stdout,
                        &tmux,
                    );
                    return;
                }
            };

            if let Err(e) = fs::create_dir_all(&config.backup_dirpath).await {
                autosave_failure(
                    format!("🛑 Could not create autosave directory: {e}"),
                    to_tmux,
                    &context,
                    &tmux,
                );
                return;
            }

            match autosave(
                &tmux,
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
                    &tmux,
                ),
                Err(e) => autosave_failure(
                    format!("🛑 Could not autosave sessions: {e}"),
                    to_tmux,
                    &context,
                    &tmux,
                ),
            }
        }

        Command::Restore {
            strategy,
            to_tmux,
            backup_filepath,
        } => {
            let catalog = init_catalog(&tmux, &config.backup_dirpath, strategy).await;

            // Either the provided filepath, or newest ordinary backup/autosave, or failure.
            let backup_to_restore = {
                if let Some(ref backup_filepath) = backup_filepath {
                    backup_filepath.as_path()
                } else if let Some(backup_filepath) = catalog.latest_for_restore() {
                    backup_filepath
                } else {
                    failure_message(
                        "🛑 No available backup to restore".to_string(),
                        to_tmux,
                        &tmux,
                    );
                    return;
                }
            };
            match restore(&tmux, backup_to_restore).await {
                Ok(overview) => {
                    let message = format!(
                        "✅ restored {overview} from `{}`",
                        backup_to_restore.to_string_lossy()
                    );
                    success_message(message, to_tmux, &tmux)
                }
                Err(e) => {
                    failure_message(
                        format!("🛑 Could not restore sessions: {e}"),
                        to_tmux,
                        &tmux,
                    );
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
    tmux: &Tmux,
) {
    match to_tmux {
        Some(AutosaveTmuxOutput::All) => display_autosave_message(tmux, context, &message),
        None | Some(AutosaveTmuxOutput::Errors) => println!("{message}"),
    }
}

fn autosave_failure(
    message: String,
    to_tmux: Option<AutosaveTmuxOutput>,
    context: &AutosaveContext,
    tmux: &Tmux,
) {
    eprintln!("{message}");
    if to_tmux.is_some() {
        display_autosave_message(tmux, context, &message);
    }
    std::process::exit(1);
}

fn success_message<O: Into<Output>>(message: String, output: O, tmux: &Tmux) {
    // Reporting into tmux is best effort: the message is already the outcome
    // being reported, so a failure to display it must not replace it.
    match output.into() {
        Output::ToTmux => report_to_tmux(tmux, &message),
        Output::Stdout => println!("{message}"),
        Output::Both => {
            println!("{message}");
            report_to_tmux(tmux, &message);
        }
    }
}

fn report_to_tmux(tmux: &Tmux, message: &str) {
    if let Err(e) = tmux.display_message(message) {
        eprintln!("could not display the message in tmux: {e}");
    }
}

fn failure_message<O: Into<Output>>(message: String, output: O, tmux: &Tmux) {
    match output.into() {
        Output::ToTmux => report_to_tmux(tmux, &message),
        Output::Stdout => eprintln!("{message}"),
        Output::Both => {
            eprintln!("{message}");
            report_to_tmux(tmux, &message);
        }
    };
    std::process::exit(1);
}
