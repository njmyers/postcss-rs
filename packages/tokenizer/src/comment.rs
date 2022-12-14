use super::utils;

const FORWARD_SLASH: &char = &'/';
const ASTERISK: &char = &'*';

pub struct Comment;

impl Comment {
    /// Tokenize a comment
    pub fn tokenize(letters: &[char]) -> usize {
        let mut iterator = letters.iter().zip(letters.iter().skip(1));
        let result = iterator.position(|(current, next)| Self::test_end(current, next));

        let position = utils::unwrap_position(result, letters);
        return position;
    }

    pub fn test(current: &char, next: &char) -> bool {
        return current == FORWARD_SLASH && next == ASTERISK;
    }

    fn test_end(current: &char, next: &char) -> bool {
        return current == ASTERISK && next == FORWARD_SLASH;
    }

    pub fn is_forward_slash(letter: &char) -> bool {
        return letter == FORWARD_SLASH;
    }
}
