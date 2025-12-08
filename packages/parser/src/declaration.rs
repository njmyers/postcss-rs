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
        let mut statement_no_comments = Statement::new();
        let mut has_inline_comments = false;
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

                    // Space with newline followed by comment or CloseCurly ends declaration
                    Kind::Space if peeked.value.starts_with(characters::NEWLINE) && !statement.is_open() => {
                        match tokenizer.peek_nth(2) {
                            Some(next) if next.kind == Kind::Comment || next.kind == Kind::CloseCurly => break,
                            _ => {} // Not before terminator, continue
                        }
                    }

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
                Kind::OpenCurly => {
                    statement.open(token.clone());
                    statement_no_comments.open(token);
                }
                Kind::CloseCurly if statement.is_open() => {
                    statement.close(token.clone());
                    statement_no_comments.close(token);
                }

                // Semicolon ends declaration and is consumed
                Kind::Semicolon if !statement.is_open() => {
                    terminator.set_ended_by(Kind::Semicolon);
                    break;
                }
                Kind::Colon if !statement.is_open() => builder.between(&token),

                // Comment inside value - check if it's before an opening paren
                Kind::Comment if !statement.is_empty() => {
                    statement.push(token.clone());

                    // Check if next token is opening paren - if so, keep comment in both versions
                    match tokenizer.peek() {
                        Some(next) if next.kind == Kind::Brackets && next.value.starts_with('(') => {
                            // Comment before function call - keep it
                            statement_no_comments.push(token);
                        }
                        _ => {
                            // Regular inline comment - strip it
                            has_inline_comments = true;
                            // Don't add to statement_no_comments
                        }
                    }
                }

                Kind::Space if !statement.is_empty() => {
                    statement.push(token.clone());
                    statement_no_comments.push(token);
                }
                Kind::Space if !statement.is_open() => builder.between(&token),

                _ => {
                    statement.push(token.clone());
                    statement_no_comments.push(token);
                }
            }
        }

        // Use clean value (without comments) as the main value
        let clean_value = statement_no_comments.to_string();
        let declaration = Self::new(&prop.value, &clean_value);
        builder.declaration(declaration);

        // If there were inline comments, store both raw and clean in raws.value
        if has_inline_comments {
            let raw_value = statement.to_string();
            builder.value_raw(&clean_value, &raw_value);
        }

        return (builder.build(tokenizer, &[]), terminator);
    }

    fn new(prop: &str, value: &str) -> Self {
        return Self {
            prop: String::from(prop),
            value: String::from(value),
        };
    }
}
