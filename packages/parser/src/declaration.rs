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

        while let Some(token) = tokenizer.next() {
            terminator.process(&token);

            match token.kind {
                Kind::OpenCurly => statement.open(token),
                Kind::CloseCurly if statement.is_open() => statement.close(token),

                Kind::CloseCurly if !statement.is_open() => break,
                Kind::Semicolon if !statement.is_open() => break,
                Kind::Colon if !statement.is_open() => builder.between(&token),

                Kind::Space if !statement.is_empty() => statement.push(token),
                Kind::Space if !statement.is_open() => builder.between(&token),

                _ => statement.push(token),
            }

            if let Some(token) = tokenizer.peek() {
                match token.kind {
                    Kind::Space if token.value == characters::NEWLINE && !statement.is_open() => break,
                    _ => (),
                }
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
