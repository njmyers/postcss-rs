use std::fmt;

use crate::characters;
use crate::Node;
use crate::NodeBuilder;

use serde::Deserialize;
use serde::Serialize;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;
use postcss_rs_tokenizer::Tokens;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Declaration {
    pub prop: String,
    pub value: String,
}

impl Declaration {
    pub fn create(
        prop: Token,
        tokenizer: &mut Tokenizer,
        spaces: &[Token],
    ) -> (Node, StatementTerminator) {
        let mut builder = NodeBuilder::new(&prop, tokenizer, spaces);
        let mut statement = Statement::new();
        let mut terminator = StatementTerminator::new();

        while let Some(token) = tokenizer.next() {
            terminator.process(&token);

            match token.kind {
                Kind::OpenCurly => statement.open(token),
                Kind::CloseCurly if statement.open => statement.close(token),

                Kind::CloseCurly if !statement.open => break,
                Kind::Semicolon if !statement.open => break,
                Kind::Colon if !statement.open => builder.between(&token),

                Kind::Space if !statement.is_empty() => statement.push(token),
                Kind::Space if !statement.open => builder.between(&token),

                _ => statement.push(token),
            }

            if let Some(token) = tokenizer.peek() {
                match token.kind {
                    Kind::Space if token.value == characters::NEWLINE && !statement.open => break,
                    _ => (),
                }
            }
        }

        let declaration = Self::new(prop.value, &statement.to_string());
        builder.declaration(declaration);

        return (builder.build(tokenizer, &[]), terminator);
    }

    fn new(prop: &str, value: &str) -> Self {
        return Self {
            prop: String::from(prop),
            value: String::from(value),
        };
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Statement<'a> {
    pub open: bool,
    pub values: Tokens<'a>,
}

impl<'a> Statement<'a> {
    fn new() -> Self {
        return Self {
            open: false,
            values: Tokens::new(),
        };
    }

    fn open(&mut self, token: Token<'a>) {
        self.open = true;
        self.values.push(token);
    }

    fn close(&mut self, token: Token<'a>) {
        self.open = false;
        self.values.push(token);
    }

    fn push(&mut self, token: Token<'a>) {
        self.values.push(token);
    }

    fn is_empty(&self) -> bool {
        return self.values.is_empty();
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
    fn new() -> Self {
        return Self::default();
    }

    fn process(&mut self, token: &Token) {
        self.semicolon = self.semicolon || token.kind == Kind::Semicolon;
    }
}

impl Default for StatementTerminator {
    fn default() -> Self {
        return Self { semicolon: false };
    }
}
