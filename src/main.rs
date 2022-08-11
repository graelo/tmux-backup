#![warn(missing_docs)]
//! Main runner

use async_std::task;
use clap::{CommandFactory, Parser};
use clap_complete::generate;

use tmux_revive::{
    actions::save,
    config::{CatalogSubcommand, Command, Config},
    management::{archive::v1, catalog::Catalog},
    tmux,
};

async fn run(config: Config) {
    let catalog = match Catalog::new(&config.backup_dirpath, config.strategy()).await {
        Ok(catalog) => catalog,
        Err(e) => {
            failure_message(
                format!(
                    "🛑 Catalog error at `{}`: {}",
                    config.backup_dirpath.to_string_lossy(),
                    e
                ),
                Output::Both,
            );
            return;
        }
    };

    match config.command {
        Command::Catalog { command } => match command {
            CatalogSubcommand::List {
                backup_status,
                details_flag,
            } => catalog.list(backup_status, details_flag).await,
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
        },

        Command::Describe { backup_filepath } => {
            v1::print_description(backup_filepath).await.unwrap()
        }

        Command::Save { to_tmux, compact } => {
            match save(&catalog.dirpath).await {
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

        Command::Restore { .. } => unimplemented!(),

        Command::Generate { shell } => {
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
