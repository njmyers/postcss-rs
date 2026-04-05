use crate::bytes::{
    BACKSLASH, CARRIAGE_RETURN, CLOSE_CURLY, CLOSE_PAREN, CLOSE_SQUARE, DOUBLE_QUOTE, FORM_FEED,
    FORWARD_SLASH, NEWLINE, NULL_BYTE, OPEN_CURLY, OPEN_PAREN, OPEN_SQUARE, SINGLE_QUOTE, SPACE,
    TAB, VERTICAL_TAB,
};
use crate::escape::Escape;
use crate::utils;

use std::cmp;

/// Result of processing a byte in bracket parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BracketResult {
    /// Continue parsing - bracket is still open
    Open,
    /// Stop parsing - found the closing bracket
    Closed,
}

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

    /// Process a byte and return the next state and bracket result
    fn process(&self, previous: u8, letter: u8) -> (Self, BracketResult) {
        let mut next = *self;
        let is_escaped = Escape::test(previous);

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
            return (next, BracketResult::Open);
        }

        // Update depth based on bracket type
        match letter {
            OPEN_PAREN => next.curvy_depth += 1,
            CLOSE_PAREN => next.curvy_depth -= 1,
            OPEN_CURLY => next.curly_depth += 1,
            CLOSE_CURLY => next.curly_depth -= 1,
            OPEN_SQUARE => next.square_depth += 1,
            CLOSE_SQUARE => next.square_depth -= 1,
            _ => {}
        }

        // Check if we found the closing bracket
        let result = if letter == CLOSE_PAREN
            && next.curvy_depth < 0  // Found more ) than (
            && next.curly_depth == 0 // All {} are balanced
            && next.square_depth == 0
        {
            BracketResult::Closed
        } else {
            BracketResult::Open
        };

        (next, result)
    }
}

pub struct Bracket;

impl Bracket {
    /// Return whether a byte sequence starts a bracketed token `(`
    pub fn test(current: u8, next: u8, prev: Option<&str>) -> bool {
        if current != OPEN_PAREN {
            return false;
        }

        if Self::is_prev_url_token(prev) {
            return !matches!(
                next,
                SINGLE_QUOTE
                    | DOUBLE_QUOTE
                    | SPACE
                    | NEWLINE
                    | TAB
                    | VERTICAL_TAB
                    | FORM_FEED
                    | CARRIAGE_RETURN
            );
        }

        !matches!(
            next,
            OPEN_PAREN
                | SINGLE_QUOTE
                | DOUBLE_QUOTE
                | SPACE
                | NEWLINE
                | TAB
                | FORWARD_SLASH
                | BACKSLASH
                | VERTICAL_TAB
                | FORM_FEED
                | CARRIAGE_RETURN
                | NULL_BYTE
        )
    }

    /// Return whether a byte is a single opening parenthesis token `(`
    pub fn is_opening_token(byte: u8) -> bool {
        byte == OPEN_PAREN
    }

    /// Return whether a byte is a single closing parenthesis token `)`
    pub fn is_closing_token(byte: u8) -> bool {
        byte == CLOSE_PAREN
    }

    /// Tokenize the bracketed statement contained within brackets like `(word)`.
    /// Properly handles nested brackets of all types: (), {}, and [].
    pub fn tokenize(bytes: &[u8]) -> usize {
        let lookbehind: usize = 1;
        let count = bytes.len();
        let bytes_lookbehind = bytes.iter().zip(bytes.iter().skip(lookbehind));

        let result = bytes_lookbehind
            .scan(BracketState::new(), |state, (&previous, &letter)| {
                let (next_state, result) = state.process(previous, letter);
                *state = next_state;
                Some(result)
            })
            .position(|result| result == BracketResult::Closed);

        let length = utils::unwrap_position(result, bytes.len()) + lookbehind;
        cmp::min(length, count - lookbehind)
    }

    fn is_prev_url_token(prev: Option<&str>) -> bool {
        prev == Some("url")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bracket_state_initializes_with_zero_state() {
        let state = BracketState::new();
        assert!(!state.in_double_string);
        assert!(!state.in_single_string);
        assert_eq!(state.curvy_depth, 0);
        assert_eq!(state.curly_depth, 0);
        assert_eq!(state.square_depth, 0);
    }

    #[test]
    fn bracket_state_enters_double_string() {
        let state = BracketState::new();
        let (next, result) = state.process(b'a', b'"');

        assert!(next.in_double_string);
        assert!(!next.in_single_string);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_exits_double_string() {
        let mut state = BracketState::new();
        state.in_double_string = true;

        let (next, result) = state.process(b'a', b'"');

        assert!(!next.in_double_string);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_enters_single_string() {
        let state = BracketState::new();
        let (next, result) = state.process(b'a', b'\'');

        assert!(next.in_single_string);
        assert!(!next.in_double_string);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_ignores_escaped_quotes() {
        let state = BracketState::new();
        let (next, result) = state.process(b'\\', b'"');

        assert!(!next.in_double_string);
        assert!(!next.in_single_string);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_tracks_curvy_depth() {
        let state = BracketState::new();

        let (next, _) = state.process(b'a', b'(');
        assert_eq!(next.curvy_depth, 1);

        let (next2, _) = next.process(b'a', b')');
        assert_eq!(next2.curvy_depth, 0);
    }

    #[test]
    fn bracket_state_tracks_curly_depth() {
        let state = BracketState::new();

        let (next, _) = state.process(b'a', b'{');
        assert_eq!(next.curly_depth, 1);

        let (next2, _) = next.process(b'a', b'}');
        assert_eq!(next2.curly_depth, 0);
    }

    #[test]
    fn bracket_state_tracks_square_depth() {
        let state = BracketState::new();

        let (next, _) = state.process(b'a', b'[');
        assert_eq!(next.square_depth, 1);

        let (next2, _) = next.process(b'a', b']');
        assert_eq!(next2.square_depth, 0);
    }

    #[test]
    fn bracket_state_ignores_brackets_in_double_string() {
        let mut state = BracketState::new();
        state.in_double_string = true;

        let (next, result) = state.process(b'a', b'(');

        assert_eq!(next.curvy_depth, 0);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_ignores_brackets_in_single_string() {
        let mut state = BracketState::new();
        state.in_single_string = true;

        let (next, result) = state.process(b'a', b'{');

        assert_eq!(next.curly_depth, 0);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_ignores_escaped_brackets() {
        let state = BracketState::new();

        let (next, result) = state.process(b'\\', b'(');

        assert_eq!(next.curvy_depth, 0);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_terminates_on_balanced_close() {
        let state = BracketState::new();

        let (next, result) = state.process(b'a', b')');

        assert_eq!(next.curvy_depth, -1);
        assert_eq!(result, BracketResult::Closed);
    }

    #[test]
    fn bracket_state_does_not_terminate_with_unbalanced_braces() {
        let mut state = BracketState::new();
        state.curly_depth = 1;

        let (next, result) = state.process(b'a', b')');

        assert_eq!(next.curvy_depth, -1);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_does_not_terminate_with_unbalanced_squares() {
        let mut state = BracketState::new();
        state.square_depth = 1;

        let (next, result) = state.process(b'a', b')');

        assert_eq!(next.curvy_depth, -1);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_handles_nested_brackets() {
        let state = BracketState::new();

        let (s1, _) = state.process(b'(', b'a');
        let (s2, _) = s1.process(b'a', b'(');
        assert_eq!(s2.curvy_depth, 1);

        let (s3, _) = s2.process(b'(', b'b');
        let (s4, _) = s3.process(b'b', b')');
        assert_eq!(s4.curvy_depth, 0);

        let (s5, _) = s4.process(b')', b'c');
        let (s6, result) = s5.process(b'c', b')');

        assert_eq!(s6.curvy_depth, -1);
        assert_eq!(result, BracketResult::Closed);
    }

    #[test]
    fn bracket_state_handles_mixed_bracket_types() {
        let state = BracketState::new();

        let (s1, _) = state.process(b'(', b'a');
        let (s2, _) = s1.process(b'a', b'[');
        assert_eq!(s2.square_depth, 1);

        let (s3, _) = s2.process(b'[', b'b');
        let (s4, _) = s3.process(b'b', b']');
        assert_eq!(s4.square_depth, 0);

        let (s5, _) = s4.process(b']', b'{');
        assert_eq!(s5.curly_depth, 1);

        let (s6, _) = s5.process(b'{', b'c');
        let (s7, _) = s6.process(b'c', b'}');
        assert_eq!(s7.curly_depth, 0);

        let (s8, _) = s7.process(b'}', b'd');
        let (_s9, result) = s8.process(b'd', b')');

        assert_eq!(result, BracketResult::Closed);
    }

    #[test]
    fn bracket_state_handles_quotes_inside_brackets() {
        let state = BracketState::new();

        let (s1, _) = state.process(b'(', b'"');
        assert!(s1.in_double_string);

        let (s2, _) = s1.process(b'"', b'a');
        let (s3, _) = s2.process(b'a', b'(');
        assert_eq!(s3.curvy_depth, 0);

        let (s4, _) = s3.process(b'(', b'b');
        let (s5, _) = s4.process(b'b', b')');
        assert_eq!(s5.curvy_depth, 0);

        let (s6, _) = s5.process(b')', b'c');
        let (s7, _) = s6.process(b'c', b'"');
        assert!(!s7.in_double_string);

        let (_s8, result) = s7.process(b'"', b')');
        assert_eq!(result, BracketResult::Closed);
    }

    #[test]
    fn bracket_state_handles_escaped_quote_in_string() {
        let mut state = BracketState::new();
        state.in_double_string = true;

        let (next, _) = state.process(b'\\', b'"');
        assert!(next.in_double_string);
    }
}
