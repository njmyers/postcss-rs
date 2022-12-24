use crate::characters::*;
use crate::Data;
use crate::Node;

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
    pub fn new(prop: &str, value: &str) -> Self {
        return Self {
            prop: String::from(prop),
            value: String::from(value),
        };
    }

    pub fn create(prop: Token, tokenizer: &mut Tokenizer, spaces: &[Token]) -> (Node, bool) {
        let mut node = Node::default();
        let mut semicolon = false;

        node.set_start(&prop, tokenizer);
        node.raws.before.push_tokens(&spaces);

        while let Some(token) = tokenizer.next() {
            if token.value == SEMICOLON {
                semicolon = true;
            }

            match token.kind {
                Kind::Symbol if token.value == SEMICOLON => break,
                Kind::Space if token.value == NEWLINE => break,
                Kind::Symbol => node.raws.between.push(&token),
                Kind::Space => node.raws.between.push(&token),
                Kind::String => node.data = Data::Declaration(Self::new(prop.value, token.value)),
                Kind::Word => node.data = Data::Declaration(Self::new(prop.value, token.value)),
                _ => break,
            }
        }

        node.set_end(tokenizer);

        return (node, semicolon);
    }
}
