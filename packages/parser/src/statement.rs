use std::fmt;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokens;

#[derive(PartialEq, Eq, Debug)]
pub struct Statement {
    pub brackets: BracketDepth,
    pub values: Tokens,
}

impl Statement {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn open(&mut self, token: Token) {
        self.brackets.process(&token);
        self.values.push(token);
    }

    pub fn close(&mut self, token: Token) {
        self.brackets.process(&token);
        self.values.push(token);
    }

    pub fn push(&mut self, token: Token) {
        self.brackets.process(&token);
        self.values.push(token);
    }

    pub fn is_empty(&self) -> bool {
        return self.values.is_empty();
    }

    pub fn is_open(&self) -> bool {
        return self.brackets.is_open();
    }
}

impl Default for Statement {
    fn default() -> Self {
        return Self {
            brackets: BracketDepth::new(),
            values: Tokens::new(),
        };
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return write!(formatter, "{}", self.values);
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct StatementTerminator {
    pub semicolon: bool,
}

impl StatementTerminator {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn process(&mut self, token: &Token) {
        self.semicolon = self.semicolon || token.kind == Kind::Semicolon;
    }
}

impl Default for StatementTerminator {
    fn default() -> Self {
        return Self { semicolon: false };
    }
}

/// Tracks the depth of different bracket types
#[derive(PartialEq, Eq, Debug, Default)]
pub struct BracketDepth {
    /// Represents the bracket depth of curly or {} type brackets
    curly: i32,
    /// Represents the bracket depth of square or [] type brackets
    square: i32,
    /// Represents the bracket depth of round or () type brackets
    round: i32,
}

impl BracketDepth {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn is_open(&self) -> bool {
        return self.curly > 0 || self.square > 0 || self.round > 0;
    }

    pub fn process(&mut self, token: &Token) {
        match token.kind {
            Kind::OpenCurly | Kind::OpenCurvy => self.curly += 1,
            Kind::CloseCurly | Kind::CloseCurvy => self.curly = (self.curly - 1).max(0),
            Kind::OpenSquare => self.square += 1,
            Kind::CloseSquare => self.square = (self.square - 1).max(0),
            Kind::Brackets => {
                // Brackets token contains both open and close
                // For now, treat as balanced (no net change)
            }
            _ => {}
        }
    }
}
