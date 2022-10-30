/// Misc utilities.

pub(crate) trait SliceExt {
    fn trim(&self) -> &Self;
    fn trim_trailing(&self) -> &Self;
}

fn is_whitespace(c: &u8) -> bool {
    *c == b'\t' || *c == b' '
}

fn is_not_whitespace(c: &u8) -> bool {
    !is_whitespace(c)
}

impl SliceExt for [u8] {
    /// Trim leading and trailing whitespaces (`\t` and ` `) in a `&[u8]`
    fn trim(&self) -> &[u8] {
        if let Some(first) = self.iter().position(is_not_whitespace) {
            if let Some(last) = self.iter().rposition(is_not_whitespace) {
                &self[first..last + 1]
            } else {
                unreachable!();
            }
        } else {
            &[]
        }
    }

    /// Trim trailing whitespaces (`\t` and ` `) in a `&[u8]`
    fn trim_trailing(&self) -> &[u8] {
        if let Some(last) = self.iter().rposition(is_not_whitespace) {
            &self[0..last + 1]
        } else {
            &[]
        }
    }
}

/// Trim each line of the buffer.
fn buf_trim_trailing(buf: &[u8]) -> Vec<&[u8]> {
    let trimmed_lines: Vec<&[u8]> = buf
        .split(|c| *c == b'\n')
        .map(SliceExt::trim_trailing) // trim each line
        .collect();

    trimmed_lines
}

/// Drop all the last empty lines.
fn drop_last_empty_lines<'a>(lines: &[&'a [u8]]) -> Vec<&'a [u8]> {
    if let Some(last) = lines.iter().rposition(|line| !line.is_empty()) {
        lines[0..=last].to_vec()
    } else {
        lines.to_vec()
    }
}

/// This function processes a pane captured bufer.
///
/// - All lines are trimmed after capture because tmux does not allow capturing escape codes and
///   trimming lines.
/// - If `drop_n_last_lines` is greater than 0, the n last line are not captured. This is used only
///   for panes with a zsh prompt, in order to avoid polluting the history with new prompts on
///   restore.
/// - In addition, the last line has an additional ascii reset escape code because tmux does not
///   capture it.
///
pub fn cleanup_captured_buffer(buffer: &[u8], drop_n_last_lines: usize) -> Vec<u8> {
    let trimmed_lines: Vec<&[u8]> = buf_trim_trailing(buffer);
    let mut buffer: Vec<&[u8]> = drop_last_empty_lines(&trimmed_lines);
    buffer.truncate(buffer.len() - drop_n_last_lines);

    // Join the lines with `b'\n'`, add reset code to the last line
    let mut final_buffer: Vec<u8> = Vec::with_capacity(buffer.len());
    for (idx, &line) in buffer.iter().enumerate() {
        final_buffer.extend_from_slice(line);

        let is_last_line = idx == buffer.len() - 1;
        if is_last_line {
            let reset = "\u{001b}[0m".as_bytes();
            final_buffer.extend_from_slice(reset);
            final_buffer.push(b'\n');
        } else {
            final_buffer.push(b'\n');
        }
    }

    final_buffer
}

#[cfg(test)]
mod tests {
    use super::{buf_trim_trailing, drop_last_empty_lines, SliceExt};

    #[test]
    fn trims_trailing_whitespaces() {
        let input = "  text   ".as_bytes();
        let expected = "  text".as_bytes();

        let actual = input.trim_trailing();
        assert_eq!(actual, expected);
    }

    #[test]
    fn trims_whitespaces() {
        let input = "  text   ".as_bytes();
        let expected = "text".as_bytes();

        let actual = input.trim();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_buf_trim_trailing() {
        let text = "line1\n\nline3   ";
        let actual = buf_trim_trailing(text.as_bytes());
        let expected = vec!["line1".as_bytes(), "".as_bytes(), "line3".as_bytes()];
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_buf_drop_last_empty_lines() {
        let text = "line1\nline2\n\nline3   ";

        let trimmed_lines = buf_trim_trailing(text.as_bytes());
        let actual = drop_last_empty_lines(&trimmed_lines);
        let expected = trimmed_lines;
        assert_eq!(actual, expected);

        //

        let text = "line1\nline2\n\n\n     ";

        let trimmed_lines = buf_trim_trailing(text.as_bytes());
        let actual = drop_last_empty_lines(&trimmed_lines);
        let expected = vec!["line1".as_bytes(), "line2".as_bytes()];
        assert_eq!(actual, expected);
    }
}
