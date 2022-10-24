//! This module provides a few types and functions to handle Tmux Panes.
//!
//! The main use cases are running Tmux commands & parsing Tmux panes
//! information.

use std::path::PathBuf;
use std::str::FromStr;

use async_std::process::Command;
use nom::{
    character::complete::{char, digit1, not_line_ending},
    combinator::{all_consuming, map_res},
    sequence::tuple,
    IResult,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::{check_empty_process_output, map_add_intent, Error},
    pane_id::{parse::pane_id, PaneId},
    parse::{boolean, quoted_nonempty_string},
    utils::SliceExt,
    window_id::WindowId,
    Result,
};

/// A Tmux pane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pane {
    /// Pane identifier, e.g. `%37`.
    pub id: PaneId,
    /// Describes the Pane index in the Window
    pub index: u16,
    /// Describes if the pane is currently active (focused).
    pub is_active: bool,
    /// Title of the Pane (usually defaults to the hostname)
    pub title: String,
    /// Current dirpath of the Pane
    pub dirpath: PathBuf,
    /// Current command executed in the Pane
    pub command: String,
}

impl FromStr for Pane {
    type Err = Error;

    /// Parse a string containing tmux panes status into a new `Pane`.
    ///
    /// This returns a `Result<Pane, Error>` as this call can obviously
    /// fail if provided an invalid format.
    ///
    /// The expected format of the tmux status is
    ///
    /// ```text
    /// %20:0:false:'rmbp':'nvim':/Users/graelo/code/rust/tmux-backup
    /// %21:1:true:'rmbp':'tmux':/Users/graelo/code/rust/tmux-backup
    /// %27:2:false:'rmbp':'man man':/Users/graelo/code/rust/tmux-backup
    /// ```
    ///
    /// This status line is obtained with
    ///
    /// ```text
    /// tmux list-panes -F "#{pane_id}:#{pane_index}:#{?pane_active,true,false}:'#{pane_title}':'#{pane_current_command}':#{pane_current_path}"
    /// ```
    ///
    /// For definitions, look at `Pane` type and the tmux man page for
    /// definitions.
    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        let desc = "Pane";
        let intent = "#{pane_id}:#{pane_index}:#{?pane_active,true,false}:'#{pane_title}':'#{pane_current_command}':#{pane_current_path}";

        let (_, pane) =
            all_consuming(parse::pane)(input).map_err(|e| map_add_intent(desc, intent, e))?;

        Ok(pane)
    }
}

impl Pane {
    /// Return the entire Pane content as a `Vec<u8>`.
    ///
    /// # Note
    ///
    /// The output contains the escape codes, joined lines, and all lines are trimmed after capture
    /// because tmux does not allow that. In addition, the last line has an additional ascii reset
    /// escape code because tmux does not capture it.
    ///
    /// If `drop_n_last_lines` is greater than 0, the n last line are not captured. This is used
    /// only for panes with a zsh prompt, in order to avoid polluting the history with new prompts
    /// on restore.
    ///
    pub async fn capture(&self, drop_n_last_lines: usize) -> Result<Vec<u8>> {
        let args = vec![
            "capture-pane",
            "-t",
            self.id.as_str(),
            "-J", // preserves trailing spaces & joins any wrapped lines
            "-e", // include escape sequences for text & background
            "-p", // output goes to stdout
            "-S", // starting line number
            "-",  // start of history
            "-E", // ending line number
            "-",  // end of history
        ];

        let output = Command::new("tmux").args(&args).output().await?;

        let mut trimmed_lines: Vec<&[u8]> = output
            .stdout
            .split(|c| *c == b'\n')
            .map(|line| line.trim_trailing())
            .collect();

        trimmed_lines.truncate(trimmed_lines.len() - drop_n_last_lines);

        // Join the lines with `b'\n'`, add reset code to the last line
        let mut output_trimmed: Vec<u8> = Vec::with_capacity(output.stdout.len());
        for (idx, &line) in trimmed_lines.iter().enumerate() {
            output_trimmed.extend_from_slice(line);
            if idx != trimmed_lines.len() - 1 {
                output_trimmed.push(b'\n');
            } else {
                let reset = "\u{001b}[0m".as_bytes();
                output_trimmed.extend_from_slice(reset);
            }
        }

        Ok(output_trimmed)
    }
}

pub(crate) mod parse {
    use super::*;

    pub(crate) fn pane(input: &str) -> IResult<&str, Pane> {
        let (input, (id, _, index, _, is_active, _, title, _, command, _, dirpath)) =
            tuple((
                pane_id,
                char(':'),
                map_res(digit1, str::parse),
                char(':'),
                boolean,
                char(':'),
                quoted_nonempty_string,
                char(':'),
                quoted_nonempty_string,
                char(':'),
                not_line_ending,
            ))(input)?;

        Ok((
            input,
            Pane {
                id,
                index,
                is_active,
                title: title.into(),
                dirpath: dirpath.into(),
                command: command.into(),
            },
        ))
    }
}

// ------------------------------
// Ops
// ------------------------------

/// Return a list of all `Pane` from all sessions.
pub async fn available_panes() -> Result<Vec<Pane>> {
    let args = vec![
        "list-panes",
        "-a",
        "-F",
        "#{pane_id}\
        :#{pane_index}\
        :#{?pane_active,true,false}\
        :'#{pane_title}'\
        :'#{pane_current_command}'\
        :#{pane_current_path}",
    ];

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;

    // Each call to `Pane::parse` returns a `Result<Pane, _>`. All results
    // are collected into a Result<Vec<Pane>, _>, thanks to `collect()`.
    let result: Result<Vec<Pane>> = buffer
        .trim_end() // trim last '\n' as it would create an empty line
        .split('\n')
        .map(Pane::from_str)
        .collect();

    result
}

/// Create a new pane (horizontal split) in the window with `window_id`, and return the new
/// pane id.
pub async fn new_pane(
    reference_pane: &Pane,
    pane_command: Option<&str>,
    window_id: &WindowId,
) -> Result<PaneId> {
    let mut args = vec![
        "split-window",
        "-h",
        "-c",
        reference_pane.dirpath.to_str().unwrap(),
        "-t",
        window_id.as_str(),
        "-P",
        "-F",
        "#{pane_id}",
    ];
    if let Some(pane_command) = pane_command {
        args.push(pane_command);
    }

    let output = Command::new("tmux").args(&args).output().await?;
    let buffer = String::from_utf8(output.stdout)?;

    let new_id = PaneId::from_str(buffer.trim_end())?;
    Ok(new_id)
}

/// Select (make active) the pane with `pane_id`.
pub async fn select_pane(pane_id: &PaneId) -> Result<()> {
    let args = vec!["select-pane", "-t", pane_id.as_str()];

    let output = Command::new("tmux").args(&args).output().await?;
    check_empty_process_output(output, "select-pane")
}

#[cfg(test)]
mod tests {
    use super::Pane;
    use super::PaneId;
    use crate::Result;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn parse_list_panes() {
        let output = vec![
            "%20:0:false:'rmbp':'nvim':/Users/graelo/code/rust/tmux-backup",
            "%21:1:true:'graelo@server: ~':'tmux':/Users/graelo/code/rust/tmux-backup",
            "%27:2:false:'rmbp':'man man':/Users/graelo/code/rust/tmux-backup",
        ];
        let panes: Result<Vec<Pane>> = output.iter().map(|&line| Pane::from_str(line)).collect();
        let panes = panes.expect("Could not parse tmux panes");

        let expected = vec![
            Pane {
                id: PaneId::from_str("%20").unwrap(),
                index: 0,
                is_active: false,
                title: String::from("rmbp"),
                dirpath: PathBuf::from_str("/Users/graelo/code/rust/tmux-backup").unwrap(),
                command: String::from("nvim"),
            },
            Pane {
                id: PaneId(String::from("%21")),
                index: 1,
                is_active: true,
                title: String::from("graelo@server: ~"),
                dirpath: PathBuf::from_str("/Users/graelo/code/rust/tmux-backup").unwrap(),
                command: String::from("tmux"),
            },
            Pane {
                id: PaneId(String::from("%27")),
                index: 2,
                is_active: false,
                title: String::from("rmbp"),
                dirpath: PathBuf::from_str("/Users/graelo/code/rust/tmux-backup").unwrap(),
                command: String::from("man man"),
            },
        ];

        assert_eq!(panes, expected);
    }
}
