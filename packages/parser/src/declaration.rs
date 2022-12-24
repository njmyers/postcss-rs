use crate::characters::*;
use crate::Node;
use crate::NodeBuilder;

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
    pub fn create(prop: Token, tokenizer: &mut Tokenizer, spaces: &[Token]) -> (Node, bool) {
        let mut builder = NodeBuilder::new(&prop, tokenizer, spaces);
        let mut semicolon = false;
        let mut open = false;
        let mut values: Vec<Token> = Vec::new();

        while let Some(token) = tokenizer.next() {
            semicolon = semicolon || token.value == SEMICOLON;

            match token.kind {
                Kind::Symbol if token.value == OPEN_CURLY => {
                    values.push(token);
                    open = true;
                }

                Kind::Symbol if token.value == CLOSE_CURLY => {
                    values.push(token);
                    open = false;
                }

                Kind::Symbol if !open && token.value == SEMICOLON => break,
                Kind::Space if !open && !semicolon && token.value == NEWLINE => break,
                Kind::Symbol if !open => builder.between(&token),
                Kind::Space if !open => builder.between(&token),
                _ => values.push(token),
            }
        }

        builder.declaration(Self::new(prop.value, &Self::combine(values)));

        return (builder.build(tokenizer, &[]), semicolon);
    }

    fn new(prop: &str, value: &str) -> Self {
        return Self {
            prop: String::from(prop),
            value: String::from(value),
        };
    }

    fn combine(tokens: Vec<Token>) -> String {
        let mut value = String::new();

        for token in tokens {
            value.push_str(token.value)
        }

        return value;
    }
}
