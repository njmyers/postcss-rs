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

/// Tracks the state of word tokenization including escape handling
#[derive(Debug, Clone, Copy)]
struct WordState {
    is_escaped: bool,
}

impl WordState {
    fn new() -> Self {
        Self { is_escaped: false }
    }

    /// Process a character and return the next state and word result
    fn process(&self, letter: &char) -> (Self, WordResult) {
        // Determine if current character should terminate and what next state is
        let (result, next_escaped) = if self.is_escaped {
            // Current char is escaped - don't terminate, next won't be escaped
            (WordResult::Continue, false)
        } else if Escape::test(letter) {
            // Current is backslash - don't terminate, next char will be escaped
            (WordResult::Continue, true)
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
            (word_result, false)
        };

        (Self { is_escaped: next_escaped }, result)
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
    fn word_state_initializes_unescaped() {
        let state = WordState::new();
        assert_eq!(state.is_escaped, false);
    }

    #[test]
    fn word_state_continues_on_normal_character() {
        let state = WordState::new();
        let (next, result) = state.process(&'a');

        assert_eq!(result, WordResult::Continue);
        assert_eq!(next.is_escaped, false);
    }

    #[test]
    fn word_state_terminates_on_whitespace() {
        let state = WordState::new();
        let (next, result) = state.process(&' ');

        assert_eq!(result, WordResult::Terminate);
        assert_eq!(next.is_escaped, false);
    }

    #[test]
    fn word_state_terminates_on_semicolon() {
        let state = WordState::new();
        let (next, result) = state.process(&';');

        assert_eq!(result, WordResult::Terminate);
        assert_eq!(next.is_escaped, false);
    }

    #[test]
    fn word_state_terminates_on_colon() {
        let state = WordState::new();
        let (next, result) = state.process(&':');

        assert_eq!(result, WordResult::Terminate);
        assert_eq!(next.is_escaped, false);
    }

    #[test]
    fn word_state_terminates_on_opening_bracket() {
        let state = WordState::new();
        let (next, result) = state.process(&'(');

        assert_eq!(result, WordResult::Terminate);
        assert_eq!(next.is_escaped, false);
    }

    #[test]
    fn word_state_sets_escaped_on_backslash() {
        let state = WordState::new();
        let (next, result) = state.process(&'\\');

        assert_eq!(result, WordResult::Continue);
        assert_eq!(next.is_escaped, true);
    }

    #[test]
    fn word_state_continues_when_character_is_escaped() {
        let mut state = WordState::new();
        state.is_escaped = true;

        // Even a normally terminating character should continue when escaped
        let (next, result) = state.process(&';');

        assert_eq!(result, WordResult::Continue);
        assert_eq!(next.is_escaped, false); // No longer escaped after consuming the escaped char
    }

    #[test]
    fn word_state_handles_escape_sequence() {
        let state = WordState::new();

        // First character is backslash - sets escaped state
        let (s1, r1) = state.process(&'\\');
        assert_eq!(r1, WordResult::Continue);
        assert_eq!(s1.is_escaped, true);

        // Next character is escaped, so it continues even though it's a semicolon
        let (s2, r2) = s1.process(&';');
        assert_eq!(r2, WordResult::Continue);
        assert_eq!(s2.is_escaped, false);

        // Back to normal - this semicolon should terminate
        let (_s3, r3) = s2.process(&';');
        assert_eq!(r3, WordResult::Terminate);
    }

    #[test]
    fn word_state_handles_escaped_backslash() {
        let state = WordState::new();

        // First backslash
        let (s1, _) = state.process(&'\\');
        assert_eq!(s1.is_escaped, true);

        // Second backslash is escaped
        let (s2, r2) = s1.process(&'\\');
        assert_eq!(r2, WordResult::Continue);
        assert_eq!(s2.is_escaped, false); // The second backslash was escaped, not an escape char

        // Next character should behave normally
        let (_s3, r3) = s2.process(&';');
        assert_eq!(r3, WordResult::Terminate);
    }
}
