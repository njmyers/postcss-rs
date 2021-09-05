use super::escape;
use super::utils;
use std::cmp;

const OPEN_PARENTHESIS: &char = &'(';
const CLOSE_PARENTHESIS: &char = &')';

/**
 * Tokenize the bracketed statement contained within brackets like `(word)`.
 */
pub fn tokenize_brackets(letters: &[char]) -> usize {
  let lookbehind: usize = 1;
  let count = letters.iter().count();
  let mut letters_lookbehind = letters.iter().zip(letters.iter().skip(lookbehind));

  let result = letters_lookbehind.position(|(previous_letter, letter)| {
    return match letter {
      CLOSE_PARENTHESIS if !escape::is_escape_character(previous_letter) => true,
      _ => false,
    };
  });

  let string_length = utils::unwrap_position(result, letters) + lookbehind;
  return cmp::min(string_length, count - lookbehind);
}

/**
 * Return whether a character is a single opening parenthesis token `(`
 */
pub fn is_opening_bracket_token(letter: &char) -> bool {
  return letter == OPEN_PARENTHESIS;
}
