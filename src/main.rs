use std::env;
use std::path::PathBuf;

use async_std::task;

use tmux_revive::save;

struct Opts {
    /// Directory where to save the archive.
    ///
    /// The archive name will be `archive-20220531T123456.tar.zst`, located under that path.
    archive_dirpath: PathBuf,
}

fn main() {
    // Config
    //
    let archive_dirpath: PathBuf = {
        let state_home = match env::var("XDG_STATE_HOME") {
            Ok(v) => PathBuf::from(v),
            Err(_) => match env::var("HOME") {
                Ok(v) => PathBuf::from(v).join(".local").join("state"),
                Err(_) => PathBuf::from("/tmp").join("state"),
            },
        };
        state_home.join("tmux-revive")
    };
    let opts = Opts { archive_dirpath };

    match task::block_on(save::save(&opts.archive_dirpath)) {
        Ok(_) => println!("✅ sessions persisted."),
        Err(e) => println!("An error ocurred: {}", e),
    };
}
