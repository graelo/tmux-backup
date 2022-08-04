use std::fmt;

/// Report the number of sessions, windows and panes.
///
/// This report is displayed after the commands `save`, `restore`, or `describe`.
#[derive(Debug)]
pub struct Report {
    /// Number of sessions in an archive.
    pub num_sessions: u16,

    /// Number of windows in an archive.
    pub num_windows: u16,

    /// Number of panes in an archive.
    pub num_panes: u16,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "✅ {} sessions ({} windows, {} panes)",
            self.num_sessions, self.num_windows, self.num_panes,
        ))
    }
}
