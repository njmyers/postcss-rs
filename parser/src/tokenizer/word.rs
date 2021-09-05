use super::brackets;
use super::comment;
use super::control;
use super::string;
use super::utils;
use super::whitespace;
use std::cmp;

/** CSS Leading Word Markers */
const EXCLAMATION_POINT: &char = &'!';
const AT_CHAR: &char = &'@';

pub fn tokenize_word(letters: &[char]) -> usize {
  let lookbehind: usize = 1;
  let count = letters.iter().count();
  let mut letters_lookbehind = letters.iter().zip(letters.iter().skip(lookbehind));

  let result = letters_lookbehind.position(|(_, letter)| {
    return match letter {
      letter if whitespace::is_whitespace(letter) => true,
      letter if control::is_control(letter) => true,
      letter if string::is_single_string_token(letter) => true,
      letter if string::is_double_string_token(letter) => true,
      letter if brackets::is_opening_bracket_token(letter) => true,
      letter if comment::is_forward_slash(letter) => true,
      letter if is_leading_word_token(letter) => true,
      _ => false,
    };
  });

  let string_length = utils::unwrap_position(result, letters);
  return cmp::min(string_length, count - lookbehind);
}

pub fn is_at_word_token(letter: &char) -> bool {
  return letter == AT_CHAR;
}

pub fn is_leading_word_token(letter: &char) -> bool {
  return match letter {
    EXCLAMATION_POINT => true,
    AT_CHAR => true,
    _ => false,
  };
}
