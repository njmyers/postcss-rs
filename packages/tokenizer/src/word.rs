use crate::brackets::*;
use crate::characters::*;
use crate::comment::*;
use crate::escape::Escape;
use crate::string::*;
use crate::symbol::*;
use crate::whitespace::*;

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
        let lookbehind: usize = 1;
        let count = letters.len();

        // Track whether the next character is escaped by a backslash
        let mut next_is_escaped = false;

        for i in 1..count {
            let letter = &letters[i];

            // If this character was escaped by previous backslash, skip it
            if next_is_escaped {
                next_is_escaped = false;
                continue;
            }

            // If this is a backslash, the next character will be escaped
            if Escape::test(letter) {
                next_is_escaped = true;
                continue;
            }

            // Check if this character should terminate the word
            let should_terminate = match letter {
                letter if Whitespace::test(letter) => true,
                letter if OpenCurly::test(letter) => true,
                letter if CloseCurly::test(letter) => true,
                letter if OpenSquare::test(letter) => true,
                letter if CloseSquare::test(letter) => true,
                letter if OpenCurvy::test(letter) => true,
                letter if CloseCurvy::test(letter) => true,
                letter if Semicolon::test(letter) => true,
                letter if Colon::test(letter) => true,
                letter if SingleString::test(letter) => true,
                letter if DoubleString::test(letter) => true,
                letter if Bracket::is_opening_token(letter) => true,
                letter if Comment::is_forward_slash(letter) => true,
                letter if Self::leading(letter) => true,
                _ => false,
            };

            if should_terminate {
                return i - 1;
            }
        }

        // No terminator found, use all characters
        return count - lookbehind;
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
