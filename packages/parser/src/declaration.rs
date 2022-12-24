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

        while let Some(token) = tokenizer.next() {
            if token.value == SEMICOLON {
                semicolon = true;
            }

            match token.kind {
                Kind::Symbol if token.value == SEMICOLON => break,
                Kind::Space if token.value == NEWLINE => break,
                Kind::Symbol => builder.between(&token),
                Kind::Space => builder.between(&token),
                Kind::String => builder.declaration(Self::new(prop.value, token.value)),
                Kind::Word => builder.declaration(Self::new(prop.value, token.value)),
                _ => break,
            }
        }

        return (builder.build(tokenizer, &[]), semicolon);
    }

    fn new(prop: &str, value: &str) -> Self {
        return Self {
            prop: String::from(prop),
            value: String::from(value),
        };
    }
}
