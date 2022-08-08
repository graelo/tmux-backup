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
                true,
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
                    success_message(message, true)
                }
                Err(e) => failure_message(format!("🛑 Could not compact backups: {}", e), true),
            },
        },

        Command::Save { to_tmux, compact } => {
            match save(&catalog.dirpath).await {
                Ok((backup_filepath, report)) => {
                    if compact {
                        // In practice this should never fail: write to the catalog already ensures
                        // the catalog's dirpath is writable.
                        catalog
                            .compact()
                            .await
                            .expect("Success saving but could not compact");
                    }
                    let message = format!("✅ {report}, persisted to `{:?}`", backup_filepath);
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

fn success_message(message: String, to_tmux: bool) {
    if to_tmux {
        tmux_display_message(&message);
    } else {
        println!("{message}");
    }
}

fn failure_message(message: String, to_tmux: bool) {
    if to_tmux {
        tmux_display_message(&message);
    } else {
        eprintln!("{message}");
    }
    std::process::exit(1);
}
