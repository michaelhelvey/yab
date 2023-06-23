use miette::Result;
use serde::Serialize;

use super::{
    code_iter::{current_span_error, previous_span_error, CodeIter, Span},
    punctuation::{Punctuation, PunctuationType},
    utils::is_line_terminator,
    Token,
};

/// Represents a regex literal token.  Since we're not actually parsing the
/// regex, or evaluating it, we don't need to parse the pattern, just the
/// pattern + the flags so that we can compile the literal into a function call
/// later if we want.
#[derive(Debug, Serialize, PartialEq)]
pub struct RegexLiteral {
    pattern: String,
    flags: String,
}

impl RegexLiteral {
    pub fn new(pattern: String, flags: String) -> Self {
        Self { pattern, flags }
    }
}

/// Parses a regex pattern, assuming that the leading '/' has been consumed.
/// Consumes the trailing '/' and returns the string in between as a pattern.
/// Does not parse escape sequences, as the runtime RegEx engine will handle
/// that.
fn parse_regex_pattern(chars: &mut CodeIter) -> Result<String> {
    let start_pos = chars.current_position();
    let mut lexeme = String::new();

    for next_char in chars.by_ref() {
        match next_char {
            '/' => return Ok(lexeme),
            c if is_line_terminator(c) => {
                return Err(previous_span_error!(
                    chars,
                    start_pos,
                    "Unexpected line terminator while parsing regular expression",
                ))
            }
            c => lexeme.push(c),
        }
    }

    Err(previous_span_error!(
        chars,
        start_pos,
        "Unexpected EOF while parsing regular expression",
    ))
}

fn parse_regex_flags(chars: &mut CodeIter) -> Result<String> {
    let mut lexeme = String::new();

    while let Some(next_char) = chars.peek() {
        match next_char {
            'g' | 'i' | 'm' | 's' | 'u' | 'y' => {
                lexeme.push(*next_char);
                _ = chars.next();
            }
            c if c.is_whitespace() => return Ok(lexeme),
            ';' => return Ok(lexeme),
            c if c.is_alphabetic() => {
                return Err(current_span_error!(
                    chars,
                    chars.current_position(),
                    "Invalid regular expression flag '{}'",
                    c
                ))
            }
            _c => return Ok(lexeme),
        }
    }

    Ok(lexeme)
}

/// Attempts to parse a regex literal (e.g. "/foo/g").
///
/// Returns:
///
/// * `Ok(Some(RegexLiteral))` if a regex literal was parsed.
///
/// * `Ok(None)` if the next characters are not a regex literal.
///
/// * `Err` if an error occurred while parsing (e.g. if an invalid character or
/// escape is encountered).
///
/// Note: this function is fairly naive about the difference between regex
/// literals and comments, (e.g. /{pattern/ vs "//"}), so it assumes that the
/// lexer tries to parse comments higher up in the loop.
pub fn try_parse_regex_literal(
    chars: &mut CodeIter,
    previous_token: Option<&Token>,
) -> Result<Option<RegexLiteral>> {
    // regex literals can only appear in expression contexts, so if the previous
    // token was not a operator or punctuator we can safely assume that we must
    // parse as division or something else.

    match previous_token {
        Some(Token::Operator(_))
        | Some(Token::Punctuation(Punctuation {
            kind: PunctuationType::Comma,
            ..
        }))
        | None => match chars.peek() {
            Some('/') => {
                _ = chars.next();
                let pattern = parse_regex_pattern(chars)?;
                let flags = parse_regex_flags(chars)?;

                Ok(Some(RegexLiteral { pattern, flags }))
            }
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::code_iter::IntoCodeIterator;

    use super::*;

    #[test]
    fn test_try_parse_regex_literal() {
        let mut chars = "/foo/g".into_code_iterator("script.js".to_string());
        let result = try_parse_regex_literal(&mut chars, None).unwrap().unwrap();
        assert_eq!(
            result,
            RegexLiteral {
                pattern: "foo".to_string(),
                flags: "g".to_string(),
            }
        );
    }

    #[test]
    fn test_regex_without_flags() {
        let mut chars = "/foo/".into_code_iterator("script.js".to_string());
        let result = try_parse_regex_literal(&mut chars, None).unwrap().unwrap();
        assert_eq!(
            result,
            RegexLiteral {
                pattern: "foo".to_string(),
                flags: "".to_string(),
            }
        );
    }

    #[test]
    fn test_regex_with_invalid_flags() {
        let mut chars = "/foo/z".into_code_iterator("script.js".to_string());
        let result = try_parse_regex_literal(&mut chars, None);

        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid regular expression flag 'z'"));
    }

    #[test]
    fn test_regex_with_unexpected_line_break() {
        let mut chars = "/foo\n/z".into_code_iterator("script.js".to_string());
        let result = try_parse_regex_literal(&mut chars, None);

        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unexpected line terminator while parsing regular expression"));
    }

    #[test]
    fn test_regex_flags_do_not_eat_next_chars() {
        let mut chars = "/foo/g.".into_code_iterator("script.js".to_string());
        let result = try_parse_regex_literal(&mut chars, None).unwrap().unwrap();
        assert_eq!(
            result,
            RegexLiteral {
                pattern: "foo".to_string(),
                flags: "g".to_string(),
            }
        );
        assert_eq!(chars.next(), Some('.'));
    }
}
