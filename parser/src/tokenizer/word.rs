use super::*;
use std::cmp;

/// CSS Leading Word Markers
const EXCLAMATION_POINT: &char = &'!';
const AT_CHAR: &char = &'@';
const COLON: &char = &':';
const DASH: &char = &'-';
const NULL_CHARACTER: &char = &'\u{0000}';

pub struct Word;

impl Word {
    pub fn test(current: &char) -> bool {
        return current.is_alphanumeric() || current == NULL_CHARACTER || current == DASH;
    }

    pub fn tokenize(letters: &[char]) -> usize {
        let lookbehind: usize = 1;
        let count = letters.iter().count();
        let mut letters_lookbehind = letters.iter().zip(letters.iter().skip(lookbehind));

        let result = letters_lookbehind.position(|(_, letter)| {
            return match letter {
                letter if Whitespace::test(letter) => true,
                letter if Symbol::test(letter) => true,
                letter if SingleString::test(letter) => true,
                letter if DoubleString::test(letter) => true,
                letter if Bracket::test(letter) => true,
                letter if Comment::is_forward_slash(letter) => true,
                letter if Self::leading(letter) => true,
                _ => false,
            };
        });

        let string_length = utils::unwrap_position(result, letters);
        return cmp::min(string_length, count - lookbehind);
    }

    pub fn leading(letter: &char) -> bool {
        return match letter {
            EXCLAMATION_POINT => true,
            COLON => true,
            _ => false,
        };
    }
}

pub struct AtWord;

impl AtWord {
    pub fn test(current: &char, next: &char) -> bool {
        return current == AT_CHAR && Word::test(next);
    }

    pub fn tokenize(letters: &[char]) -> usize {
        return Word::tokenize(letters);
    }
}
