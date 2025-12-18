use std::fmt;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokens;

#[derive(PartialEq, Eq, Debug)]
pub struct Statement {
    pub brackets: BracketDepth,
    pub values: Tokens,
    pub values_clean: Tokens,
}

impl Statement {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn open(&mut self, token: Token) {
        self.brackets.process(&token);
        self.values_clean.push(token.clone());
        self.values.push(token);
    }

    pub fn close(&mut self, token: Token) {
        self.brackets.process(&token);
        self.values_clean.push(token.clone());
        self.values.push(token);
    }

    pub fn push(&mut self, token: Token) {
        self.brackets.process(&token);
        self.values_clean.push(token.clone());
        self.values.push(token);
    }

    pub fn push_raw_only(&mut self, token: Token) {
        self.brackets.process(&token);
        self.values.push(token);
    }

    pub fn is_empty(&self) -> bool {
        return self.values.is_empty();
    }

    pub fn is_open(&self) -> bool {
        return self.brackets.is_open();
    }

    pub fn to_string_clean(&self) -> String {
        return self.values_clean.to_string();
    }

    /// Collect and remove trailing space tokens, returning them as a string
    pub fn collect_trailing_spaces(&mut self) -> String {
        let mut spaces = Vec::new();

        // Collect from values (raw) - in reverse order
        while let Some(last) = self.values.last() {
            if last.kind == Kind::Space {
                spaces.push(self.values.pop().unwrap().value);
            } else {
                break;
            }
        }

        // Also remove from values_clean
        while let Some(last) = self.values_clean.last() {
            if last.kind == Kind::Space {
                self.values_clean.pop();
            } else {
                break;
            }
        }

        // Reverse to get correct order and join
        spaces.reverse();
        spaces.join("")
    }

    /// Push a sliced portion of a token to both values and values_clean
    /// Preserves accurate source positions from the original token
    pub fn push_token_slice(&mut self, token: &Token, start: usize, end: usize) {
        let sliced = token.slice(start, end);
        self.values_clean.push(sliced.clone());
        self.values.push(sliced);
    }
}

impl Default for Statement {
    fn default() -> Self {
        return Self {
            brackets: BracketDepth::new(),
            values: Tokens::new(),
            values_clean: Tokens::new(),
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
    pub ended_by: Option<Kind>,
}

impl StatementTerminator {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn process(&mut self, token: &Token) {
        self.semicolon = self.semicolon || token.kind == Kind::Semicolon;
    }

    pub fn set_ended_by(&mut self, kind: Kind) {
        self.ended_by = Some(kind);
    }
}

impl Default for StatementTerminator {
    fn default() -> Self {
        return Self {
            semicolon: false,
            ended_by: None,
        };
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
            Kind::OpenCurly => self.curly += 1,
            Kind::CloseCurly => self.curly = (self.curly - 1).max(0),
            Kind::OpenCurvy => self.round += 1,
            Kind::CloseCurvy => self.round = (self.round - 1).max(0),
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
