use super::*;
use std::cmp;

const OPEN_PARENTHESIS: &char = &'(';
const CLOSE_PARENTHESIS: &char = &')';

pub struct Bracket;

impl Bracket {
    /// Return whether a character is a single opening parenthesis token `(`
    pub fn test(current: &char) -> bool {
        return current == OPEN_PARENTHESIS;
    }

    /// Return whether a character is a single closing parenthesis token `(`
    pub fn is_closing_token(current: &char) -> bool {
        return current == CLOSE_PARENTHESIS;
    }

    /// Tokenize the bracketed statement contained within brackets like `(word)`.
    pub fn tokenize(letters: &[char]) -> usize {
        let lookbehind: usize = 1;
        let count = letters.iter().count();
        let mut letters_lookbehind = letters.iter().zip(letters.iter().skip(lookbehind));

        let result = letters_lookbehind.position(|(previous_letter, letter)| {
            return match letter {
                CLOSE_PARENTHESIS if !Escape::test(previous_letter) => true,
                _ => false,
            };
        });

        let string_length = utils::unwrap_position(result, letters) + lookbehind;
        return cmp::min(string_length, count - lookbehind);
    }
}
