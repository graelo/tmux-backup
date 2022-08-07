use async_std::task;
use clap::Parser;

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
            CatalogSubcommand::Compact => match catalog.compact().await {
                Ok(n) => {
                    let message = format!("✅ deleted {n} outdated backups");
                    success_message(message, true)
                }
                Err(e) => failure_message(format!("🛑 Could not compact backups: {}", e), true),
            },
        },

        Command::Save { to_tmux } => {
            match save(&catalog.dirpath).await {
                Ok((backup_filepath, report)) => {
                    let message = format!("✅ {report}, persisted to `{:?}`", backup_filepath);
                    success_message(message, to_tmux);
                }
                Err(e) => {
                    failure_message(format!("🛑 Could not save sessions: {}", e), to_tmux);
                }
            };
        }

        Command::Restore { .. } => unimplemented!(),
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
