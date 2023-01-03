use serde::Deserialize;
use serde::Serialize;

use crate::Declaration;
use crate::Node;
use crate::NodeBuilder;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;
use postcss_rs_tokenizer::Tokens;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct AtRule {
    pub name: String,
    pub params: Option<String>,
}

impl AtRule {
    pub fn create(token: Token, tokenizer: &mut Tokenizer, spaces: &[Token]) -> Node {
        let mut builder = NodeBuilder::new(&token, tokenizer, spaces);
        builder.ensure_empty_before();

        let mut data = Self::new(&token.value[1..]);
        let mut params = Tokens::new();

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Space => builder.between(&token),
                Kind::Word => params.push(token),
                Kind::Brackets => params.push(token),
                Kind::OpenCurly => break,
                Kind::Colon => params.push(token),
                Kind::Semicolon => params.push(token),
                _ => break,
            };
        }

        data.params = Some(params.to_string());

        let mut spaces: Vec<Token> = Vec::new();

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Word => {
                    let (child, terminator) = Declaration::create(token, tokenizer, &spaces);
                    builder.terminator(&terminator);
                    builder.append(child);
                    spaces.clear();
                }
                Kind::Space => spaces.push(token),
                Kind::CloseCurly => break,
                _ => break,
            };
        }

        builder.at_rule(data);

        return builder.build(tokenizer, &spaces);
    }

    fn new(name: &str) -> Self {
        return Self {
            name: String::from(name),
            params: None,
        };
    }
}
