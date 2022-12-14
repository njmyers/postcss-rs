use super::*;
use std::cmp;

/// CSS String Markers
const SINGLE_QUOTE: &char = &'\'';
const DOUBLE_QUOTE: &char = &'"';

// Representation of a single string
pub struct SingleString;

/// Implementation of single string test and tokenize methods.
///
/// # Example
///
/// ```
/// assert_eq!(SingleString::test("'"), true);
/// ```
///
impl SingleString {
    /// Return whether a character is a single string token `'`
    pub fn test(letter: &char) -> bool {
        return match letter {
            SINGLE_QUOTE => true,
            _ => false,
        };
    }

    /// Tokenize the string contained within single quotation marks like `'word'`.
    pub fn tokenize(letters: &[char]) -> usize {
        let lookbehind: usize = 1;
        let count = letters.iter().count();
        let mut letters_lookbehind = letters.iter().zip(letters.iter().skip(lookbehind));

        let result = letters_lookbehind.position(|(previous_letter, letter)| {
            return match letter {
                SINGLE_QUOTE if !Escape::test(previous_letter) => true,
                _ => false,
            };
        });

        let string_length = utils::unwrap_position(result, letters) + lookbehind;
        return cmp::min(string_length, count - lookbehind);
    }
}

pub struct DoubleString;

impl DoubleString {
    /// Return whether a character is a double string token `"`
    pub fn test(letter: &char) -> bool {
        return match letter {
            DOUBLE_QUOTE => true,
            _ => false,
        };
    }

    /// Tokenize the string contained within single quotation marks like `"word"`.
    pub fn tokenize(letters: &[char]) -> usize {
        let lookbehind: usize = 1;
        let count = letters.iter().count();
        let mut letters_lookbehind = letters.iter().zip(letters.iter().skip(lookbehind));

        let result = letters_lookbehind.position(|(previous_letter, letter)| {
            return match letter {
                DOUBLE_QUOTE if !Escape::test(previous_letter) => true,
                _ => false,
            };
        });

        let string_length = utils::unwrap_position(result, letters) + lookbehind;
        return cmp::min(string_length, count - lookbehind);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_identifies_a_single_quote() {
        assert_eq!(SingleString::test(&'\''), true)
    }

    #[test]
    fn it_does_not_identify_a_double_quote() {
        assert_eq!(SingleString::test(&'"'), false)
    }

    #[test]
    fn it_identifies_a_double_quote() {
        assert_eq!(DoubleString::test(&'"'), true)
    }

    #[test]
    fn it_does_not_identify_a_single_quote() {
        assert_eq!(DoubleString::test(&'\''), false)
    }

    #[test]
    fn it_tokenizes_double_string() {
        let letters: Vec<char> = "\"'hi'\"".chars().collect();
        assert_eq!(DoubleString::tokenize(&letters[..]), 5);
    }

    #[test]
    fn it_tokenizes_single_string() {
        let letters: Vec<char> = "'\"hi\"'".chars().collect();
        assert_eq!(SingleString::tokenize(&letters[..]), 5);
    }
}
