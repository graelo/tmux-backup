mod error;
mod tmux;

use futures::future::join_all;
use std::env;
use tokio::fs;

// Just a generic Result type to ease error handling for us. Errors in multithreaded
// async contexts needs some extra restrictions
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn app() -> Result<()> {
    let output_dir = env::temp_dir().join("tmux-revive");
    println!("{:?}", output_dir);
    fs::create_dir_all(&output_dir).await?;

    let mut handles = Vec::new();

    let handle = tokio::spawn(async move {
        println!("---- sessions ----");
        let sessions = tmux::session::available_sessions().await.unwrap();
        for session in sessions {
            println!("{:?}", session);
        }
        // Ok(())
    });
    handles.push(handle);

    let handle = tokio::spawn(async move {
        println!("---- windows ----");
        let windows = tmux::window::available_windows().await.unwrap();
        for window in windows {
            println!("{:?}", window);
        }
        // Ok(())
    });
    handles.push(handle);

    println!("---- panes ----");
    let panes_output_dir = output_dir.join("panes");
    fs::create_dir_all(&panes_output_dir).await?;
    let panes = tmux::pane::available_panes().await?;
    save_panes_content(panes, panes_output_dir).await?;

    Ok(())
}

async fn save_panes_content(
    panes: Vec<tmux::pane::Pane>,
    panes_output_dir: std::path::PathBuf,
) -> Result<()> {
    let mut handles = Vec::new();

    for pane in panes {
        let tmp_dir = panes_output_dir.clone();
        let handle = tokio::spawn(async move {
            let output = pane.capture().await.unwrap();

            let filename = format!("pane-{}.txt", pane.id);
            let filepath = tmp_dir.join(filename);
            fs::write(filepath, output).await
        });
        handles.push(handle);
    }
    join_all(handles).await;

    Ok(())
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    match rt.block_on(app()) {
        Ok(_) => println!("✅ sessions persisted."),
        Err(e) => println!("An error ocurred: {}", e),
    };
}
