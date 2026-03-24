use crate::bytes::{CARRIAGE_RETURN, FORM_FEED, NEWLINE, SPACE, TAB, VERTICAL_TAB};
use crate::utils;

/// Tokenizer for CSS whitespace sequences.
///
/// CSS whitespace consists of space (U+0020), horizontal tab (U+0009),
/// newline (U+000A), vertical tab (U+000B), form feed (U+000C), and
/// carriage return (U+000D).
pub struct Whitespace;

impl Whitespace {
    /// Returns `true` if `byte` is a CSS whitespace character.
    pub fn test(byte: u8) -> bool {
        matches!(byte, TAB | NEWLINE | VERTICAL_TAB | FORM_FEED | CARRIAGE_RETURN | SPACE)
    }

    /// Returns the index of the last whitespace byte in a run of whitespace.
    ///
    /// Scans `bytes` from the start and returns the index of the last
    /// consecutive whitespace byte (0-based). Returns `0` when the
    /// entire slice is whitespace or when the slice is empty.
    pub fn tokenize(bytes: &[u8]) -> usize {
        let result = bytes.iter().position(|&b| !Self::test(b));
        let position = utils::unwrap_position(result, bytes.len());
        return utils::back(position);
    }
}

/// Detector for CSS newline characters (LF and CR).
pub struct Newline;

impl Newline {
    /// Returns `true` if `byte` is a newline (`\n`) or carriage return (`\r`).
    pub fn test(byte: u8) -> bool {
        matches!(byte, NEWLINE | CARRIAGE_RETURN)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_tests_newline_character() {
        assert!(Newline::test(b'\n'));
    }

    #[test]
    fn test_newline_carriage_return() {
        assert!(Newline::test(b'\r'));
    }

    #[test]
    fn test_newline_rejects_space() {
        assert!(!Newline::test(b' '));
        assert!(!Newline::test(b'\t'));
    }

    #[test]
    fn test_whitespace_space() {
        assert!(Whitespace::test(b' '));
    }

    #[test]
    fn test_whitespace_tab() {
        assert!(Whitespace::test(b'\t'));
    }

    #[test]
    fn test_whitespace_newline() {
        assert!(Whitespace::test(b'\n'));
    }

    #[test]
    fn test_whitespace_rejects_word_char() {
        assert!(!Whitespace::test(b'a'));
        assert!(!Whitespace::test(b'{'));
    }

    #[test]
    fn test_whitespace_tokenize_single_space() {
        assert_eq!(Whitespace::tokenize(b" hello"), 0);
    }

    #[test]
    fn test_whitespace_tokenize_multiple_spaces() {
        assert_eq!(Whitespace::tokenize(b"   hello"), 2);
    }

    #[test]
    fn test_whitespace_tokenize_all_whitespace() {
        assert_eq!(Whitespace::tokenize(b"   "), 2);
    }
}
