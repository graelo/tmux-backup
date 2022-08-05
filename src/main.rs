use async_std::task;
use clap::Parser;

use tmux_revive::{
    config::CatalogSubcommand, config::Command, config::Config, save, tmux_display_message, Catalog,
};

fn main() {
    let config = Config::parse();

    match config.command {
        Command::Save { rotate_size } => {
            match task::block_on(save::save(&config.archive_dirpath, rotate_size)) {
                Ok(report) => {
                    let message = format!(
                        "{report}, persisted to {}",
                        config.archive_dirpath.to_string_lossy()
                    );
                    success_message(&message, config.stdout);
                }
                Err(e) => {
                    let message = format!("🛑 Could not save sessions: {}", e);
                    failure_message(&message, config.stdout);
                }
            };
        }

        Command::Restore { .. } => unimplemented!(),

        Command::Catalog { command } => match command {
            CatalogSubcommand::List { rotate_size } => {
                match task::block_on(Catalog::new(&config.archive_dirpath, rotate_size)) {
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
                        let message = format!("🛑 Could not list archives: {}", e);
                        failure_message(&message, config.stdout);
                    }
                }
            }
        },
    }
}

fn success_message(message: &str, stdout: bool) {
    if stdout {
        println!("{message}");
    } else {
        tmux_display_message(message);
    }
}

fn failure_message(message: &str, stdout: bool) {
    if stdout {
        eprintln!("{message}");
        std::process::exit(1);
    } else {
        tmux_display_message(message);
    }
}
