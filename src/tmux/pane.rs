//! This module provides types and functions to use Tmux.
//!
//! The main use cases are running Tmux commands & parsing Tmux panes
//! information.

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use crate::error::ParseError;

#[derive(Debug, PartialEq)]
pub struct Pane {
    /// Pane identifier, e.g. `%37`.
    pub id: PaneId,
    /// Describes the Pane index in the Window
    pub index: u16,
    /// Describes if the pane is currently active (focused).
    pub is_active: bool,
    /// Number of columns in the pane.
    pub width: u16,
    /// Number of lines in the pane.
    pub height: u16,
    /// Position (column) of the left of the Pane
    pub pos_left: u16,
    /// Position (column) of the right of the Pane
    pub pos_right: u16,
    /// Position (row) of the top of the Pane
    pub pos_top: u16,
    /// Position (row) of the bottom of the Pane
    pub pos_bottom: u16,
    /// Title of the Pane (usually defaults to the hostname)
    pub title: String,
    /// Current path of the Pane
    pub path: PathBuf,
    /// Current command executed in the Pane
    pub command: String,
}

impl FromStr for Pane {
    type Err = ParseError;

    /// Parse a string containing tmux panes status into a new `Pane`.
    ///
    /// This returns a `Result<Pane, ParseError>` as this call can obviously
    /// fail if provided an invalid format.
    ///
    /// The expected format of the tmux status is
    ///
    /// ```text
    /// %20:0:false:175:85:0:174:0:84:rmbp:/Users/graelo/Travail/code/rust/tmux-revive:nvim
    /// %21:1:true:158:42:176:333:0:41:rmbp:/Users/graelo/Travail/code/rust/tmux-revive:tmux
    /// %27:2:false:158:42:176:333:43:84:rmbp:/Users/graelo/Travail/code/rust/tmux-revive:man
    /// ```
    ///
    /// This status line is obtained with
    ///
    /// ```text
    /// tmux list-panes -F "#{pane_id}:#{pane_index}:#{?pane_active,true,false}:#{pane_width}:#{pane_height}:#{pane_left}:#{pane_right}:#{pane_top}:#{pane_bottom}:#{pane_title}:#{pane_current_path}:#{pane_current_command}"`.
    /// ```
    ///
    /// For definitions, look at `Pane` type and the tmux man page for
    /// definitions.
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = src.split(':').collect();
        assert_eq!(
            items.len(),
            12,
            "tmux should have returned 12 items per line"
        );

        let mut iter = items.iter();

        // Pane id must be start with '%' followed by a `u32`
        let id_str = iter.next().unwrap();
        let id = PaneId::from_str(id_str)?;

        let index = iter.next().unwrap().parse::<u16>()?;

        let is_active = iter.next().unwrap().parse::<bool>()?;

        let width = iter.next().unwrap().parse::<u16>()?;
        let height = iter.next().unwrap().parse::<u16>()?;

        let pos_left = iter.next().unwrap().parse::<u16>()?;
        let pos_right = iter.next().unwrap().parse::<u16>()?;
        let pos_top = iter.next().unwrap().parse::<u16>()?;
        let pos_bottom = iter.next().unwrap().parse::<u16>()?;

        let title = iter.next().unwrap().to_string();

        let path = PathBuf::from(iter.next().unwrap());
        let command = iter.next().unwrap().to_string();

        Ok(Pane {
            id,
            index,
            is_active,
            width,
            height,
            pos_left,
            pos_right,
            pos_top,
            pos_bottom,
            title,
            path,
            command,
        })
    }
}

impl Pane {
    /// Returns the entire Pane content as a `String`.
    ///
    /// The provided `region` specifies if the visible area is captured, or the
    /// entire history.
    ///
    /// # Note
    ///
    /// In Tmux, the start line is the line at the top of the pane. The end line
    /// is the last line at the bottom of the pane.
    ///
    /// - In normal mode, the index of the start line is always 0. The index of
    /// the end line is always the pane's height minus one. These do not need to
    /// be specified when capturing the pane's content.
    ///
    /// - If navigating history in copy mode, the index of the start line is the
    /// opposite of the pane's scroll position. For instance a pane of 40 lines,
    /// scrolled up by 3 lines. It is necessarily in copy mode. Its start line
    /// index is `-3`. The index of the last line is `(40-1) - 3 = 36`.
    ///
    pub fn capture_pane(&self) -> Result<String, ParseError> {
        let args = vec![
            "capture-pane",
            "-t",
            self.id.as_str(),
            "-J",
            "-p",
            "-S",
            "-",
            "-E",
            "-",
        ];

        let output = duct::cmd("tmux", &args).read()?;
        Ok(output)
    }
}

#[derive(Debug, PartialEq)]
pub struct PaneId(String);

impl FromStr for PaneId {
    type Err = ParseError;

    /// Parse into PaneId. The `&str` must be start with '%'
    /// followed by a `u32`.
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        if !src.starts_with('%') {
            return Err(ParseError::ExpectedPaneIdMarker);
        }
        let id = src[1..].parse::<u32>()?;
        let id = format!("%{}", id);
        Ok(PaneId(id))
    }
}

impl PaneId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PaneId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Returns a list of `Pane` from the current tmux session.
pub fn list_panes() -> Result<Vec<Pane>, ParseError> {
    let args = vec![
        "list-panes",
        "-F",
        "#{pane_id}:\
            #{pane_index}\
            :#{?pane_active,true,false}\
            :#{pane_width}:#{pane_height}\
            :#{pane_left}:#{pane_right}:#{pane_top}:#{pane_bottom}\
            :#{pane_title}\
            :#{pane_current_path}\
            :#{pane_current_command}",
    ];

    let output = duct::cmd("tmux", &args).read()?;

    // Each call to `Pane::parse` returns a `Result<Pane, _>`. All results
    // are collected into a Result<Vec<Pane>, _>, thanks to `collect()`.
    let result: Result<Vec<Pane>, ParseError> = output
        .trim_end() // trim last '\n' as it would create an empty line
        .split('\n')
        .map(|line| Pane::from_str(line))
        .collect();

    result
}

#[cfg(test)]
mod tests {
    use super::Pane;
    use super::PaneId;
    use crate::error;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn parse_list_panes() {
        let output = vec![
            "%20:0:false:175:85:0:174:0:84:rmbp:/Users/graelo/Travail/code/rust/tmux-revive:nvim",
            "%21:1:true:158:42:176:333:0:41:rmbp:/Users/graelo/Travail/code/rust/tmux-revive:tmux",
            "%27:2:false:158:42:176:333:43:84:rmbp:/Users/graelo/Travail/code/rust/tmux-revive:man",
        ];
        let panes: Result<Vec<Pane>, error::ParseError> =
            output.iter().map(|&line| Pane::from_str(line)).collect();
        let panes = panes.expect("Could not parse tmux panes");

        let expected = vec![
            Pane {
                id: PaneId::from_str("%20").unwrap(),
                index: 0,
                is_active: false,
                width: 175,
                height: 85,
                pos_left: 0,
                pos_right: 174,
                pos_top: 0,
                pos_bottom: 84,
                title: String::from("rmbp"),
                path: PathBuf::from_str("/Users/graelo/Travail/code/rust/tmux-revive").unwrap(),
                command: String::from("nvim"),
            },
            Pane {
                id: PaneId(String::from("%21")),
                index: 1,
                is_active: true,
                width: 158,
                height: 42,
                pos_left: 176,
                pos_right: 333,
                pos_top: 0,
                pos_bottom: 41,
                title: String::from("rmbp"),
                path: PathBuf::from_str("/Users/graelo/Travail/code/rust/tmux-revive").unwrap(),
                command: String::from("tmux"),
            },
            Pane {
                id: PaneId(String::from("%27")),
                index: 2,
                is_active: false,
                width: 158,
                height: 42,
                pos_left: 176,
                pos_right: 333,
                pos_top: 43,
                pos_bottom: 84,
                title: String::from("rmbp"),
                path: PathBuf::from_str("/Users/graelo/Travail/code/rust/tmux-revive").unwrap(),
                command: String::from("man"),
            },
        ];

        assert_eq!(panes, expected);
    }
}
