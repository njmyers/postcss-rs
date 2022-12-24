pub use crate::characters::*;

pub struct OpenCurly;

impl OpenCurly {
    pub fn test(letter: &char) -> bool {
        return matches!(letter, OPEN_CURLY);
    }

    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

pub struct CloseCurly;

impl CloseCurly {
    pub fn test(letter: &char) -> bool {
        return matches!(letter, CLOSE_CURLY);
    }

    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

pub struct OpenSquare;

impl OpenSquare {
    pub fn test(letter: &char) -> bool {
        return matches!(letter, OPEN_SQUARE);
    }

    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

pub struct CloseSquare;

impl CloseSquare {
    pub fn test(letter: &char) -> bool {
        return matches!(letter, CLOSE_SQUARE);
    }

    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

pub struct OpenCurvy;

impl OpenCurvy {
    pub fn test(letter: &char) -> bool {
        return matches!(letter, OPEN_CURVY);
    }

    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

pub struct CloseCurvy;

impl CloseCurvy {
    pub fn test(letter: &char) -> bool {
        return matches!(letter, CLOSE_CURVY);
    }

    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

pub struct Semicolon;

impl Semicolon {
    pub fn test(letter: &char) -> bool {
        return matches!(letter, SEMICOLON);
    }

    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}

pub struct Colon;

impl Colon {
    pub fn test(letter: &char) -> bool {
        return matches!(letter, COLON);
    }

    pub fn tokenize(_: &[char]) -> usize {
        return 0;
    }
}
