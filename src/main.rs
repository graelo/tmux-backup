use async_std::task;
use clap::Parser;

use tmux_revive::{config::Command, config::Config, display_message, save};

fn main() {
    let config = Config::parse();

    match config.command {
        Command::Save => {
            match task::block_on(save::save(&config.archive_dirpath)) {
                Ok(report) => {
                    let message = format!(
                        "✅ {} sessions ({} windows, {} panes) persisted to {}",
                        report.num_sessions,
                        report.num_windows,
                        report.num_panes,
                        config.archive_dirpath.to_str().unwrap_or("???")
                    );
                    if config.stdout {
                        println!("{message}");
                    } else {
                        display_message(&message)
                            .expect("Cannot communicate with Tmux for displaying message")
                    }
                }
                Err(e) => println!("An error ocurred: {}", e),
            };
        }
        Command::Restore => unimplemented!(),
    }
}
