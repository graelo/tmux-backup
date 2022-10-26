use std::{io, process::Output};

/// Describes all errors variants from this crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A tmux invocation returned some output where none was expected (actions such as
    /// some `tmux display-message` invocations).
    #[error(
        "unexpected process output: intent: `{intent}`, stdout: `{stdout}`, stderr: `{stderr}`"
    )]
    UnexpectedTmuxOutput {
        intent: &'static str,
        stdout: String,
        stderr: String,
    },

    /// Indicates Tmux has a weird config, like missing the `"default-shell"`.
    #[error("unexpected tmux config: `{0}`")]
    TmuxConfig(&'static str),

    /// Some parsing error.
    #[error("failed parsing: `{intent}`")]
    ParseError {
        desc: &'static str,
        intent: &'static str,
        err: nom::Err<nom::error::Error<String>>,
    },

    /// Failed parsing the output of a process invocation as utf-8.
    #[error("failed parsing utf-8 string: `{source}`")]
    Utf8 {
        #[from]
        /// Source error.
        source: std::string::FromUtf8Error,
    },

    /// Some IO error.
    #[error("failed with io: `{source}`")]
    Io {
        #[from]
        /// Source error.
        source: io::Error,
    },
}

/// Convert a nom error into an owned error and add the parsing intent.
///
/// # Errors
///
/// This maps to a `Error::ParseError`.
#[must_use]
pub fn map_add_intent(
    desc: &'static str,
    intent: &'static str,
    nom_err: nom::Err<nom::error::Error<&str>>,
) -> Error {
    Error::ParseError {
        desc,
        intent,
        err: nom_err.to_owned(),
    }
}

/// Ensure that the output's stdout and stderr are empty, indicating
/// the command had succeeded.
///
/// # Errors
///
/// Returns a `Error::UnexpectedTmuxOutput` in case .
pub fn check_empty_process_output(
    output: &Output,
    intent: &'static str,
) -> std::result::Result<(), Error> {
    if !output.stdout.is_empty() || !output.stderr.is_empty() {
        let stdout = String::from_utf8_lossy(&output.stdout[..]).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr[..]).to_string();
        return Err(Error::UnexpectedTmuxOutput {
            intent,
            stdout,
            stderr,
        });
    }
    Ok(())
}
