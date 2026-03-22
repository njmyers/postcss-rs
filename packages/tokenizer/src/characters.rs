//! Character constants used throughout the tokenizer.
//!
//! All CSS-significant characters are defined here as `&char` constants so
//! they can be used in `matches!` patterns and comparisons without repeating
//! raw literals throughout the codebase.

pub const SINGLE_QUOTE: &char = &'\'';
pub const DOUBLE_QUOTE: &char = &'"';
pub const OPEN_SQUARE: &char = &'[';
pub const CLOSE_SQUARE: &char = &']';
pub const OPEN_CURVY: &char = &'(';
pub const CLOSE_CURVY: &char = &')';
pub const OPEN_CURLY: &char = &'{';
pub const CLOSE_CURLY: &char = &'}';
pub const SEMICOLON: &char = &';';
pub const COLON: &char = &':';
pub const HORIZONTAL_TAB: &char = &'\u{0009}';
pub const NEWLINE: &char = &'\u{000A}';
pub const VERTICAL_TAB: &char = &'\u{000B}';
pub const FORM_FEED: &char = &'\u{000C}';
pub const CARRIAGE_RETURN: &char = &'\u{000D}';
pub const SPACE: &char = &' ';
pub const NULL_CHARACTER: &char = &'\u{0000}';
pub const FORWARD_SLASH: &char = &'/';
pub const ASTERISK: &char = &'*';
pub const BACKWARD_SLASH: &char = &'\\';
pub const EXCLAMATION_POINT: &char = &'!';
pub const AT_CHAR: &char = &'@';
pub const DASH: &char = &'-';
