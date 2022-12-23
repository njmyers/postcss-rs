pub use crate::characters::*;

pub struct Symbol;

impl Symbol {
    pub fn test(current: &char) -> bool {
        return matches!(
            current,
            OPEN_CURLY
                | CLOSE_CURLY
                | OPEN_SQUARE
                | CLOSE_SQUARE
                | OPEN_CURVY
                | CLOSE_CURVY
                | SEMICOLON
                | COLON
        );
    }

    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}
