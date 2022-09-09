//! This module provides a few types and functions to handle Tmux windows.
//!
//! The main use cases are running Tmux commands & parsing Tmux window information.

use std::str::FromStr;

use async_std::process::Command;

use nom::{
    character::complete::{char, digit1},
    combinator::{all_consuming, map_res, recognize},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::{check_empty_process_output, map_add_intent, Error},
    layout::{self, window_layout},
    pane::Pane,
    pane_id::{parse::pane_id, PaneId},
    parse::{boolean, quoted_nonempty_string},
    session::Session,
    window_id::{parse::window_id, WindowId},
    Result,
};

/// A Tmux window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Window {
    /// Window identifier, e.g. `@3`.
    pub id: WindowId,
    /// Index of the Window in the Session.
    pub index: u16,
    /// Describes whether the Window is active.
    pub is_active: bool,
    /// Describes how panes are laid out in the Window.
    pub layout: String,
    /// Name of the Window.
    pub name: String,
    /// Name of Sessions to which this Window is attached.
    pub sessions: Vec<String>,
}

impl FromStr for Window {
    type Err = Error;

    /// Parse a string containing the tmux window status into a new `Window`.
    ///
    /// This returns a `Result<Window, Error>` as this call can obviously
    /// fail if provided an invalid format.
    ///
    /// The expected format of the tmux status is
    ///
    /// ```text
    /// @1:0:true:035d,334x85,0,0{167x85,0,0,1,166x85,168,0[166x48,168,0,2,166x36,168,49,3]}:'ignite':'pytorch'
    /// @2:1:false:4438,334x85,0,0[334x41,0,0{167x41,0,0,4,166x41,168,0,5},334x43,0,42{167x43,0,42,6,166x43,168,42,7}]:'dates-attn':'pytorch'
    /// @3:2:false:9e8b,334x85,0,0{167x85,0,0,8,166x85,168,0,9}:'th-bits':'pytorch'
    /// @4:3:false:64ef,334x85,0,0,10:'docker-pytorch':'pytorch'
    /// @5:0:true:64f0,334x85,0,0,11:'ben':'rust'
    /// @6:1:false:64f1,334x85,0,0,12:'pyo3':'rust'
    /// @7:2:false:64f2,334x85,0,0,13:'mdns-repeater':'rust'
    /// @8:0:true:64f3,334x85,0,0,14:'combine':'swift'
    /// @9:0:false:64f4,334x85,0,0,15:'copyrat':'tmux-hacking'
    /// @10:1:false:ae3a,334x85,0,0[334x48,0,0,17,334x36,0,49{175x36,0,49,18,158x36,176,49,19}]:'mytui-app':'tmux-hacking'
    /// @11:2:true:e2e2,334x85,0,0{175x85,0,0,20,158x85,176,0[158x42,176,0,21,158x42,176,43,27]}:'tmux-backup':'tmux-hacking'
    /// ```
    ///
    /// This status line is obtained with
    ///
    /// ```text
    /// tmux list-windows -a -F "#{window_id}:#{window_index}:#{?window_active,true,false}:#{window_layout}:'#{window_name}':'#{window_linked_sessions_list}'"
    /// ```
    ///
    /// For definitions, look at `Window` type and the tmux man page for
    /// definitions.
    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        let desc = "Window";
        let intent = "#{window_id}:#{window_index}:#{?window_active,true,false}:#{window_layout}:'#{window_name}':'#{window_linked_sessions_list}'";

        let (_, window) =
            all_consuming(parse::window)(input).map_err(|e| map_add_intent(desc, intent, e))?;

        Ok(window)
    }
}

impl Window {
    /// Return all `PaneId` in this window.
    pub fn pane_ids(&self) -> Vec<PaneId> {
        let layout = layout::parse_window_layout(&self.layout).unwrap();
        layout.pane_ids().iter().map(PaneId::from).collect()
    }
}

pub(crate) mod parse {
    use super::*;

    pub(crate) fn window(input: &str) -> IResult<&str, Window> {
        let (input, (id, _, index, _, is_active, _, layout, _, name, _, session_names)) =
            tuple((
                window_id,
                char(':'),
                map_res(digit1, str::parse),
                char(':'),
                boolean,
                char(':'),
                recognize(window_layout),
                char(':'),
                quoted_nonempty_string,
                char(':'),
                quoted_nonempty_string,
            ))(input)?;

        Ok((
            input,
            Window {
                id,
                index,
                is_active,
                layout: layout.to_string(),
                name: name.to_string(),
                sessions: vec![session_names.to_string()],
            },
        ))
    }
}

// ------------------------------
// Ops
// ------------------------------

/// Return a list of all `Window` from all sessions.
pub async fn available_windows() -> Result<Vec<Window>> {
    let args = vec![
        "list-windows",
        "-a",
        "-F",
        "#{window_id}\
        :#{window_index}\
        :#{?window_active,true,false}\
        :#{window_layout}\
        :'#{window_name}'\
        :'#{window_linked_sessions_list}'",
    ];

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;

    // Note: each call to the `Window::from_str` returns a `Result<Window, _>`.
    // All results are then collected into a Result<Vec<Window>, _>, via
    // `collect()`.
    let result: Result<Vec<Window>> = buffer
        .trim_end() // trim last '\n' as it would create an empty line
        .split('\n')
        .map(Window::from_str)
        .collect();

    result
}

/// Create a Tmux window in a session exactly named as the passed `session`.
///
/// The new window attributes:
///
/// - created in the `session`
/// - the window name is taken from the passed `window`
/// - the working directory is the pane's working directory.
///
pub async fn new_window(
    session: &Session,
    window: &Window,
    pane: &Pane,
    pane_command: Option<&str>,
) -> Result<(WindowId, PaneId)> {
    let exact_session_name = format!("={}", session.name);

    let mut args = vec![
        "new-window",
        "-d",
        "-c",
        pane.dirpath.to_str().unwrap(),
        "-n",
        &window.name,
        "-t",
        &exact_session_name,
        "-P",
        "-F",
        "#{window_id}:#{pane_id}",
    ];
    if let Some(pane_command) = pane_command {
        args.push(pane_command);
    }

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;
    let buffer = buffer.trim_end();

    let desc = "new-window";
    let intent = "#{window_id}:#{pane_id}";

    let (_, (new_window_id, _, new_pane_id)) =
        all_consuming(tuple((window_id, char(':'), pane_id)))(buffer)
            .map_err(|e| map_add_intent(desc, intent, e))?;

    Ok((new_window_id, new_pane_id))
}

/// Apply the provided `layout` to the window with `window_id`.
pub async fn set_layout(layout: &str, window_id: &WindowId) -> Result<()> {
    let args = vec!["select-layout", "-t", window_id.as_str(), layout];

    let output = Command::new("tmux").args(&args).output().await?;
    check_empty_process_output(output, "select-layout")
}

/// Select (make active) the window with `window_id`.
pub async fn select_window(window_id: &WindowId) -> Result<()> {
    let args = vec!["select-window", "-t", window_id.as_str()];

    let output = Command::new("tmux").args(&args).output().await?;
    check_empty_process_output(output, "select-window")
}

#[cfg(test)]
mod tests {
    use super::Window;
    use super::WindowId;
    use crate::Result;
    use std::str::FromStr;

    #[test]
    fn parse_list_sessions() {
        let output = vec![
            "@1:0:true:035d,334x85,0,0{167x85,0,0,1,166x85,168,0[166x48,168,0,2,166x36,168,49,3]}:'ignite':'pytorch'",
            "@2:1:false:4438,334x85,0,0[334x41,0,0{167x41,0,0,4,166x41,168,0,5},334x43,0,42{167x43,0,42,6,166x43,168,42,7}]:'dates-attn':'pytorch'",
            "@3:2:false:9e8b,334x85,0,0{167x85,0,0,8,166x85,168,0,9}:'th-bits':'pytorch'",
            "@4:3:false:64ef,334x85,0,0,10:'docker-pytorch':'pytorch'",
            "@5:0:true:64f0,334x85,0,0,11:'ben':'rust'",
            "@6:1:false:64f1,334x85,0,0,12:'pyo3':'rust'",
            "@7:2:false:64f2,334x85,0,0,13:'mdns-repeater':'rust'",
            "@8:0:true:64f3,334x85,0,0,14:'combine':'swift'",
            "@9:0:false:64f4,334x85,0,0,15:'copyrat':'tmux-hacking'",
            "@10:1:false:ae3a,334x85,0,0[334x48,0,0,17,334x36,0,49{175x36,0,49,18,158x36,176,49,19}]:'mytui-app':'tmux-hacking'",
            "@11:2:true:e2e2,334x85,0,0{175x85,0,0,20,158x85,176,0[158x42,176,0,21,158x42,176,43,27]}:'tmux-backup':'tmux-hacking'",
        ];
        let sessions: Result<Vec<Window>> =
            output.iter().map(|&line| Window::from_str(line)).collect();
        let windows = sessions.expect("Could not parse tmux sessions");

        let expected = vec![
            Window {
                id: WindowId::from_str("@1").unwrap(),
                index: 0,
                is_active: true,
                layout: String::from(
                    "035d,334x85,0,0{167x85,0,0,1,166x85,168,0[166x48,168,0,2,166x36,168,49,3]}",
                ),
                name: String::from("ignite"),
                sessions: vec![String::from("pytorch")],
            },
            Window {
                id: WindowId::from_str("@2").unwrap(),
                index: 1,
                is_active: false,
                layout: String::from(
                    "4438,334x85,0,0[334x41,0,0{167x41,0,0,4,166x41,168,0,5},334x43,0,42{167x43,0,42,6,166x43,168,42,7}]",
                ),
                name: String::from("dates-attn"),
                sessions: vec![String::from("pytorch")],
            },
            Window {
                id: WindowId::from_str("@3").unwrap(),
                index: 2,
                is_active: false,
                layout: String::from(
                    "9e8b,334x85,0,0{167x85,0,0,8,166x85,168,0,9}",
                ),
                name: String::from("th-bits"),
                sessions: vec![String::from("pytorch")],
            },
            Window {
                id: WindowId::from_str("@4").unwrap(),
                index: 3,
                is_active: false,
                layout: String::from(
                    "64ef,334x85,0,0,10",
                ),
                name: String::from("docker-pytorch"),
                sessions: vec![String::from("pytorch")],
            },
            Window {
                id: WindowId::from_str("@5").unwrap(),
                index: 0,
                is_active: true,
                layout: String::from(
                    "64f0,334x85,0,0,11",
                ),
                name: String::from("ben"),
                sessions: vec![String::from("rust")],
            },
            Window {
                id: WindowId::from_str("@6").unwrap(),
                index: 1,
                is_active: false,
                layout: String::from(
                    "64f1,334x85,0,0,12",
                ),
                name: String::from("pyo3"),
                sessions: vec![String::from("rust")],
            },
            Window {
                id: WindowId::from_str("@7").unwrap(),
                index: 2,
                is_active: false,
                layout: String::from(
                    "64f2,334x85,0,0,13",
                ),
                name: String::from("mdns-repeater"),
                sessions: vec![String::from("rust")],
            },
            Window {
                id: WindowId::from_str("@8").unwrap(),
                index: 0,
                is_active: true,
                layout: String::from(
                    "64f3,334x85,0,0,14",
                ),
                name: String::from("combine"),
                sessions: vec![String::from("swift")],
            },
            Window {
                id: WindowId::from_str("@9").unwrap(),
                index: 0,
                is_active: false,
                layout: String::from(
                    "64f4,334x85,0,0,15",
                ),
                name: String::from("copyrat"),
                sessions: vec![String::from("tmux-hacking")],
            },
            Window {
                id: WindowId::from_str("@10").unwrap(),
                index: 1,
                is_active: false,
                layout: String::from(
                    "ae3a,334x85,0,0[334x48,0,0,17,334x36,0,49{175x36,0,49,18,158x36,176,49,19}]",
                ),
                name: String::from("mytui-app"),
                sessions: vec![String::from("tmux-hacking")],
            },
            Window {
                id: WindowId::from_str("@11").unwrap(),
                index: 2,
                is_active: true,
                layout: String::from(
                    "e2e2,334x85,0,0{175x85,0,0,20,158x85,176,0[158x42,176,0,21,158x42,176,43,27]}",
                ),
                name: String::from("tmux-backup"),
                sessions: vec![String::from("tmux-hacking")],
            },
        ];

        assert_eq!(windows, expected);
    }
}
