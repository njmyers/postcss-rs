use crate::bytes::BACKSLASH;
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
    /// Returns the number of bytes to consume (not including the backslash).
    /// Returns 0 if this is not a hex escape.
    pub fn tokenize(bytes: &[u8]) -> usize {
        let count = bytes.len();

        // Need at least one character after the backslash
        if count <= 1 {
            return 0;
        }

        // Check if second character (after \) is a hex digit
        if !bytes[1].is_ascii_hexdigit() {
            return 0;
        }

        // Count consecutive hex digits (up to 6)
        let mut hex_count = 0usize;
        for byte in bytes.iter().take(count.min(7)).skip(1) {
            if byte.is_ascii_hexdigit() {
                hex_count += 1;
            } else {
                break;
            }
        }

        // Check if followed by whitespace (which is part of the escape)
        let after_hex = 1 + hex_count;
        if after_hex < count && bytes[after_hex].is_ascii_whitespace() {
            // Include the whitespace in the escape sequence
            return hex_count + 1; // +1 for the space (total: \ + hex + space)
        }

        // No trailing space, just backslash + hex digits
        hex_count // (total: \ + hex)
    }
}

pub struct Escape;

impl Escape {
    /// Return whether a byte is a backslash `\`
    pub fn test(byte: u8) -> bool {
        byte == BACKSLASH
    }

    /// Tokenize the "word" created by escape sequences like `\(`, `\\`, or `\62 `.
    /// CSS escapes can be:
    /// - A backslash followed by any non-hex character (length 2)
    /// - A backslash followed by 1-6 hex digits, optionally followed by whitespace
    pub fn tokenize(bytes: &[u8]) -> usize {
        let lookbehind: usize = 1;
        let count = bytes.len();

        // If we only have the backslash, return 0 (will create 1-char token)
        if count <= 1 {
            return 0;
        }

        // Try to parse as hex escape
        let hex_result = HexEscape::tokenize(bytes);
        if hex_result > 0 {
            return hex_result;
        }

        // Not a hex escape - handle backslash sequences like \\, \(, \", etc.
        let mut bytes_lookbehind = bytes.iter().zip(bytes.iter().skip(lookbehind));

        let result = bytes_lookbehind
            .position(|(previous_byte, byte)| !Self::test(*previous_byte) || !Self::test(*byte));

        let position = utils::unwrap_position(result, bytes.len());
        let string_length = cmp::min(position + lookbehind, count);

        if string_length.is_multiple_of(2) {
            return utils::back(string_length);
        }

        string_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_escapes_backslash() {
        assert_eq!(Escape::tokenize(b"\\\\'"), 1);
    }

    #[test]
    fn it_escapes_control() {
        assert_eq!(Escape::tokenize(b"\\(\\"), 1);
    }

    #[test]
    fn it_escapes_backslash_sequences() {
        assert_eq!(Escape::tokenize(b"\\\\\\\\"), 3);
    }

    #[test]
    fn hex_escape_returns_zero_for_non_hex() {
        assert_eq!(HexEscape::tokenize(b"\\("), 0);
    }

    #[test]
    fn hex_escape_tokenizes_single_hex_digit() {
        assert_eq!(HexEscape::tokenize(b"\\6olor"), 1); // Just the '6'
    }

    #[test]
    fn hex_escape_tokenizes_multiple_hex_digits() {
        assert_eq!(HexEscape::tokenize(b"\\62olor"), 2); // '62'
    }

    #[test]
    fn hex_escape_tokenizes_hex_with_trailing_space() {
        assert_eq!(HexEscape::tokenize(b"\\62 olor"), 3); // '62 ' (includes space)
    }

    #[test]
    fn hex_escape_tokenizes_max_six_hex_digits() {
        assert_eq!(HexEscape::tokenize(b"\\123456789"), 6); // Only first 6 hex digits
    }

    #[test]
    fn hex_escape_stops_at_non_hex_character() {
        assert_eq!(HexEscape::tokenize(b"\\12g"), 2); // '12' (stops at 'g')
    }

    #[test]
    fn hex_escape_handles_empty_input() {
        assert_eq!(HexEscape::tokenize(b"\\"), 0);
    }

    #[test]
    fn hex_escape_includes_tab_as_whitespace() {
        assert_eq!(HexEscape::tokenize(b"\\62\tolor"), 3); // '62\t' (includes tab)
    }

    #[test]
    fn hex_escape_includes_newline_as_whitespace() {
        assert_eq!(HexEscape::tokenize(b"\\62\nolor"), 3); // '62\n' (includes newline)
    }
}
