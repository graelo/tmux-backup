mod error;
mod tmux;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    println!("---- sessions ----");
    let sessions = tmux::session::available_sessions()?;
    for session in sessions {
        println!("{:?}", session);
    }

    println!("---- windows ----");
    let windows = tmux::available_windows()?;
    for window in windows {
        println!("{:?}", window);
    }

    println!("---- panes ----");
    let panes = tmux::pane::available_panes()?;
    for pane in panes {
        println!("{:?}", pane);
    }

    Ok(())
}
