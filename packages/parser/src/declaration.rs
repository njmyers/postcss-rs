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
            semicolon = semicolon || token.kind == Kind::Semicolon;

            match token.kind {
                Kind::OpenCurly => {
                    values.push(token);
                    open = true;
                }

                Kind::CloseCurly if open => {
                    values.push(token);
                    open = false;
                }

                Kind::CloseCurly if !open => break,
                Kind::Semicolon if !open => break,
                Kind::Colon if !open => builder.between(&token),

                Kind::Space if !values.is_empty() => values.push(token),
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
