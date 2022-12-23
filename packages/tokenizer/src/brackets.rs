use crate::characters::*;
use crate::escape::Escape;
use crate::utils;
use crate::Token;

use std::cmp;

pub struct Bracket;

impl Bracket {
    /// Return whether a character is a single opening parenthesis token `(`
    pub fn test(current: &char, next: &char, prev: Option<Token>) -> bool {
        if current != OPEN_CURVY {
            return false;
        }

        if Self::is_prev_url_token(prev) {
            return match next {
                SINGLE_QUOTE => false,
                DOUBLE_QUOTE => false,
                SPACE => false,
                NEWLINE => false,
                HORIZONTAL_TAB => false,
                VERTICAL_TAB => false,
                FORM_FEED => false,
                CARRIAGE_RETURN => false,
                _ => true,
            };
        }

        return match next {
            OPEN_CURLY => false,
            _ => true,
        };
    }

    /// Return whether a character is a single opening parenthesis token `(`
    pub fn is_opening_token(current: &char) -> bool {
        return current == OPEN_CURVY;
    }

    /// Return whether a character is a single closing parenthesis token `)`
    pub fn is_closing_token(current: &char) -> bool {
        return current == CLOSE_CURVY;
    }

    /// Tokenize the bracketed statement contained within brackets like `(word)`.
    pub fn tokenize(letters: &[char]) -> usize {
        let lookbehind: usize = 1;
        let count = letters.iter().count();
        let mut letters_lookbehind = letters.iter().zip(letters.iter().skip(lookbehind));

        let result = letters_lookbehind.position(|(previous_letter, letter)| {
            return match letter {
                CLOSE_CURVY if !Escape::test(previous_letter) => true,
                _ => false,
            };
        });

        let string_length = utils::unwrap_position(result, letters) + lookbehind;
        return cmp::min(string_length, count - lookbehind);
    }

    fn is_prev_url_token(prev: Option<Token>) -> bool {
        return match prev {
            Some(token) => token.value == "url",
            None => true,
        };
    }
}
