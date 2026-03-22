use crate::characters::*;
use crate::utils;

/// Tokenizer for CSS block comments (`/* ... */`).
pub struct Comment;

impl Comment {
    /// Returns the index of the `*` in the closing `*/` sequence within `letters`.
    ///
    /// The caller is expected to pass the slice that starts **after** the
    /// opening `/*`.  The returned index is then added to `3` by the
    /// [`Tokenizer`] to compute the total token length including the `/*` prefix
    /// and the `/` of `*/`.
    ///
    /// If no closing `*/` is found, returns the length of the slice.
    pub fn tokenize(letters: &[char]) -> usize {
        let mut iterator = letters.iter().zip(letters.iter().skip(1));
        let result = iterator.position(|(current, next)| Self::test_end(current, next));

        let position = utils::unwrap_position(result, letters);
        return position;
    }

    /// Returns `true` if the two characters form the start of a CSS comment (`/*`).
    pub fn test(current: &char, next: &char) -> bool {
        return current == FORWARD_SLASH && next == ASTERISK;
    }

    /// Returns `true` if the two characters form the end of a CSS comment (`*/`).
    fn test_end(current: &char, next: &char) -> bool {
        return current == ASTERISK && next == FORWARD_SLASH;
    }

    /// Returns `true` if the character is a forward slash (`/`).
    ///
    /// Used when disambiguating between a comment start (`/*`) and a lone `/`.
    pub fn is_forward_slash(letter: &char) -> bool {
        return letter == FORWARD_SLASH;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_comment_start() {
        assert!(Comment::test(&'/', &'*'));
        assert!(!Comment::test(&'/', &'/'));
        assert!(!Comment::test(&'*', &'/'));
    }

    #[test]
    fn test_is_forward_slash() {
        assert!(Comment::is_forward_slash(&'/'));
        assert!(!Comment::is_forward_slash(&'*'));
        assert!(!Comment::is_forward_slash(&'a'));
    }

    #[test]
    fn test_tokenize_simple_comment() {
        // tokenize() is called with the slice AFTER "/*".
        // For " hello */", the '*' of the closing "*/" is at index 7.
        let chars: Vec<char> = " hello */".chars().collect();
        let end = Comment::tokenize(&chars);
        assert_eq!(end, 7);
    }

    #[test]
    fn test_tokenize_no_end_returns_length() {
        // If there is no "*/" the function returns the length of the slice.
        let chars: Vec<char> = " unclosed".chars().collect();
        let end = Comment::tokenize(&chars);
        assert_eq!(end, chars.len());
    }

    #[test]
    fn test_tokenize_empty_comment() {
        // For "*/", the '*' of the closing "*/" is at index 0.
        let chars: Vec<char> = "*/".chars().collect();
        let end = Comment::tokenize(&chars);
        assert_eq!(end, 0);
    }
}
