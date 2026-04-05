use crate::brackets::*;
use crate::bytes::{AT, COLON, DASH, EXCLAMATION, NULL_BYTE};
use crate::comment::*;
use crate::escape::Escape;
use crate::string::*;
use crate::symbol::*;
use crate::whitespace::*;

use std::cmp;

pub struct Word;

impl Word {
    pub fn test(current: u8, next: u8) -> bool {
        if Self::leading(current) {
            // Colons followed by digits should not be Word tokens (e.g., :1 in --a:1)
            // But colons followed by letters are valid (e.g., :root, :hover)
            if current == COLON && next.is_ascii_digit() {
                return false;
            }
            Self::is_word_byte(next) || next == DASH || next == NULL_BYTE
        } else {
            Self::is_word_byte(current) || current == DASH || current == NULL_BYTE
        }
    }

    pub fn tokenize(bytes: &[u8]) -> usize {
        let count = bytes.len();
        let mut escape_open = false;

        for (i, &byte) in bytes.iter().enumerate().skip(1) {
            if escape_open {
                escape_open = false;
                continue;
            }
            if Escape::test(byte) {
                escape_open = true;
                continue;
            }
            if Whitespace::test(byte)
                || OpenCurly::test(byte)
                || CloseCurly::test(byte)
                || OpenSquare::test(byte)
                || CloseSquare::test(byte)
                || OpenParen::test(byte)
                || CloseParen::test(byte)
                || Semicolon::test(byte)
                || Colon::test(byte)
                || SingleString::test(byte)
                || DoubleString::test(byte)
                || Bracket::is_opening_token(byte)
                || Comment::is_forward_slash(byte)
                || Word::leading(byte)
            {
                return cmp::min(i - 1, count - 1);
            }
        }
        count - 1
    }

    fn leading(byte: u8) -> bool {
        byte == EXCLAMATION || byte == COLON
    }

    /// Returns true for bytes that are valid word characters.
    /// ASCII alphanumerics plus any non-ASCII byte (part of a multi-byte UTF-8 sequence).
    #[inline]
    fn is_word_byte(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || byte >= 0x80
    }
}

pub struct AtWord;

impl AtWord {
    pub fn test(current: u8, next: u8) -> bool {
        current == AT && Word::test(next, NULL_BYTE)
    }

    pub fn tokenize(bytes: &[u8]) -> usize {
        Word::tokenize(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple_word() {
        // "color " — stops at space (returns position of space - 1 = 4)
        assert_eq!(Word::tokenize(b"color "), 4);
    }

    #[test]
    fn tokenize_stops_at_semicolon() {
        // "red;" — stops at ; (returns position 2)
        assert_eq!(Word::tokenize(b"red;"), 2);
    }

    #[test]
    fn tokenize_stops_at_colon() {
        // "color:red" — stops at : (returns 4)
        assert_eq!(Word::tokenize(b"color:red"), 4);
    }

    #[test]
    fn tokenize_stops_at_opening_bracket() {
        // "url(" — stops at ( (returns 2)
        assert_eq!(Word::tokenize(b"url("), 2);
    }

    #[test]
    fn tokenize_splits_at_exclamation() {
        // "aa!bb" — stops before !, like the original (returns 1 → "aa")
        assert_eq!(Word::tokenize(b"aa!bb"), 1);
    }

    #[test]
    fn tokenize_mid_word_escape_skips_terminator() {
        // "a\;x" — backslash mid-word escapes the semicolon, all consumed
        assert_eq!(Word::tokenize(b"a\\;x"), 3);
    }

    #[test]
    fn tokenize_single_char_word() {
        // "a{" — just "a" (returns 0)
        assert_eq!(Word::tokenize(b"a{"), 0);
    }
}
