use crate::characters::*;
use crate::utils;

use std::cmp;

pub struct Escape;

impl Escape {
    /// Return whether a character is a single escape token `\`
    pub fn test(letter: &char) -> bool {
        return letter == BACKWARD_SLASH;
    }

    /// Tokenize the "word" created by escape sequences like `\(`, `\\`, or `\62 `.
    /// CSS escapes can be:
    /// - A backslash followed by any non-hex character (length 2)
    /// - A backslash followed by 1-6 hex digits, optionally followed by whitespace
    pub fn tokenize(letters: &[char]) -> usize {
        let lookbehind: usize = 1;
        let count = letters.len();

        // If we only have the backslash, return 0 (will create 1-char token)
        if count <= 1 {
            return 0;
        }

        // Check if second character is a hex digit
        if letters[1].is_ascii_hexdigit() {
            // Count consecutive hex digits (up to 6)
            let mut hex_count = 0;
            for i in 1..count.min(7) {
                if letters[i].is_ascii_hexdigit() {
                    hex_count += 1;
                } else {
                    break;
                }
            }

            // Check if followed by whitespace (which is part of the escape)
            let after_hex = 1 + hex_count;
            if after_hex < count && letters[after_hex].is_whitespace() {
                // Include the whitespace in the escape sequence
                return hex_count + 1; // +1 for the space (total: \ + hex + space)
            }

            // No trailing space, just backslash + hex digits
            return hex_count; // (total: \ + hex)
        }

        // Not a hex escape - handle backslash sequences like \\, \(, \", etc.
        let mut letters_lookbehind = letters.iter().zip(letters.iter().skip(lookbehind));

        let result = letters_lookbehind.position(|(previous_letter, letter)| {
            return !Self::test(previous_letter) || !Self::test(letter);
        });

        let position = utils::unwrap_position(result, letters);
        let string_length = cmp::min(position + lookbehind, count);

        if string_length.is_multiple_of(2) {
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
