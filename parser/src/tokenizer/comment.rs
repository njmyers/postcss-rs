use super::utils;

const FORWARD_SLASH: &char = &'/';
const ASTERISK: &char = &'*';

pub fn tokenize_comment(letters: &[char]) -> usize {
  let result =
    letters
      .iter()
      .zip(letters.iter().skip(1))
      .position(|(first_letter, second_letter)| {
        return is_closing_comment_token(first_letter, second_letter);
      });

  let position = utils::unwrap_position(result, letters);
  return position;
}

pub fn is_opening_comment_token(first_letter: &char, second_letter: &char) -> bool {
  return first_letter == FORWARD_SLASH && second_letter == ASTERISK;
}

pub fn is_closing_comment_token(first_letter: &char, second_letter: &char) -> bool {
  return first_letter == ASTERISK && second_letter == FORWARD_SLASH;
}

pub fn is_forward_slash(letter: &char) -> bool {
  return letter == FORWARD_SLASH;
}
