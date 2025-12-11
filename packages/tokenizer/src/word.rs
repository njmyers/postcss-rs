use crate::brackets::*;
use crate::characters::*;
use crate::comment::*;
use crate::escape::Escape;
use crate::string::*;
use crate::symbol::*;
use crate::utils;
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
        let lookbehind: usize = 1;
        let count = letters.len();
        let letters_lookbehind = letters.iter().zip(letters.iter().skip(lookbehind));

        let result = letters_lookbehind
            .scan(false, |prev_was_escape, (_, letter)| {
                // If previous was an escape backslash, current char is escaped
                if *prev_was_escape {
                    *prev_was_escape = false;
                    return Some(false); // Don't terminate
                }

                // If current is a backslash, next char will be escaped
                if Escape::test(letter) {
                    *prev_was_escape = true;
                    return Some(false); // Don't terminate on the backslash itself
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

                Some(should_terminate)
            })
            .position(|should_terminate| should_terminate);

        let string_length = utils::unwrap_position(result, letters);
        return cmp::min(string_length, count - lookbehind);
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
