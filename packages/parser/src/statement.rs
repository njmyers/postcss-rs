use std::fmt;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokens;

#[derive(PartialEq, Eq, Debug)]
pub struct Statement<'a> {
    pub open: bool,
    pub values: Tokens<'a>,
}

impl<'a> Statement<'a> {
    pub fn new() -> Self {
        return Self::default();
    }

    pub fn open(&mut self, token: Token<'a>) {
        self.open = true;
        self.values.push(token);
    }

    pub fn close(&mut self, token: Token<'a>) {
        self.open = false;
        self.values.push(token);
    }

    pub fn push(&mut self, token: Token<'a>) {
        self.values.push(token);
    }

    pub fn is_empty(&self) -> bool {
        return self.values.is_empty();
    }
}

impl<'a> Default for Statement<'a> {
    fn default() -> Self {
        return Self {
            open: false,
            values: Tokens::new(),
        };
    }
}

impl<'a> fmt::Display for Statement<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return write!(formatter, "{}", self.values);
    }
}

#[derive(PartialEq, Eq, Debug)]
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
