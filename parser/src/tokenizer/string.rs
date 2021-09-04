use super::escape;
use super::utils;
use std::cmp;

/** CSS String Markers */
pub const SINGLE_QUOTE: &char = &'\'';
pub const DOUBLE_QUOTE: &char = &'"';

/**
 * Tokenize the string contained within single quotation marks like `'word'`.
 */
pub fn tokenize_single_string(letters: &[char]) -> usize {
  let lookbehind: usize = 1;
  let count = letters.iter().count();
  let mut letters_lookbehind = letters
    .iter()
    .zip(letters.iter().skip(lookbehind))
    .skip(lookbehind);

  let result = letters_lookbehind.position(|(previous_letter, letter)| {
    return match letter {
      SINGLE_QUOTE if !escape::is_escape_character(previous_letter) => true,
      _ => false,
    };
  });

  let string_length = utils::unwrap_position(result, letters) + 2;
  return cmp::min(string_length, count - lookbehind);
}

/**
 * Tokenize the string contained within single quotation marks like `"word"`.
 */
pub fn tokenize_double_string(letters: &[char]) -> usize {
  let lookbehind: usize = 1;
  let count = letters.iter().count();
  let mut letters_lookbehind = letters
    .iter()
    .zip(letters.iter().skip(lookbehind))
    .skip(lookbehind);

  let result = letters_lookbehind.position(|(previous_letter, letter)| {
    return match letter {
      DOUBLE_QUOTE if !escape::is_escape_character(previous_letter) => true,
      _ => false,
    };
  });

  let string_length = utils::unwrap_position(result, letters) + 2;
  return cmp::min(string_length, count - lookbehind);
}

/**
 * Return whether a character is a single string token `'`
 */
pub fn is_single_string_token(letter: &char) -> bool {
  return match letter {
    SINGLE_QUOTE => true,
    _ => false,
  };
}

/**
 * Return whether a character is a double string token `"`
 */
pub fn is_double_string_token(letter: &char) -> bool {
  return match letter {
    DOUBLE_QUOTE => true,
    _ => false,
  };
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_identifies_a_single_quote() {
    assert_eq!(is_single_string_token(&'\''), true)
  }

  #[test]
  fn it_does_not_identify_a_double_quote() {
    assert_eq!(is_single_string_token(&'"'), false)
  }

  #[test]
  fn it_identifies_a_double_quote() {
    assert_eq!(is_double_string_token(&'"'), true)
  }

  #[test]
  fn it_does_not_identify_a_single_quote() {
    assert_eq!(is_double_string_token(&'\''), false)
  }

  #[test]
  fn it_tokenizes_double_string() {
    let letters: Vec<char> = "\"'hi'\"".chars().collect();
    assert_eq!(tokenize_double_string(&letters[..]), 5);
  }

  #[test]
  fn it_tokenizes_single_string() {
    let letters: Vec<char> = "'\"hi\"'".chars().collect();
    assert_eq!(tokenize_single_string(&letters[..]), 5);
  }
}
