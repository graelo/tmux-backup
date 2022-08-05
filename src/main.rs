use async_std::task;
use clap::Parser;

use tmux_revive::{
    config::CatalogSubcommand, config::Command, config::Config, save, tmux_display_message, Catalog,
};

async fn run(config: Config) {
    match config.command {
        Command::Save {
            rotate_size,
            stdout,
        } => {
            match save::save(&config.archive_dirpath, rotate_size).await {
                Ok(report) => {
                    let message = format!(
                        "{report}, persisted to {}",
                        config.archive_dirpath.to_string_lossy()
                    );
                    success_message(message, stdout);
                }
                Err(e) => {
                    failure_message(format!("🛑 Could not save sessions: {}", e), stdout);
                }
            };
        }

        Command::Restore { .. } => unimplemented!(),

        Command::Catalog { command } => match command {
            CatalogSubcommand::List { rotate_size } => {
                match Catalog::new(&config.archive_dirpath, rotate_size).await {
                    Ok(catalog) => {
                        println!(
                            "Catalog: {} archives in `{}`\n",
                            &catalog.size(),
                            &catalog.dirpath.to_string_lossy()
                        );
                        for archive_path in catalog.outdated.iter() {
                            println!("{} (outdated)", archive_path.to_string_lossy());
                        }
                        for archive_path in catalog.recent.iter() {
                            println!("{}", archive_path.to_string_lossy());
                        }
                    }
                    Err(e) => {
                        failure_message(format!("🛑 Could not list archives: {}", e), false);
                    }
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
        std::process::exit(1);
    } else {
        tmux_display_message(&message);
    }
}
