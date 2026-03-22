use crate::brackets::*;
use crate::characters::*;
use crate::comment::*;
use crate::escape::Escape;
use crate::string::*;
use crate::symbol::*;
use crate::whitespace::*;

use std::cmp;

pub struct Word;

impl Word {
    pub fn test(current: &char, next: &char) -> bool {
        if Self::leading(current) {
            // Colons followed by digits should not be Word tokens (e.g., :1 in --a:1)
            // But colons followed by letters are valid (e.g., :root, :hover)
            if current == COLON && next.is_numeric() {
                return false;
            }
            return next.is_alphanumeric() || next == DASH || next == NULL_CHARACTER;
        }

        return current.is_alphanumeric() || current == DASH || current == NULL_CHARACTER;
    }

    pub fn tokenize(letters: &[char]) -> usize {
        let count = letters.len();
        let mut escape_open = false;

        for i in 1..count {
            let letter = &letters[i];
            if escape_open {
                escape_open = false;
                continue;
            }
            if Escape::test(letter) {
                escape_open = true;
                continue;
            }
            if Whitespace::test(letter)
                || OpenCurly::test(letter)
                || CloseCurly::test(letter)
                || OpenSquare::test(letter)
                || CloseSquare::test(letter)
                || OpenCurvy::test(letter)
                || CloseCurvy::test(letter)
                || Semicolon::test(letter)
                || Colon::test(letter)
                || SingleString::test(letter)
                || DoubleString::test(letter)
                || Bracket::is_opening_token(letter)
                || Comment::is_forward_slash(letter)
                || Word::leading(letter)
            {
                return cmp::min(i - 1, count - 1);
            }
        }
        count - 1
    }

    fn leading(letter: &char) -> bool {
        return letter == EXCLAMATION_POINT || letter == COLON;
    }
}

pub struct AtWord;

impl AtWord {
    pub fn test(current: &char, next: &char) -> bool {
        return current == AT_CHAR && Word::test(next, NULL_CHARACTER);
    }

    pub fn tokenize(letters: &[char]) -> usize {
        return Word::tokenize(letters);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn tokenize_simple_word() {
        // "color " — stops at space (returns position of space - 1 = 4)
        let letters = chars("color ");
        assert_eq!(Word::tokenize(&letters), 4);
    }

    #[test]
    fn tokenize_stops_at_semicolon() {
        // "red;" — stops at ; (returns position 2)
        let letters = chars("red;");
        assert_eq!(Word::tokenize(&letters), 2);
    }

    #[test]
    fn tokenize_stops_at_colon() {
        // "color:red" — stops at : (returns 4)
        let letters = chars("color:red");
        assert_eq!(Word::tokenize(&letters), 4);
    }

    #[test]
    fn tokenize_stops_at_opening_bracket() {
        // "url(" — stops at ( (returns 2)
        let letters = chars("url(");
        assert_eq!(Word::tokenize(&letters), 2);
    }

    #[test]
    fn tokenize_splits_at_exclamation() {
        // "aa!bb" — stops before !, like the original (returns 1 → "aa")
        let letters = chars("aa!bb");
        assert_eq!(Word::tokenize(&letters), 1);
    }

    #[test]
    fn tokenize_mid_word_escape_skips_terminator() {
        // "a\;x" — backslash mid-word escapes the semicolon, all consumed
        let letters = chars("a\\;x");
        assert_eq!(Word::tokenize(&letters), 3);
    }

    #[test]
    fn tokenize_single_char_word() {
        // "a{" — just "a" (returns 0)
        let letters = chars("a{");
        assert_eq!(Word::tokenize(&letters), 0);
    }
}
