#![warn(missing_docs)]
//! Main runner

use async_std::task;
use clap::{CommandFactory, Parser};
use clap_complete::generate;

use tmux_revive::{
    actions::save,
    config::{CatalogSubcommand, Command, Config},
    management::Catalog,
    tmux_display_message,
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
            CatalogSubcommand::List { sublist } => catalog.list(sublist),
            CatalogSubcommand::Describe { backup_filepath } => catalog.describe(backup_filepath),
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

        Command::Save { to_tmux, compact } => {
            match save(&catalog.dirpath).await {
                Ok((backup_filepath, backup_overview)) => {
                    if compact {
                        // In practice this should never fail: write to the catalog already ensures
                        // the catalog's dirpath is writable.
                        catalog
                            .compact()
                            .await
                            .expect("Success saving but could not compact");
                    }
                    let message = format!(
                        "✅ {backup_overview}, persisted to `{}`",
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
        Output::ToTmux => tmux_display_message(&message),
        Output::Stdout => println!("{message}"),
        Output::Both => {
            println!("{message}");
            tmux_display_message(&message)
        }
    }
}

fn failure_message<O: Into<Output>>(message: String, output: O) {
    match output.into() {
        Output::ToTmux => tmux_display_message(&message),
        Output::Stdout => eprintln!("{message}"),
        Output::Both => {
            eprintln!("{message}");
            tmux_display_message(&message)
        }
    };
    std::process::exit(1);
}
