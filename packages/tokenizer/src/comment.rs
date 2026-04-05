use crate::bytes::{ASTERISK, FORWARD_SLASH};
use crate::utils;

/// Tokenizer for CSS block comments (`/* ... */`).
pub struct Comment;

impl Comment {
    /// Returns the index of the `*` in the closing `*/` sequence within `bytes`.
    ///
    /// The caller is expected to pass the slice that starts **after** the
    /// opening `/*`.  The returned index is then added to `3` by the
    /// [`Tokenizer`] to compute the total token length including the `/*` prefix
    /// and the `/` of `*/`.
    ///
    /// If no closing `*/` is found, returns the length of the slice.
    pub fn tokenize(bytes: &[u8]) -> usize {
        let mut iterator = bytes.iter().zip(bytes.iter().skip(1));
        let result = iterator.position(|(&current, &next)| Self::test_end(current, next));
        utils::unwrap_position(result, bytes.len())
    }

    /// Returns `true` if the two bytes form the start of a CSS comment (`/*`).
    pub fn test(current: u8, next: u8) -> bool {
        current == FORWARD_SLASH && next == ASTERISK
    }

    /// Returns `true` if the two bytes form the end of a CSS comment (`*/`).
    fn test_end(current: u8, next: u8) -> bool {
        current == ASTERISK && next == FORWARD_SLASH
    }

    /// Returns `true` if the byte is a forward slash (`/`).
    ///
    /// Used when disambiguating between a comment start (`/*`) and a lone `/`.
    pub fn is_forward_slash(byte: u8) -> bool {
        byte == FORWARD_SLASH
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_comment_start() {
        assert!(Comment::test(b'/', b'*'));
        assert!(!Comment::test(b'/', b'/'));
        assert!(!Comment::test(b'*', b'/'));
    }

    #[test]
    fn test_is_forward_slash() {
        assert!(Comment::is_forward_slash(b'/'));
        assert!(!Comment::is_forward_slash(b'*'));
        assert!(!Comment::is_forward_slash(b'a'));
    }

    #[test]
    fn test_tokenize_simple_comment() {
        // tokenize() is called with the slice AFTER "/*".
        // For " hello */", the '*' of the closing "*/" is at index 7.
        let end = Comment::tokenize(b" hello */");
        assert_eq!(end, 7);
    }

    #[test]
    fn test_tokenize_no_end_returns_length() {
        // If there is no "*/" the function returns the length of the slice.
        let end = Comment::tokenize(b" unclosed");
        assert_eq!(end, 9);
    }

    #[test]
    fn test_tokenize_empty_comment() {
        // For "*/", the '*' of the closing "*/" is at index 0.
        let end = Comment::tokenize(b"*/");
        assert_eq!(end, 0);
    }
}
