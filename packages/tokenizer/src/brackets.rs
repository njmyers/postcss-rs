use crate::characters::*;
use crate::escape::Escape;
use crate::utils;
use crate::Token;

use std::cmp;

/// Result of processing a character in bracket parsing
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

    /// Process a character and return the next state and bracket result
    fn process(&self, previous: &char, letter: &char) -> (Self, BracketResult) {
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
            OPEN_CURVY => next.curvy_depth += 1,
            CLOSE_CURVY => next.curvy_depth -= 1,
            OPEN_CURLY => next.curly_depth += 1,
            CLOSE_CURLY => next.curly_depth -= 1,
            OPEN_SQUARE => next.square_depth += 1,
            CLOSE_SQUARE => next.square_depth -= 1,
            _ => {}
        }

        // Check if we found the closing bracket
        let result = if letter == CLOSE_CURVY
            && next.curvy_depth < 0  // Found more ) than (
            && next.curly_depth == 0 // All {} are balanced
            && next.square_depth == 0
        // All [] are balanced
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
                let (next_state, result) = state.process(previous, letter);
                *state = next_state;
                Some(result)
            })
            .position(|result| result == BracketResult::Closed);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bracket_state_initializes_with_zero_state() {
        let state = BracketState::new();
        assert_eq!(state.in_double_string, false);
        assert_eq!(state.in_single_string, false);
        assert_eq!(state.curvy_depth, 0);
        assert_eq!(state.curly_depth, 0);
        assert_eq!(state.square_depth, 0);
    }

    #[test]
    fn bracket_state_enters_double_string() {
        let state = BracketState::new();
        let (next, result) = state.process(&'a', &'"');

        assert_eq!(next.in_double_string, true);
        assert_eq!(next.in_single_string, false);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_exits_double_string() {
        let mut state = BracketState::new();
        state.in_double_string = true;

        let (next, result) = state.process(&'a', &'"');

        assert_eq!(next.in_double_string, false);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_enters_single_string() {
        let state = BracketState::new();
        let (next, result) = state.process(&'a', &'\'');

        assert_eq!(next.in_single_string, true);
        assert_eq!(next.in_double_string, false);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_ignores_escaped_quotes() {
        let state = BracketState::new();
        let (next, result) = state.process(&'\\', &'"');

        // Should not enter string mode when quote is escaped
        assert_eq!(next.in_double_string, false);
        assert_eq!(next.in_single_string, false);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_tracks_curvy_depth() {
        let state = BracketState::new();

        // Open parenthesis
        let (next, _) = state.process(&'a', &'(');
        assert_eq!(next.curvy_depth, 1);

        // Close parenthesis
        let (next2, _) = next.process(&'a', &')');
        assert_eq!(next2.curvy_depth, 0);
    }

    #[test]
    fn bracket_state_tracks_curly_depth() {
        let state = BracketState::new();

        // Open brace
        let (next, _) = state.process(&'a', &'{');
        assert_eq!(next.curly_depth, 1);

        // Close brace
        let (next2, _) = next.process(&'a', &'}');
        assert_eq!(next2.curly_depth, 0);
    }

    #[test]
    fn bracket_state_tracks_square_depth() {
        let state = BracketState::new();

        // Open square bracket
        let (next, _) = state.process(&'a', &'[');
        assert_eq!(next.square_depth, 1);

        // Close square bracket
        let (next2, _) = next.process(&'a', &']');
        assert_eq!(next2.square_depth, 0);
    }

    #[test]
    fn bracket_state_ignores_brackets_in_double_string() {
        let mut state = BracketState::new();
        state.in_double_string = true;

        let (next, result) = state.process(&'a', &'(');

        // Should not increment depth when inside string
        assert_eq!(next.curvy_depth, 0);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_ignores_brackets_in_single_string() {
        let mut state = BracketState::new();
        state.in_single_string = true;

        let (next, result) = state.process(&'a', &'{');

        // Should not increment depth when inside string
        assert_eq!(next.curly_depth, 0);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_ignores_escaped_brackets() {
        let state = BracketState::new();

        let (next, result) = state.process(&'\\', &'(');

        // Should not increment depth when bracket is escaped
        assert_eq!(next.curvy_depth, 0);
        assert_eq!(result, BracketResult::Open);
    }

    #[test]
    fn bracket_state_terminates_on_balanced_close() {
        let state = BracketState::new();

        // Should return Closed when we see ) with negative curvy_depth and all others balanced
        let (next, result) = state.process(&'a', &')');

        assert_eq!(next.curvy_depth, -1);
        assert_eq!(result, BracketResult::Closed);
    }

    #[test]
    fn bracket_state_does_not_terminate_with_unbalanced_braces() {
        let mut state = BracketState::new();
        state.curly_depth = 1; // Unbalanced braces

        let (next, result) = state.process(&'a', &')');

        assert_eq!(next.curvy_depth, -1);
        assert_eq!(result, BracketResult::Open); // Should not close - braces not balanced
    }

    #[test]
    fn bracket_state_does_not_terminate_with_unbalanced_squares() {
        let mut state = BracketState::new();
        state.square_depth = 1; // Unbalanced square brackets

        let (next, result) = state.process(&'a', &')');

        assert_eq!(next.curvy_depth, -1);
        assert_eq!(result, BracketResult::Open); // Should not close - squares not balanced
    }

    #[test]
    fn bracket_state_handles_nested_brackets() {
        let state = BracketState::new();

        // Simulate: url(a(b)c)
        let (s1, _) = state.process(&'(', &'a');
        let (s2, _) = s1.process(&'a', &'(');  // nested open
        assert_eq!(s2.curvy_depth, 1);

        let (s3, _) = s2.process(&'(', &'b');
        let (s4, _) = s3.process(&'b', &')');  // nested close
        assert_eq!(s4.curvy_depth, 0);

        let (s5, _) = s4.process(&')', &'c');
        let (s6, result) = s5.process(&'c', &')');  // outer close

        assert_eq!(s6.curvy_depth, -1);
        assert_eq!(result, BracketResult::Closed);
    }

    #[test]
    fn bracket_state_handles_mixed_bracket_types() {
        let state = BracketState::new();

        // Simulate: url(a[b]{c}d)
        let (s1, _) = state.process(&'(', &'a');
        let (s2, _) = s1.process(&'a', &'[');
        assert_eq!(s2.square_depth, 1);

        let (s3, _) = s2.process(&'[', &'b');
        let (s4, _) = s3.process(&'b', &']');
        assert_eq!(s4.square_depth, 0);

        let (s5, _) = s4.process(&']', &'{');
        assert_eq!(s5.curly_depth, 1);

        let (s6, _) = s5.process(&'{', &'c');
        let (s7, _) = s6.process(&'c', &'}');
        assert_eq!(s7.curly_depth, 0);

        let (s8, _) = s7.process(&'}', &'d');
        let (_s9, result) = s8.process(&'d', &')');

        // All brackets balanced, should close
        assert_eq!(result, BracketResult::Closed);
    }

    #[test]
    fn bracket_state_handles_quotes_inside_brackets() {
        let state = BracketState::new();

        // Simulate: url("a(b)c")
        let (s1, _) = state.process(&'(', &'"');
        assert_eq!(s1.in_double_string, true);

        let (s2, _) = s1.process(&'"', &'a');
        let (s3, _) = s2.process(&'a', &'('); // Should be ignored (in string)
        assert_eq!(s3.curvy_depth, 0);

        let (s4, _) = s3.process(&'(', &'b');
        let (s5, _) = s4.process(&'b', &')'); // Should be ignored (in string)
        assert_eq!(s5.curvy_depth, 0);

        let (s6, _) = s5.process(&')', &'c');
        let (s7, _) = s6.process(&'c', &'"'); // Exit string
        assert_eq!(s7.in_double_string, false);

        let (_s8, result) = s7.process(&'"', &')'); // Should close
        assert_eq!(result, BracketResult::Closed);
    }

    #[test]
    fn bracket_state_handles_escaped_quote_in_string() {
        let mut state = BracketState::new();
        state.in_double_string = true;

        // Escaped quote should not exit string
        let (next, _) = state.process(&'\\', &'"');
        assert_eq!(next.in_double_string, true);
    }
}
