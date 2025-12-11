use crate::characters::*;
use crate::escape::Escape;
use crate::utils;
use crate::Token;

use std::cmp;

/// Tracks the state of bracket parsing including string context and nesting depth
#[derive(Debug, Clone, Copy)]
struct BracketState {
    in_double_string: bool,
    in_single_string: bool,
    curvy_depth: i32,  // Track () parentheses
    curly_depth: i32,  // Track {} curly braces
    square_depth: i32, // Track [] square brackets
}

impl BracketState {
    fn new() -> Self {
        Self {
            in_double_string: false,
            in_single_string: false,
            curvy_depth: 0,
            curly_depth: 0,
            square_depth: 0,
        }
    }

    /// Process a character and return the next state and whether to terminate
    fn process(&self, letter: &char, is_escaped: bool) -> (Self, bool) {
        let mut next = *self;

        // Update string state if character is not escaped
        if !is_escaped {
            match letter {
                DOUBLE_QUOTE => next.in_double_string = !next.in_double_string,
                SINGLE_QUOTE => next.in_single_string = !next.in_single_string,
                _ => {}
            }
        }

        // Skip bracket processing if inside a string or if current char is escaped
        let in_string = next.in_double_string || next.in_single_string;
        if in_string || is_escaped {
            return (next, false);
        }

        // Update depth based on bracket type
        match letter {
            OPEN_CURVY => next.curvy_depth += 1,
            CLOSE_CURVY => next.curvy_depth -= 1,
            OPEN_CURLY => next.curly_depth += 1,
            CLOSE_CURLY => next.curly_depth -= 1,
            OPEN_SQUARE => next.square_depth += 1,
            CLOSE_SQUARE => next.square_depth -= 1,
            _ => {}
        }

        // Terminate when we find a closing paren that closes the initial bracket
        let should_terminate = letter == CLOSE_CURVY
            && next.curvy_depth < 0  // Found more ) than (
            && next.curly_depth == 0 // All {} are balanced
            && next.square_depth == 0; // All [] are balanced

        (next, should_terminate)
    }
}

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
        let letters_lookbehind = letters.iter().zip(letters.iter().skip(lookbehind));

        let result = letters_lookbehind
            .scan(BracketState::new(), |state, (previous, letter)| {
                let is_escaped = Escape::test(previous);
                let (next_state, should_terminate) = state.process(letter, is_escaped);
                *state = next_state;
                Some(should_terminate)
            })
            .position(|should_terminate| should_terminate);

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
