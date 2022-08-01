mod error;
mod tmux;

use async_std::{fs, task};
use futures::future::join_all;
use std::env;
use std::path::PathBuf;

const PANES_DIR_NAME: &str = "panes-content";

// Just a generic Result type to ease error handling for us. Errors in multithreaded
// async contexts needs some extra restrictions
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn app() -> Result<()> {
    let output_dir = env::temp_dir().join("tmux-revive");
    // println!("{:?}", output_dir);
    fs::create_dir_all(&output_dir).await?;

    let mut handles = Vec::new();

    let handle = task::spawn(async move {
        println!("---- sessions ----");
        let sessions = tmux::session::available_sessions().await.unwrap();
        for session in sessions {
            println!("{:?}", session);
        }
        // Ok(())
    });
    handles.push(handle);

    let handle = task::spawn(async move {
        println!("---- windows ----");
        let windows = tmux::window::available_windows().await.unwrap();
        for window in windows {
            println!("{:?}", window);
        }
        // Ok(())
    });
    handles.push(handle);

    println!("---- panes ----");

    let panes_archive_filepath = output_dir.join(format!("{PANES_DIR_NAME}.tar.zst"));
    let panes_content_dir = output_dir.join(PANES_DIR_NAME);
    fs::create_dir_all(&panes_content_dir).await?;

    let panes = tmux::pane::available_panes().await?;
    save_panes_content(panes, panes_content_dir.clone()).await?;
    compress_panes_content(panes_archive_filepath, panes_content_dir.clone())?;
    std::fs::remove_dir_all(panes_content_dir)?;

    Ok(())
}

async fn save_panes_content(panes: Vec<tmux::pane::Pane>, panes_output_dir: PathBuf) -> Result<()> {
    let mut handles = Vec::new();

    for pane in panes {
        let tmp_dir = panes_output_dir.clone();
        let handle = task::spawn(async move {
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

fn compress_panes_content(output_filepath: PathBuf, panes_output_dir: PathBuf) -> Result<()> {
    println!("compressing content of {:?}", panes_output_dir);
    let archive = std::fs::File::create(output_filepath)?;
    let enc = zstd::stream::write::Encoder::new(archive, 0)?.auto_finish();
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all(PANES_DIR_NAME, panes_output_dir)?;
    tar.finish()?;

    Ok(())
}

fn main() {
    match task::block_on(app()) {
        Ok(_) => println!("✅ sessions persisted."),
        Err(e) => println!("An error ocurred: {}", e),
    };
}
