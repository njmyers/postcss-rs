use crate::characters::*;
use crate::utils;

pub struct Whitespace;

impl Whitespace {
    pub fn test(letter: &char) -> bool {
        return match letter {
            HORIZONTAL_TAB => true,
            NEWLINE => true,
            VERTICAL_TAB => true,
            FORM_FEED => true,
            CARRIAGE_RETURN => true,
            SPACE => true,
            _ => false,
        };
    }

    pub fn tokenize(letters: &[char]) -> usize {
        let result = letters.iter().position(|letter| {
            return match letter {
                letter if Self::test(letter) => false,
                _ => true,
            };
        });

        let position = utils::unwrap_position(result, letters);
        return utils::back(position);
    }
}

pub struct Newline;

impl Newline {
    pub fn test(letter: &char) -> bool {
        return match letter {
            NEWLINE => true,
            CARRIAGE_RETURN => true,
            _ => false,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_tests_newline_character() {
        assert_eq!(Newline::test(&'\n'), true);
    }

}