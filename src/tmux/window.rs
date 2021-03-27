//! This module provides a few types and functions to handle Tmux windows.
//!
//! The main use cases are running Tmux commands & parsing Tmux window
//! information.

use std::fmt;
use std::str::FromStr;

use crate::error::ParseError;

#[derive(Debug, PartialEq)]
pub struct Window {
    /// Window identifier, e.g. `@3`.
    pub id: WindowId,
    /// Index of the Window.
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

#[derive(Debug, PartialEq)]
pub struct WindowId(String);

impl FromStr for WindowId {
    type Err = ParseError;

    /// Parse into WindowId. The `&str` must start with '@' followed by a
    /// `u16`.
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        if !src.starts_with('@') {
            return Err(ParseError::ExpectedIdMarker('@'));
        }
        let id = src[1..].parse::<u16>()?;
        let id = format!("@{}", id);
        Ok(WindowId(id))
    }
}

impl WindowId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Window {
    type Err = ParseError;

    /// Parse a string containing the tmux window status into a new `Window`.
    ///
    /// This returns a `Result<Window, ParseError>` as this call can obviously
    /// fail if provided an invalid format.
    ///
    /// The expected format of the tmux status is
    ///
    /// ```text
    /// @1:0:true:035d,334x85,0,0{167x85,0,0,1,166x85,168,0[166x48,168,0,2,166x36,168,49,3]}:ignite:pytorch
    /// @2:1:false:4438,334x85,0,0[334x41,0,0{167x41,0,0,4,166x41,168,0,5},334x43,0,42{167x43,0,42,6,166x43,168,42,7}]:dates-attn:pytorch
    /// @3:2:false:9e8b,334x85,0,0{167x85,0,0,8,166x85,168,0,9}:th-bits:pytorch
    /// @4:3:false:64ef,334x85,0,0,10:docker-pytorch:pytorch
    /// @5:0:true:64f0,334x85,0,0,11:ben-williamson:rust
    /// @6:1:false:64f1,334x85,0,0,12:pyo3:rust
    /// @7:2:false:64f2,334x85,0,0,13:mdns-repeater:rust
    /// @8:0:true:64f3,334x85,0,0,14:combine:swift
    /// @9:0:false:64f4,334x85,0,0,15:copyrat:tmux-hacking
    /// @10:1:false:ae3a,334x85,0,0[334x48,0,0,17,334x36,0,49{175x36,0,49,18,158x36,176,49,19}]:mytui-app:tmux-hacking
    /// @11:2:true:e2e2,334x85,0,0{175x85,0,0,20,158x85,176,0[158x42,176,0,21,158x42,176,43,27]}:tmux-revive:tmux-hacking
    /// ```
    ///
    /// This status line is obtained with
    ///
    /// ```text
    /// tmux list-windows -a -F "#{window_id}:#{window_index}:#{?window_active,true,false}:#{window_layout}:#{window_name}:#{window_linked_sessions_list}"
    /// ```
    ///
    /// For definitions, look at `Window` type and the tmux man page for
    /// definitions.
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = src.split(':').collect();
        assert_eq!(items.len(), 6, "tmux should have returned 6 items per line");

        let mut iter = items.iter();

        // Window id must be start with '%' followed by a `u32`
        let id_str = iter.next().unwrap();
        let id = WindowId::from_str(id_str)?;

        let index = iter.next().unwrap().parse::<u16>()?;

        let is_active = iter.next().unwrap().parse::<bool>()?;

        let layout = iter.next().unwrap().to_string();

        let name = iter.next().unwrap().to_string();

        let session_names = iter.next().unwrap().to_string();
        let sessions = vec![session_names];

        Ok(Window {
            id,
            index,
            is_active,
            layout,
            name,
            sessions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Window;
    use super::WindowId;
    use crate::error;
    use std::str::FromStr;

    #[test]
    fn parse_list_sessions() {
        let output = vec![
            "@1:0:true:035d,334x85,0,0{167x85,0,0,1,166x85,168,0[166x48,168,0,2,166x36,168,49,3]}:ignite:pytorch",
            "@2:1:false:4438,334x85,0,0[334x41,0,0{167x41,0,0,4,166x41,168,0,5},334x43,0,42{167x43,0,42,6,166x43,168,42,7}]:dates-attn:pytorch",
            "@3:2:false:9e8b,334x85,0,0{167x85,0,0,8,166x85,168,0,9}:th-bits:pytorch",
            "@4:3:false:64ef,334x85,0,0,10:docker-pytorch:pytorch",
            "@5:0:true:64f0,334x85,0,0,11:ben:rust",
            "@6:1:false:64f1,334x85,0,0,12:pyo3:rust",
            "@7:2:false:64f2,334x85,0,0,13:mdns-repeater:rust",
            "@8:0:true:64f3,334x85,0,0,14:combine:swift",
            "@9:0:false:64f4,334x85,0,0,15:copyrat:tmux-hacking",
            "@10:1:false:ae3a,334x85,0,0[334x48,0,0,17,334x36,0,49{175x36,0,49,18,158x36,176,49,19}]:mytui-app:tmux-hacking",
            "@11:2:true:e2e2,334x85,0,0{175x85,0,0,20,158x85,176,0[158x42,176,0,21,158x42,176,43,27]}:tmux-revive:tmux-hacking",
        ];
        let sessions: Result<Vec<Window>, error::ParseError> =
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
                name: String::from("tmux-revive"),
                sessions: vec![String::from("tmux-hacking")],
            },
        ];

        assert_eq!(windows, expected);
    }
}
