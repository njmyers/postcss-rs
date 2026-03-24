//! Named byte constants for every CSS-significant character.
//!
//! All sub-tokenizers operate on `&[u8]` slices.  Using named constants here
//! avoids scattering raw byte literals (`b'{'`, `0x0B`, …) throughout the
//! codebase and makes intent immediately clear at each call site.

/// Single quote `'`
pub const SINGLE_QUOTE: u8 = b'\'';
/// Double quote `"`
pub const DOUBLE_QUOTE: u8 = b'"';
/// Opening square bracket `[`
pub const OPEN_SQUARE: u8 = b'[';
/// Closing square bracket `]`
pub const CLOSE_SQUARE: u8 = b']';
/// Opening curly brace `{`
pub const OPEN_CURLY: u8 = b'{';
/// Closing curly brace `}`
pub const CLOSE_CURLY: u8 = b'}';
/// Opening parenthesis `(`
pub const OPEN_PAREN: u8 = b'(';
/// Closing parenthesis `)`
pub const CLOSE_PAREN: u8 = b')';
/// Semicolon `;`
pub const SEMICOLON: u8 = b';';
/// Colon `:`
pub const COLON: u8 = b':';
/// Horizontal tab (U+0009)
pub const TAB: u8 = b'\t';
/// Line feed / newline (U+000A)
pub const NEWLINE: u8 = b'\n';
/// Vertical tab (U+000B)
pub const VERTICAL_TAB: u8 = b'\x0B';
/// Form feed (U+000C)
pub const FORM_FEED: u8 = b'\x0C';
/// Carriage return (U+000D)
pub const CARRIAGE_RETURN: u8 = b'\r';
/// Space (U+0020)
pub const SPACE: u8 = b' ';
/// Null byte (U+0000); used as a sentinel for "no next byte"
pub const NULL_BYTE: u8 = 0;
/// Forward slash `/`
pub const FORWARD_SLASH: u8 = b'/';
/// Asterisk `*`
pub const ASTERISK: u8 = b'*';
/// Backslash `\`
pub const BACKSLASH: u8 = b'\\';
/// Exclamation mark `!`
pub const EXCLAMATION: u8 = b'!';
/// At-sign `@`
pub const AT: u8 = b'@';
/// Hyphen-minus `-`
pub const DASH: u8 = b'-';
