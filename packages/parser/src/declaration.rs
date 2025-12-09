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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub important: Option<bool>,
    pub value: String,
}

impl Declaration {
    pub fn create(
        prop: Token,
        tokenizer: &mut Tokenizer,
        spaces: &[Token],
    ) -> (Node, StatementTerminator) {
        let mut builder = NodeBuilder::new(&prop, tokenizer, spaces);
        builder.ensure_empty_before();
        builder.ensure_empty_between();
        let mut statement = Statement::new();
        let mut terminator = StatementTerminator::new();
        let mut pending_spaces: Vec<Token> = Vec::new(); // Track spaces between colon and first non-space token
        let mut important_flag = false;
        let mut important_raw = String::new();

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
                Kind::OpenCurly => statement.open(token),
                Kind::CloseCurly if statement.is_open() => statement.close(token),

                // Semicolon ends declaration and is consumed
                Kind::Semicolon if !statement.is_open() => {
                    terminator.set_ended_by(Kind::Semicolon);
                    break;
                }
                Kind::Colon if !statement.is_open() => builder.between(&token),

                // Comment handling
                Kind::Comment => {
                    // Check if clean value is empty (ignore raw-only tokens)
                    if statement.values_clean.is_empty() && !statement.is_open() {
                        // Comment before any real content - check if it's before a function call
                        match tokenizer.peek() {
                            Some(next) if next.kind == Kind::Brackets && next.value.starts_with('(') => {
                                // Comment before function - move pending to between, keep comment
                                if !pending_spaces.is_empty() {
                                    for space in pending_spaces.drain(..) {
                                        builder.between(&space);
                                    }
                                }
                                statement.push(token);
                            }
                            _ => {
                                // Regular comment - strip it, keep pending spaces in pending
                                statement.push_raw_only(token);
                            }
                        }
                    } else {
                        // Comment inside value - check if it's before an opening paren
                        match tokenizer.peek() {
                            Some(next) if next.kind == Kind::Brackets && next.value.starts_with('(') => {
                                statement.push(token);
                            }
                            _ => {
                                // Regular inline comment - only add to raw, not clean
                                statement.push_raw_only(token);
                            }
                        }
                    }
                }

                Kind::Space => {
                    // Check if clean value is empty (ignore raw-only tokens like stripped comments)
                    if statement.values_clean.is_empty() && !statement.is_open() {
                        // Space before any value content - hold in pending
                        pending_spaces.push(token);
                    } else {
                        // Space within value - add to statement
                        statement.push(token);
                    }
                }

                // Check for !important
                Kind::Word if token.value.starts_with("!important") => {
                    important_flag = true;
                    // Pending spaces before !important should stay in value, not in important_raw
                    if !pending_spaces.is_empty() {
                        for space in pending_spaces.drain(..) {
                            statement.push(space);
                        }
                    }
                    important_raw.push_str(&token.value);
                    // Don't add to statement - strip from value
                }

                _ => {
                    // Non-space token - decide what to do with pending spaces
                    if !pending_spaces.is_empty() {
                        // Add pending spaces to between (they're before actual content)
                        for space in pending_spaces.drain(..) {
                            builder.between(&space);
                        }
                    }
                    statement.push(token);
                }
            }
        }

        // End of declaration - handle any remaining pending spaces
        if !pending_spaces.is_empty() {
            // Declaration ended with only spaces - they should be the value
            for space in pending_spaces.iter() {
                statement.push(space.clone());
            }
        }

        // Use clean value (without comments) as the main value
        let clean_value = statement.to_string_clean();
        let important = if important_flag { Some(true) } else { None };
        let declaration = Self::new(&prop.value, &clean_value, important);
        builder.declaration(declaration);

        // If raw differs from clean (inline comments were stripped), store both in raws.value
        let raw_value = statement.to_string();
        if raw_value != clean_value {
            builder.value_raw(&clean_value, &raw_value);
        }

        // If important flag was set, store the raw important string
        if important_flag {
            builder.important_raw(&important_raw);
        }

        return (builder.build(tokenizer, &[]), terminator);
    }

    fn new(prop: &str, value: &str, important: Option<bool>) -> Self {
        return Self {
            prop: String::from(prop),
            important,
            value: String::from(value),
        };
    }
}
