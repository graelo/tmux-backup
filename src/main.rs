use async_std::task;
use clap::Parser;

use tmux_revive::{config::Command, config::Config, display_message, save};

fn main() {
    let config = Config::parse();

    match config.command {
        Command::Save {
            stdout,
            num_archives,
        } => {
            match task::block_on(save::save(&config.archive_dirpath, num_archives)) {
                Ok(report) => {
                    let message = format!(
                        "{report}, persisted to {}",
                        config.archive_dirpath.to_string_lossy()
                    );
                    if stdout {
                        println!("{message}");
                    } else {
                        display_message(&message);
                    }
                }
                Err(e) => {
                    let message = format!("🛑 Could not save sessions: {}", e);
                    if stdout {
                        eprintln!("{message}");
                        std::process::exit(1);
                    } else {
                        display_message(&message);
                    }
                }
            };
        }
        Command::Restore { .. } => unimplemented!(),
    }
}
