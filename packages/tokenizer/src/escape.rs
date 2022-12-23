use crate::characters::*;
use crate::utils;

use std::cmp;

pub struct Escape;

impl Escape {
    /// Return whether a character is a single escape token `\`
    pub fn test(letter: &char) -> bool {
        return letter == BACKWARD_SLASH;
    }

    /// Tokenize the "word" created by a series of escape tokens like `\(` or `\\\\`.
    pub fn tokenize(letters: &[char]) -> usize {
        let lookbehind: usize = 1;
        let count = letters.len();
        let mut letters_lookbehind = letters.iter().zip(letters.iter().skip(lookbehind));

        let result = letters_lookbehind.position(|(previous_letter, letter)| {
            return !Self::test(previous_letter) || !Self::test(letter);
        });

        let position = utils::unwrap_position(result, letters);
        let string_length = cmp::min(position + lookbehind, count);

        if string_length % 2 == 0 {
            return utils::back(string_length);
        }

        return string_length;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_escapes_backslash() {
        let letters: Vec<char> = "\\\\'".chars().collect();
        assert_eq!(Escape::tokenize(&letters[..]), 1);
    }

    #[test]
    fn it_escapes_control() {
        let letters: Vec<char> = "\\(\\".chars().collect();
        assert_eq!(Escape::tokenize(&letters[..]), 1);
    }

    #[test]
    fn it_escapes_backslash_sequences() {
        let letters: Vec<char> = "\\\\\\\\".chars().collect();
        assert_eq!(Escape::tokenize(&letters[..]), 3);
    }
}
