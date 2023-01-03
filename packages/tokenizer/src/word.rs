use crate::brackets::*;
use crate::characters::*;
use crate::comment::*;
use crate::string::*;
use crate::symbol::*;
use crate::utils;
use crate::whitespace::*;

use std::cmp;

pub struct Word;

impl Word {
    pub fn test(current: &char, next: &char) -> bool {
        if Self::leading(current) {
            return next.is_alphanumeric() || next == DASH || next == NULL_CHARACTER;
        }

        return current.is_alphanumeric() || current == DASH || current == NULL_CHARACTER;
    }

    pub fn tokenize(letters: &[char]) -> usize {
        let lookbehind: usize = 1;
        let count = letters.len();
        let mut letters_lookbehind = letters.iter().zip(letters.iter().skip(lookbehind));

        let result = letters_lookbehind.position(|(_, letter)| {
            return match letter {
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
        });

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
