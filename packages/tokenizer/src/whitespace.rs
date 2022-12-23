use crate::characters::*;
use crate::utils;

pub struct Whitespace;

impl Whitespace {
    pub fn test(letter: &char) -> bool {
        return matches!(
            letter,
            HORIZONTAL_TAB | NEWLINE | VERTICAL_TAB | FORM_FEED | CARRIAGE_RETURN | SPACE
        );
    }

    pub fn tokenize(letters: &[char]) -> usize {
        let result = letters.iter().position(|letter| {
            return !matches!(letter, letter if Self::test(letter));
        });

        let position = utils::unwrap_position(result, letters);
        return utils::back(position);
    }
}

pub struct Newline;

impl Newline {
    pub fn test(letter: &char) -> bool {
        return matches!(letter, NEWLINE | CARRIAGE_RETURN);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_tests_newline_character() {
        assert!(Newline::test(&'\n'));
    }
}
