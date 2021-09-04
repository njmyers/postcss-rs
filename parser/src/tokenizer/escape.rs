pub const BACKWARD_SLASH: &char = &'\\';

pub fn is_escape_character(letter: &char) -> bool {
  return letter == BACKWARD_SLASH;
}
