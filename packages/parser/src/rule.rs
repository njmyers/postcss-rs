use serde::Deserialize;
use serde::Serialize;

use super::Data;
use super::Declaration;
use super::Node;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub selector: String,
}

impl Rule {
    pub fn new(selector: &str) -> Self {
        return Self {
            selector: String::from(selector),
        };
    }

    pub fn create(token: Token, tokenizer: &mut Tokenizer) -> Node {
        let mut node = Node::new(Data::Rule(Self::new(token.value)));
        node.set_start(&token, tokenizer);

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Space => node.raws.between.push(token.value),
                Kind::Symbol => break,
                _ => break,
            };
        }

        let mut spaces: Vec<Token> = Vec::new();

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Space => spaces.push(token),
                Kind::Word => node.append(Declaration::create(token, tokenizer)),
                Kind::Symbol => break,
                _ => break,
            };
        }

        node.set_end(tokenizer);

        return node;
    }
}
