pub use crate::characters::*;

pub struct Symbol;

impl Symbol {
    pub fn test(current: &char) -> bool {
        return match current {
            OPEN_CURLY => true,
            CLOSE_CURLY => true,
            OPEN_SQUARE => true,
            CLOSE_SQUARE => true,
            OPEN_CURVY => true,
            CLOSE_CURVY => true,
            SEMICOLON => true,
            COLON => true,
            _ => false,
        };
    }

    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}
