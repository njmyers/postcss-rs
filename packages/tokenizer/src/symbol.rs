pub use crate::characters::*;

/// Tokenizer for the opening curly brace (`{`).
pub struct OpenCurly;

impl OpenCurly {
    /// Returns `true` if `letter` is an opening curly brace (`{`).
    pub fn test(letter: &char) -> bool {
        return matches!(letter, OPEN_CURLY);
    }

    /// Single-character token; always returns `0` (no additional look-ahead needed).
    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

/// Tokenizer for the closing curly brace (`}`).
pub struct CloseCurly;

impl CloseCurly {
    /// Returns `true` if `letter` is a closing curly brace (`}`).
    pub fn test(letter: &char) -> bool {
        return matches!(letter, CLOSE_CURLY);
    }

    /// Single-character token; always returns `0`.
    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

/// Tokenizer for the opening square bracket (`[`).
pub struct OpenSquare;

impl OpenSquare {
    /// Returns `true` if `letter` is an opening square bracket (`[`).
    pub fn test(letter: &char) -> bool {
        return matches!(letter, OPEN_SQUARE);
    }

    /// Single-character token; always returns `0`.
    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

/// Tokenizer for the closing square bracket (`]`).
pub struct CloseSquare;

impl CloseSquare {
    /// Returns `true` if `letter` is a closing square bracket (`]`).
    pub fn test(letter: &char) -> bool {
        return matches!(letter, CLOSE_SQUARE);
    }

    /// Single-character token; always returns `0`.
    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

/// Tokenizer for the opening parenthesis (`(`).
pub struct OpenCurvy;

impl OpenCurvy {
    /// Returns `true` if `letter` is an opening parenthesis (`(`).
    pub fn test(letter: &char) -> bool {
        return matches!(letter, OPEN_CURVY);
    }

    /// Single-character token; always returns `0`.
    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

/// Tokenizer for the closing parenthesis (`)`).
pub struct CloseCurvy;

impl CloseCurvy {
    /// Returns `true` if `letter` is a closing parenthesis (`)`).
    pub fn test(letter: &char) -> bool {
        return matches!(letter, CLOSE_CURVY);
    }

    /// Single-character token; always returns `0`.
    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

/// Tokenizer for the semicolon (`;`).
pub struct Semicolon;

impl Semicolon {
    /// Returns `true` if `letter` is a semicolon (`;`).
    pub fn test(letter: &char) -> bool {
        return matches!(letter, SEMICOLON);
    }

    /// Single-character token; always returns `0`.
    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

/// Tokenizer for the colon (`:`).
pub struct Colon;

impl Colon {
    /// Returns `true` if `letter` is a colon (`:`).
    pub fn test(letter: &char) -> bool {
        return matches!(letter, COLON);
    }

    /// Single-character token; always returns `0`.
    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_curly() {
        assert!(OpenCurly::test(&'{'));
        assert!(!OpenCurly::test(&'}'));
        assert_eq!(OpenCurly::tokenize(&[]), 0);
    }

    #[test]
    fn test_close_curly() {
        assert!(CloseCurly::test(&'}'));
        assert!(!CloseCurly::test(&'{'));
        assert_eq!(CloseCurly::tokenize(&[]), 0);
    }

    #[test]
    fn test_open_square() {
        assert!(OpenSquare::test(&'['));
        assert!(!OpenSquare::test(&']'));
        assert_eq!(OpenSquare::tokenize(&[]), 0);
    }

    #[test]
    fn test_close_square() {
        assert!(CloseSquare::test(&']'));
        assert!(!CloseSquare::test(&'['));
        assert_eq!(CloseSquare::tokenize(&[]), 0);
    }

    #[test]
    fn test_open_curvy() {
        assert!(OpenCurvy::test(&'('));
        assert!(!OpenCurvy::test(&')'));
        assert_eq!(OpenCurvy::tokenize(&[]), 0);
    }

    #[test]
    fn test_close_curvy() {
        assert!(CloseCurvy::test(&')'));
        assert!(!CloseCurvy::test(&'('));
        assert_eq!(CloseCurvy::tokenize(&[]), 0);
    }

    #[test]
    fn test_semicolon() {
        assert!(Semicolon::test(&';'));
        assert!(!Semicolon::test(&':'));
        assert_eq!(Semicolon::tokenize(&[]), 0);
    }

    #[test]
    fn test_colon() {
        assert!(Colon::test(&':'));
        assert!(!Colon::test(&';'));
        assert_eq!(Colon::tokenize(&[]), 0);
    }
}
