use nom::{
    branch::alt,
    bytes::complete::{escaped, tag},
    character::complete::none_of,
    combinator::value,
    sequence::delimited,
    IResult,
};

/// Return the `&str` between single quotes. The returned string may be empty.
#[allow(unused)]
pub(crate) fn quoted_string(input: &str) -> IResult<&str, &str> {
    let esc = escaped(none_of("\\\'"), '\\', tag("'"));
    let esc_or_empty = alt((esc, tag("")));

    delimited(tag("'"), esc_or_empty, tag("'"))(input)
}

/// Return the `&str` between single quotes. The returned string may not be empty.
pub(crate) fn quoted_nonempty_string(input: &str) -> IResult<&str, &str> {
    let esc = escaped(none_of("\\\'"), '\\', tag("'"));
    delimited(tag("'"), esc, tag("'"))(input)
}

/// Return a bool: allowed values: `"true"` or `"false"`.
pub(crate) fn boolean(input: &str) -> IResult<&str, bool> {
    // This is a parser that returns `true` if it sees the string "true", and
    // an error otherwise.
    let parse_true = value(true, tag("true"));

    let parse_false = value(false, tag("false"));

    alt((parse_true, parse_false))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quoted_nonempty_string() {
        let (input, res) = quoted_nonempty_string(r#"'foo\' 🤖 bar'"#).unwrap();
        assert!(input.is_empty());
        assert_eq!(res, r#"foo\' 🤖 bar"#);
        let (input, res) = quoted_nonempty_string("'λx → x'").unwrap();
        assert!(input.is_empty());
        assert_eq!(res, "λx → x");
        let (input, res) = quoted_nonempty_string("'  '").unwrap();
        assert!(input.is_empty());
        assert_eq!(res, "  ");

        assert!(quoted_nonempty_string("''").is_err());
    }

    #[test]
    fn test_quoted_string() {
        let (input, res) = quoted_string("''").unwrap();
        assert!(input.is_empty());
        assert!(res.is_empty());
    }
}
