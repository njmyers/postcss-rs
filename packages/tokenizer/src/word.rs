use crate::brackets::*;
use crate::characters::*;
use crate::comment::*;
use crate::escape::Escape;
use crate::string::*;
use crate::symbol::*;
use crate::utils;
use crate::whitespace::*;

use std::cmp;

/// Result of processing a character in word tokenization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WordResult {
    /// Continue tokenizing - character is part of the word
    Continue,
    /// Stop tokenizing - found a terminating character
    Terminate,
}

/// State of escape sequence processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WordEscapeState {
    /// Escape sequence is open - next character will be escaped
    Open,
    /// No active escape sequence
    Closed,
}

/// Tracks the state of word tokenization including escape handling
#[derive(Debug, Clone, Copy)]
struct WordState {
    escape_state: WordEscapeState,
}

impl WordState {
    fn new() -> Self {
        Self {
            escape_state: WordEscapeState::Closed,
        }
    }

    /// Process a character and return the next state and word result
    fn process(&self, letter: &char) -> (Self, WordResult) {
        // Determine if current character should terminate and what next state is
        let (result, next_escape_state) = if self.escape_state == WordEscapeState::Open {
            // Current char is escaped - don't terminate, close the escape sequence
            (WordResult::Continue, WordEscapeState::Closed)
        } else if Escape::test(letter) {
            // Current is backslash - don't terminate, open escape sequence
            (WordResult::Continue, WordEscapeState::Open)
        } else {
            // Normal character - check if it should terminate
            let terminates = match letter {
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
                letter if Word::leading(letter) => true,
                _ => false,
            };

            let word_result = if terminates {
                WordResult::Terminate
            } else {
                WordResult::Continue
            };
            (word_result, WordEscapeState::Closed)
        };

        (
            Self {
                escape_state: next_escape_state,
            },
            result,
        )
    }
}

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
            .scan(WordState::new(), |state, (_, letter)| {
                let (next_state, result) = state.process(letter);
                *state = next_state;
                Some(result)
            })
            .position(|result| result == WordResult::Terminate);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_state_initializes_with_closed_escape() {
        let state = WordState::new();
        assert_eq!(state.escape_state, WordEscapeState::Closed);
    }

    #[test]
    fn word_state_continues_on_normal_character() {
        let state = WordState::new();
        let (next, result) = state.process(&'a');

        assert_eq!(result, WordResult::Continue);
        assert_eq!(next.escape_state, WordEscapeState::Closed);
    }

    #[test]
    fn word_state_terminates_on_whitespace() {
        let state = WordState::new();
        let (next, result) = state.process(&' ');

        assert_eq!(result, WordResult::Terminate);
        assert_eq!(next.escape_state, WordEscapeState::Closed);
    }

    #[test]
    fn word_state_terminates_on_semicolon() {
        let state = WordState::new();
        let (next, result) = state.process(&';');

        assert_eq!(result, WordResult::Terminate);
        assert_eq!(next.escape_state, WordEscapeState::Closed);
    }

    #[test]
    fn word_state_terminates_on_colon() {
        let state = WordState::new();
        let (next, result) = state.process(&':');

        assert_eq!(result, WordResult::Terminate);
        assert_eq!(next.escape_state, WordEscapeState::Closed);
    }

    #[test]
    fn word_state_terminates_on_opening_bracket() {
        let state = WordState::new();
        let (next, result) = state.process(&'(');

        assert_eq!(result, WordResult::Terminate);
        assert_eq!(next.escape_state, WordEscapeState::Closed);
    }

    #[test]
    fn word_state_opens_escape_on_backslash() {
        let state = WordState::new();
        let (next, result) = state.process(&'\\');

        assert_eq!(result, WordResult::Continue);
        assert_eq!(next.escape_state, WordEscapeState::Open);
    }

    #[test]
    fn word_state_continues_when_escape_is_open() {
        let mut state = WordState::new();
        state.escape_state = WordEscapeState::Open;

        // Even a normally terminating character should continue when escape is open
        let (next, result) = state.process(&';');

        assert_eq!(result, WordResult::Continue);
        assert_eq!(next.escape_state, WordEscapeState::Closed); // Escape closes after consuming the char
    }

    #[test]
    fn word_state_handles_escape_sequence() {
        let state = WordState::new();

        // First character is backslash - opens escape
        let (s1, r1) = state.process(&'\\');
        assert_eq!(r1, WordResult::Continue);
        assert_eq!(s1.escape_state, WordEscapeState::Open);

        // Next character is escaped, so it continues even though it's a semicolon
        let (s2, r2) = s1.process(&';');
        assert_eq!(r2, WordResult::Continue);
        assert_eq!(s2.escape_state, WordEscapeState::Closed);

        // Back to normal - this semicolon should terminate
        let (_s3, r3) = s2.process(&';');
        assert_eq!(r3, WordResult::Terminate);
    }

    #[test]
    fn word_state_handles_escaped_backslash() {
        let state = WordState::new();

        // First backslash opens escape
        let (s1, _) = state.process(&'\\');
        assert_eq!(s1.escape_state, WordEscapeState::Open);

        // Second backslash is escaped, closes the escape
        let (s2, r2) = s1.process(&'\\');
        assert_eq!(r2, WordResult::Continue);
        assert_eq!(s2.escape_state, WordEscapeState::Closed);

        // Next character should behave normally
        let (_s3, r3) = s2.process(&';');
        assert_eq!(r3, WordResult::Terminate);
    }
}
