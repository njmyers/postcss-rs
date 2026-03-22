use crate::characters::*;
use crate::utils;

use std::cmp;

/// Handles CSS hex escape sequences like `\62 olor` (represents "bolor")
pub struct HexEscape;

impl HexEscape {
    /// Tokenize a hex escape sequence starting after the backslash.
    ///
    /// CSS hex escapes are:
    /// - 1-6 hex digits, optionally followed by whitespace
    /// - The whitespace is consumed as part of the escape
    ///
    /// Returns the number of characters to consume (not including the backslash).
    /// Returns 0 if this is not a hex escape.
    pub fn tokenize(letters: &[char]) -> usize {
        let count = letters.len();

        // Need at least one character after the backslash
        if count <= 1 {
            return 0;
        }

        // Check if second character (after \) is a hex digit
        if !letters[1].is_ascii_hexdigit() {
            return 0;
        }

        // Count consecutive hex digits (up to 6)
        let mut hex_count = 0usize;
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
}

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

        // Try to parse as hex escape
        let hex_result = HexEscape::tokenize(letters);
        if hex_result > 0 {
            return hex_result;
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

    #[test]
    fn hex_escape_returns_zero_for_non_hex() {
        let letters: Vec<char> = "\\(".chars().collect();
        assert_eq!(HexEscape::tokenize(&letters[..]), 0);
    }

    #[test]
    fn hex_escape_tokenizes_single_hex_digit() {
        let letters: Vec<char> = "\\6olor".chars().collect();
        assert_eq!(HexEscape::tokenize(&letters[..]), 1); // Just the '6'
    }

    #[test]
    fn hex_escape_tokenizes_multiple_hex_digits() {
        let letters: Vec<char> = "\\62olor".chars().collect();
        assert_eq!(HexEscape::tokenize(&letters[..]), 2); // '62'
    }

    #[test]
    fn hex_escape_tokenizes_hex_with_trailing_space() {
        let letters: Vec<char> = "\\62 olor".chars().collect();
        assert_eq!(HexEscape::tokenize(&letters[..]), 3); // '62 ' (includes space)
    }

    #[test]
    fn hex_escape_tokenizes_max_six_hex_digits() {
        let letters: Vec<char> = "\\123456789".chars().collect();
        assert_eq!(HexEscape::tokenize(&letters[..]), 6); // Only first 6 hex digits
    }

    #[test]
    fn hex_escape_stops_at_non_hex_character() {
        let letters: Vec<char> = "\\12g".chars().collect();
        assert_eq!(HexEscape::tokenize(&letters[..]), 2); // '12' (stops at 'g')
    }

    #[test]
    fn hex_escape_handles_empty_input() {
        let letters: Vec<char> = "\\".chars().collect();
        assert_eq!(HexEscape::tokenize(&letters[..]), 0);
    }

    #[test]
    fn hex_escape_includes_tab_as_whitespace() {
        let letters: Vec<char> = "\\62\tolor".chars().collect();
        assert_eq!(HexEscape::tokenize(&letters[..]), 3); // '62\t' (includes tab)
    }

    #[test]
    fn hex_escape_includes_newline_as_whitespace() {
        let letters: Vec<char> = "\\62\nolor".chars().collect();
        assert_eq!(HexEscape::tokenize(&letters[..]), 3); // '62\n' (includes newline)
    }
}
