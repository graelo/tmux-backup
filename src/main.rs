use async_std::task;
use clap::Parser;

use tmux_revive::{config::Command, config::Config, display_message, save};

fn main() {
    let config = Config::parse();

    match config.command {
        Command::Save => {
            match task::block_on(save::save(&config.archive_dirpath)) {
                Ok(_) => {
                    let message = format!(
                        "✅ sessions persisted to {}",
                        config.archive_dirpath.to_str().unwrap_or("???")
                    );
                    display_message(&message)
                        .expect("Cannot communicate with Tmux for displaying message")
                }
                Err(e) => println!("An error ocurred: {}", e),
            };
        }
        Command::Restore => unimplemented!(),
    }
}
