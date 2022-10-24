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

#[cfg(test)]
mod tests {
    use super::SliceExt;

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
}
