use crate::characters::*;
use crate::escape::Escape;
use crate::string::DoubleString;
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
            return !matches!(
                next,
                SINGLE_QUOTE
                    | DOUBLE_QUOTE
                    | SPACE
                    | NEWLINE
                    | HORIZONTAL_TAB
                    | VERTICAL_TAB
                    | FORM_FEED
                    | CARRIAGE_RETURN
            );
        }

        return !matches!(
            next,
            OPEN_CURVY
                | SINGLE_QUOTE
                | DOUBLE_QUOTE
                | SPACE
                | NEWLINE
                | HORIZONTAL_TAB
                | FORWARD_SLASH
                | BACKWARD_SLASH
                | VERTICAL_TAB
                | FORM_FEED
                | CARRIAGE_RETURN
                | NULL_CHARACTER
        );
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
    /// Properly handles nested brackets of all types: (), {}, and [].
    pub fn tokenize(letters: &[char]) -> usize {
        let lookbehind: usize = 1;
        let count = letters.len();
        let mut letters_lookbehind = letters.iter().zip(letters.iter().skip(lookbehind));

        // Track depth of all bracket types
        let mut curvy_depth = 0; // Track () parentheses
        let mut curly_depth = 0; // Track {} curly braces
        let mut square_depth = 0; // Track [] square brackets

        let result = letters_lookbehind.position(|(previous, letter)| {
            // Skip if escaped or in string
            if Escape::test(previous) || DoubleString::test(previous) {
                return false;
            }

            // Update depth based on bracket type
            match letter {
                OPEN_CURVY => curvy_depth += 1,
                CLOSE_CURVY => curvy_depth -= 1,
                OPEN_CURLY => curly_depth += 1,
                CLOSE_CURLY => curly_depth -= 1,
                OPEN_SQUARE => square_depth += 1,
                CLOSE_SQUARE => square_depth -= 1,
                _ => {}
            }

            // Match closing paren only when all brackets are balanced
            return letter == CLOSE_CURVY
                && curvy_depth < 0  // Found more ) than (
                && curly_depth == 0 // All {} are balanced
                && square_depth == 0; // All [] are balanced
        });

        let length = utils::unwrap_position(result, letters) + lookbehind;
        return cmp::min(length, count - lookbehind);
    }

    fn is_prev_url_token(prev: Option<Token>) -> bool {
        return match prev {
            Some(token) => token.value == "url",
            None => false,
        };
    }
}
