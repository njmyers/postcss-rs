use crate::characters::*;
use crate::utils;

/// Tokenizer for CSS whitespace sequences.
///
/// CSS whitespace consists of space (U+0020), horizontal tab (U+0009),
/// newline (U+000A), vertical tab (U+000B), form feed (U+000C), and
/// carriage return (U+000D).
pub struct Whitespace;

impl Whitespace {
    /// Returns `true` if `letter` is a CSS whitespace character.
    pub fn test(letter: &char) -> bool {
        return matches!(
            letter,
            HORIZONTAL_TAB | NEWLINE | VERTICAL_TAB | FORM_FEED | CARRIAGE_RETURN | SPACE
        );
    }

    /// Returns the index of the last whitespace character in a run of whitespace.
    ///
    /// Scans `letters` from the start and returns the index of the last
    /// consecutive whitespace character (0-based). Returns `0` when the
    /// entire slice is whitespace or when the slice is empty.
    pub fn tokenize(letters: &[char]) -> usize {
        let result = letters.iter().position(|letter| {
            return !matches!(letter, letter if Self::test(letter));
        });

        let position = utils::unwrap_position(result, letters);
        return utils::back(position);
    }
}

/// Detector for CSS newline characters (LF and CR).
pub struct Newline;

impl Newline {
    /// Returns `true` if `letter` is a newline (`\n`) or carriage return (`\r`).
    pub fn test(letter: &char) -> bool {
        return matches!(letter, NEWLINE | CARRIAGE_RETURN);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_tests_newline_character() {
        assert!(Newline::test(&'\n'));
    }

    #[test]
    fn test_newline_carriage_return() {
        assert!(Newline::test(&'\r'));
    }

    #[test]
    fn test_newline_rejects_space() {
        assert!(!Newline::test(&' '));
        assert!(!Newline::test(&'\t'));
    }

    #[test]
    fn test_whitespace_space() {
        assert!(Whitespace::test(&' '));
    }

    #[test]
    fn test_whitespace_tab() {
        assert!(Whitespace::test(&'\t'));
    }

    #[test]
    fn test_whitespace_newline() {
        assert!(Whitespace::test(&'\n'));
    }

    #[test]
    fn test_whitespace_rejects_word_char() {
        assert!(!Whitespace::test(&'a'));
        assert!(!Whitespace::test(&'{'));
    }

    #[test]
    fn test_whitespace_tokenize_single_space() {
        let chars: Vec<char> = " hello".chars().collect();
        // Single leading space → index 0
        assert_eq!(Whitespace::tokenize(&chars), 0);
    }

    #[test]
    fn test_whitespace_tokenize_multiple_spaces() {
        let chars: Vec<char> = "   hello".chars().collect();
        // Three leading spaces → last whitespace is at index 2
        assert_eq!(Whitespace::tokenize(&chars), 2);
    }

    #[test]
    fn test_whitespace_tokenize_all_whitespace() {
        let chars: Vec<char> = "   ".chars().collect();
        // Entirely whitespace — back(3) = 2
        assert_eq!(Whitespace::tokenize(&chars), 2);
    }
}
