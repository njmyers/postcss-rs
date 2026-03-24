use crate::bytes::{CLOSE_CURLY, CLOSE_PAREN, CLOSE_SQUARE, COLON, OPEN_CURLY, OPEN_PAREN, OPEN_SQUARE, SEMICOLON};

/// Tokenizer for the opening curly brace (`{`).
pub struct OpenCurly;

impl OpenCurly {
    /// Returns `true` if `byte` is an opening curly brace (`{`).
    pub fn test(byte: u8) -> bool {
        byte == OPEN_CURLY
    }

    /// Single-byte token; always returns `0`.
    pub fn tokenize(_: &[u8]) -> usize {
        return 0;
    }
}

/// Tokenizer for the closing curly brace (`}`).
pub struct CloseCurly;

impl CloseCurly {
    /// Returns `true` if `byte` is a closing curly brace (`}`).
    pub fn test(byte: u8) -> bool {
        byte == CLOSE_CURLY
    }

    /// Single-byte token; always returns `0`.
    pub fn tokenize(_: &[u8]) -> usize {
        return 0;
    }
}

/// Tokenizer for the opening square bracket (`[`).
pub struct OpenSquare;

impl OpenSquare {
    /// Returns `true` if `byte` is an opening square bracket (`[`).
    pub fn test(byte: u8) -> bool {
        byte == OPEN_SQUARE
    }

    /// Single-byte token; always returns `0`.
    pub fn tokenize(_: &[u8]) -> usize {
        return 0;
    }
}

/// Tokenizer for the closing square bracket (`]`).
pub struct CloseSquare;

impl CloseSquare {
    /// Returns `true` if `byte` is a closing square bracket (`]`).
    pub fn test(byte: u8) -> bool {
        byte == CLOSE_SQUARE
    }

    /// Single-byte token; always returns `0`.
    pub fn tokenize(_: &[u8]) -> usize {
        return 0;
    }
}

/// Tokenizer for the opening parenthesis (`(`).
pub struct OpenCurvy;

impl OpenCurvy {
    /// Returns `true` if `byte` is an opening parenthesis (`(`).
    pub fn test(byte: u8) -> bool {
        byte == OPEN_PAREN
    }

    /// Single-byte token; always returns `0`.
    pub fn tokenize(_: &[u8]) -> usize {
        return 0;
    }
}

/// Tokenizer for the closing parenthesis (`)`).
pub struct CloseCurvy;

impl CloseCurvy {
    /// Returns `true` if `byte` is a closing parenthesis (`)`).
    pub fn test(byte: u8) -> bool {
        byte == CLOSE_PAREN
    }

    /// Single-byte token; always returns `0`.
    pub fn tokenize(_: &[u8]) -> usize {
        return 0;
    }
}

/// Tokenizer for the semicolon (`;`).
pub struct Semicolon;

impl Semicolon {
    /// Returns `true` if `byte` is a semicolon (`;`).
    pub fn test(byte: u8) -> bool {
        byte == SEMICOLON
    }

    /// Single-byte token; always returns `0`.
    pub fn tokenize(_: &[u8]) -> usize {
        return 0;
    }
}

/// Tokenizer for the colon (`:`).
pub struct Colon;

impl Colon {
    /// Returns `true` if `byte` is a colon (`:`).
    pub fn test(byte: u8) -> bool {
        byte == COLON
    }

    /// Single-byte token; always returns `0`.
    pub fn tokenize(_: &[u8]) -> usize {
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_curly() {
        assert!(OpenCurly::test(b'{'));
        assert!(!OpenCurly::test(b'}'));
        assert_eq!(OpenCurly::tokenize(&[]), 0);
    }

    #[test]
    fn test_close_curly() {
        assert!(CloseCurly::test(b'}'));
        assert!(!CloseCurly::test(b'{'));
        assert_eq!(CloseCurly::tokenize(&[]), 0);
    }

    #[test]
    fn test_open_square() {
        assert!(OpenSquare::test(b'['));
        assert!(!OpenSquare::test(b']'));
        assert_eq!(OpenSquare::tokenize(&[]), 0);
    }

    #[test]
    fn test_close_square() {
        assert!(CloseSquare::test(b']'));
        assert!(!CloseSquare::test(b'['));
        assert_eq!(CloseSquare::tokenize(&[]), 0);
    }

    #[test]
    fn test_open_curvy() {
        assert!(OpenCurvy::test(b'('));
        assert!(!OpenCurvy::test(b')'));
        assert_eq!(OpenCurvy::tokenize(&[]), 0);
    }

    #[test]
    fn test_close_curvy() {
        assert!(CloseCurvy::test(b')'));
        assert!(!CloseCurvy::test(b'('));
        assert_eq!(CloseCurvy::tokenize(&[]), 0);
    }

    #[test]
    fn test_semicolon() {
        assert!(Semicolon::test(b';'));
        assert!(!Semicolon::test(b':'));
        assert_eq!(Semicolon::tokenize(&[]), 0);
    }

    #[test]
    fn test_colon() {
        assert!(Colon::test(b':'));
        assert!(!Colon::test(b';'));
        assert_eq!(Colon::tokenize(&[]), 0);
    }
}
