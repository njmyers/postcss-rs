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
        builder.ensure_empty_after_name();

        let mut data = Self::new(&token.value[1..]);
        let mut params = Tokens::new();
        let mut spaces = Tokens::new();

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Space if params.is_empty() => spaces.push(token),
                Kind::Space => builder.between(&token),
                Kind::Word => params.push(token),
                Kind::Brackets => params.push(token),
                Kind::OpenCurly => break,
                Kind::Colon => params.push(token),
                Kind::Semicolon => params.push(token),
                _ => break,
            };
        }

        data.set_params(&params);
        data.set_params_spaces(&mut builder, &spaces, &params);

        let mut spaces: Vec<Token> = Vec::new();

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Word => {
                    let (node, terminator) = Declaration::create(token, tokenizer, &spaces);
                    builder.terminator(&terminator);
                    builder.append(node);
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

    fn set_params(&mut self, params: &Tokens) {
        self.params = Some(params.to_string());
    }

    fn set_params_spaces(&mut self, builder: &mut NodeBuilder, spaces: &Tokens, params: &Tokens) {
        for token in spaces.iter() {
            if !params.is_empty() {
                builder.after_name(token);
            } else {
                builder.between(token);
            }
        }
    }
}
