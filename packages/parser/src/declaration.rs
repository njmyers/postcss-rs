use crate::characters;
use crate::Node;
use crate::NodeBuilder;
use crate::Statement;
use crate::StatementTerminator;

use serde::Deserialize;
use serde::Serialize;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;

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

        loop {
            // Peek at next token to check for terminators
            if let Some(peeked) = tokenizer.peek() {
                match peeked.kind {
                    // CloseCurly ends declaration without being consumed (belongs to parent)
                    Kind::CloseCurly if !statement.is_open() => {
                        terminator.set_ended_by(Kind::CloseCurly);
                        break;
                    }
                    // Space with newline ends declaration when not open
                    Kind::Space if peeked.value == characters::NEWLINE && !statement.is_open() => break,

                    // Check for trailing spaces before CloseCurly
                    // We need two-token lookahead: current space + what follows
                    // Use peek_nth(2) to avoid consuming tokens and preserve correct source positions
                    Kind::Space if !statement.is_empty() && !statement.is_open() => {
                        match tokenizer.peek_nth(2) {
                            Some(next) if next.kind == Kind::CloseCurly => {
                                // Space followed by CloseCurly - trailing space, don't consume
                                terminator.set_ended_by(Kind::CloseCurly);
                                break;
                            }
                            _ => {} // Not trailing, will be consumed normally
                        }
                    }
                    _ => {}
                }
            }

            // Consume the token
            let token = match tokenizer.next() {
                Some(t) => t,
                None => break,
            };

            terminator.process(&token);

            match token.kind {
                Kind::OpenCurly => statement.open(token),
                Kind::CloseCurly if statement.is_open() => statement.close(token),

                // Semicolon ends declaration and is consumed
                Kind::Semicolon if !statement.is_open() => {
                    terminator.set_ended_by(Kind::Semicolon);
                    break;
                }
                Kind::Colon if !statement.is_open() => builder.between(&token),

                Kind::Space if !statement.is_empty() => statement.push(token),
                Kind::Space if !statement.is_open() => builder.between(&token),

                _ => statement.push(token),
            }
        }

        let declaration = Self::new(&prop.value, &statement.to_string());
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
