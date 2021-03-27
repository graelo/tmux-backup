pub mod pane;
pub mod session;
pub mod window;

use std::str::FromStr;

use crate::error;
use session::Session;
use window::Window;

/// Returns a list of all `Window` from all sessions.
pub fn available_windows() -> Result<Vec<Window>, error::ParseError> {
    let args = vec![
        "list-windows",
        "-a",
        "-F",
        "#{window_id}\
        :#{window_index}\
        :#{?window_active,true,false}\
        :#{window_layout}\
        :#{window_name}\
        :#{window_linked_sessions_list}",
    ];

    let output = duct::cmd("tmux", &args).read()?;

    // Each call to `Window::parse` returns a `Result<Window, _>`. All results
    // are collected into a Result<Vec<Window>, _>, thanks to `collect()`.
    let result: Result<Vec<Window>, error::ParseError> = output
        .trim_end() // trim last '\n' as it would create an empty line
        .split('\n')
        .map(|line| Window::from_str(line))
        .collect();

    result
}
