use serde::Deserialize;
use serde::Serialize;

use crate::Data;
use crate::Declaration;
use crate::Node;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub selector: String,
}

impl Rule {
    pub fn new(selector: &str) -> Self {
        return Self {
            selector: String::from(selector),
        };
    }

    pub fn create(token: Token, tokenizer: &mut Tokenizer, spaces: &[Token]) -> Node {
        let mut node = Node::new(Data::Rule(Self::new(token.value)));
        node.set_start(&token, tokenizer);
        node.raws.before.push_str("");
        node.raws.before.push_tokens(spaces);

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Space => node.raws.between.push(&token),
                Kind::Symbol => break,
                _ => break,
            };
        }

        let mut spaces: Vec<Token> = Vec::new();

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Word => {
                    let (child, semicolon) = Declaration::create(token, tokenizer, &spaces);

                    if semicolon {
                        node.raws.semicolon.set_true();
                    }

                    node.append(child);
                    spaces.clear();
                }
                Kind::Space => spaces.push(token),
                Kind::Symbol => break,
                _ => break,
            };
        }

        node.set_end(tokenizer);
        node.raws.after.push_tokens(&spaces);

        return node;
    }
}
