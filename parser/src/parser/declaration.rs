use super::Data;
use super::Node;
use super::Source;

use serde::Deserialize;
use serde::Serialize;

use crate::tokenizer::Kind;
use crate::tokenizer::Token;
use crate::tokenizer::Tokenizer;

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
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

    pub fn create(prop: Token, tokenizer: &mut Tokenizer) -> Node {
        let mut node = Node::default();
        node.sources.start = Source::create_start(&prop, tokenizer);

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Symbol if token.value == ";" => break,
                Kind::Symbol => node.raws.between.push(token.value),
                Kind::Space => node.raws.between.push(token.value),
                Kind::Word => node.data = Data::Declaration(Self::new(prop.value, token.value)),
                _ => break,
            }
        }

        node.sources.end = Some(Source::create_end(tokenizer));

        return node;
    }
}
