use super::utils;

pub struct Space;

/// CSS Whitespace Characters
const HORIZONTAL_TAB: &char = &'\u{0009}';
const NEWLINE: &char = &'\u{000A}';
const VERTICAL_TAB: &char = &'\u{000B}';
const FORM_FEED: &char = &'\u{000C}';
const CARRIAGE_RETURN: &char = &'\u{000D}';
const SPACE: &char = &' ';

pub fn is_whitespace(letter: &char) -> bool {
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

pub fn tokenize_whitespace(letters: &[char]) -> usize {
    let result = letters.iter().position(|letter| {
        return match letter {
            letter if is_whitespace(letter) => false,
            _ => true,
        };
    });

    let position = utils::unwrap_position(result, letters);
    return utils::back(position);
}
