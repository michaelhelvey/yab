use super::{
    code_iter::CodeIter,
    utils::{try_parse_from_prefix_lookup, HasPrefixLookup},
};
use serde::Serialize;
use strum_macros::EnumString;
use yab_parser_macros::HasPrefixLookup;

#[derive(Debug, Serialize, PartialEq, EnumString, HasPrefixLookup)]
pub enum PunctuationType {
    #[token(lexeme = ";")]
    #[strum(serialize = ";")]
    Semicolon,

    #[token(lexeme = ":")]
    #[strum(serialize = ":")]
    Colon,

    #[token(lexeme = "(")]
    #[strum(serialize = "(")]
    OpenParen,

    #[token(lexeme = ")")]
    #[strum(serialize = ")")]
    CloseParen,

    #[token(lexeme = "[")]
    #[strum(serialize = "[")]
    OpenBracket,

    #[token(lexeme = "]")]
    #[strum(serialize = "]")]
    CloseBracket,

    #[token(lexeme = "{")]
    #[strum(serialize = "{")]
    OpenBrace,

    #[token(lexeme = "}")]
    #[strum(serialize = "}")]
    CloseBrace,

    #[token(lexeme = ".")]
    #[strum(serialize = ".")]
    Dot,

    #[token(lexeme = ",")]
    #[strum(serialize = ",")]
    Comma,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Punctuation {
    pub kind: PunctuationType,
}

impl Punctuation {
    pub fn new(kind: PunctuationType) -> Self {
        Self { kind }
    }
}

pub fn try_parse_punctuation(chars: &mut CodeIter) -> Option<Punctuation> {
    try_parse_from_prefix_lookup::<PunctuationType>(chars).map(Punctuation::new)
}

#[cfg(test)]
mod tests {
    use crate::lexer::code_iter::IntoCodeIterator;

    use super::*;

    #[test]
    fn test_parse_punctuation() {
        let punctuators = vec![
            (";", PunctuationType::Semicolon),
            ("(", PunctuationType::OpenParen),
            (")", PunctuationType::CloseParen),
            ("{", PunctuationType::OpenBrace),
            ("}", PunctuationType::CloseBrace),
            (".", PunctuationType::Dot),
        ];

        for p in punctuators {
            let mut chars = p.0.into_code_iterator("script.js".to_string());
            let parsed = try_parse_punctuation(&mut chars).unwrap();
            assert_eq!(parsed.kind, p.1);
        }
    }

    #[test]
    fn test_non_existent_punctuator() {
        let mut chars = "!~~~~".into_code_iterator("script.js".to_string());
        let parsed = try_parse_punctuation(&mut chars);
        assert!(parsed.is_none());
    }
}
