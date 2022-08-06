use async_std::task;
use clap::Parser;

use tmux_revive::{
    actions::save,
    config::{CatalogSubcommand, Command, Config, SubList},
    management::{Catalog, Plan},
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
            CatalogSubcommand::List { sublist } => {
                let Plan {
                    deletable,
                    retainable,
                } = catalog.plan();

                if let Some(sublist) = sublist {
                    match sublist {
                        SubList::Deletable => {
                            for backup_path in deletable.iter() {
                                println!("{}", backup_path.to_string_lossy());
                            }
                        }
                        SubList::Retainable => {
                            for backup_path in retainable.iter() {
                                println!("{}", backup_path.to_string_lossy());
                            }
                        }
                    }
                } else {
                    println!("Catalog");
                    println!("- location: `{}`:", &catalog.location());
                    println!("- strategy: {}", &catalog.strategy);

                    let reset = "\u{001b}[0m";
                    let magenta = "\u{001b}[35m";
                    let green = "\u{001b}[32m";

                    println!("- deletable:");
                    for backup_path in deletable.iter() {
                        println!("    {magenta}{}{reset}", backup_path.to_string_lossy());
                    }
                    println!("- keep:");
                    for backup_path in retainable.iter() {
                        println!("    {green}{}{reset}", backup_path.to_string_lossy());
                    }
                    println!(
                        "\n{} backups: {} retainable, {} deletable",
                        &catalog.size(),
                        retainable.len(),
                        deletable.len(),
                    );
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
