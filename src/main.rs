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
                    "🛑 Could not read `{}`: {}",
                    config.backup_dirpath.to_string_lossy(),
                    e
                ),
                false,
            );
            return;
        }
    };

    match config.command {
        Command::Save { stdout } => {
            match save(&catalog.dirpath).await {
                Ok((backup_filepath, report)) => {
                    let message = format!("{report}, persisted to `{:?}`", backup_filepath);
                    success_message(message, stdout);
                }
                Err(e) => {
                    failure_message(format!("🛑 Could not save sessions: {}", e), stdout);
                }
            };
        }

        Command::Restore { .. } => unimplemented!(),

        Command::Catalog { command } => match command {
            CatalogSubcommand::List => {
                println!(
                    "Catalog: {} backups in `{}`\n",
                    &catalog.size(),
                    &catalog.dirpath.to_string_lossy()
                );
                for backup_path in catalog.outdated_backups.iter() {
                    println!("{} (outdated)", backup_path.to_string_lossy());
                }
                for backup_path in catalog.recent_backups.iter() {
                    println!("{}", backup_path.to_string_lossy());
                }
            }
        },
    }
}

fn main() {
    let config = Config::parse();
    task::block_on(run(config));
}

fn success_message(message: String, stdout: bool) {
    if stdout {
        println!("{message}");
    } else {
        tmux_display_message(&message);
    }
}

fn failure_message(message: String, stdout: bool) {
    if stdout {
        eprintln!("{message}");
    } else {
        tmux_display_message(&message);
    }
    std::process::exit(1);
}
