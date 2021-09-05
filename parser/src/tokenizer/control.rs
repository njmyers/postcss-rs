/** CSS Control Characters */
const OPEN_SQUARE_BRACKET: &char = &'[';
const CLOSE_SQUARE_BRACKET: &char = &']';
const OPEN_CURLY: &char = &'{';
const CLOSE_CURLY: &char = &'}';
const SEMICOLON: &char = &';';
const COLON: &char = &':';

pub fn is_control(letter: &char) -> bool {
  return match letter {
    OPEN_CURLY => true,
    CLOSE_CURLY => true,
    OPEN_SQUARE_BRACKET => true,
    CLOSE_SQUARE_BRACKET => true,
    SEMICOLON => true,
    COLON => true,
    _ => false,
  };
}
