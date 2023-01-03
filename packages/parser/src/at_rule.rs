use std::ops::Deref;
use std::ops::DerefMut;

use serde::Deserialize;
use serde::Serialize;

use crate::Declaration;
use crate::Node;
use crate::NodeBuilder;
use crate::Statement;
use crate::StatementTerminator;

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
    pub fn create(
        token: Token,
        tokenizer: &mut Tokenizer,
        spaces: &[Token],
    ) -> (Node, StatementTerminator) {
        let mut builder = NodeBuilder::new(&token, tokenizer, spaces);
        builder.ensure_empty_before();
        builder.ensure_empty_between();
        builder.ensure_empty_after_name();

        let mut data = Self::new(&token.value[1..]);
        let mut spaces = Tokens::new();
        let mut params = ParamsBuilder::new();
        let mut terminator = StatementTerminator::new();

        while let Some(token) = tokenizer.next() {
            terminator.process(&token);

            match token.kind {
                Kind::Space if params.is_empty() => spaces.push(token),
                Kind::Space => builder.between(&token),
                Kind::Word => params.push(token),
                Kind::Brackets => params.push(token),
                Kind::OpenCurly => break,
                Kind::Colon => params.push(token),
                Kind::String => params.push(token),
                _ => break,
            };
        }

        params.set_spaces(&mut builder, &spaces);
        data.set_params(&params);

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
        let node = builder.build(tokenizer, &spaces);

        return (node, terminator);
    }

    fn new(name: &str) -> Self {
        return Self {
            name: String::from(name),
            params: None,
        };
    }

    fn set_params(&mut self, params: &ParamsBuilder) {
        self.params = Some(params.to_string());
    }
}

struct ParamsBuilder<'a>(Statement<'a>);

impl<'a> ParamsBuilder<'a> {
    fn new() -> Self {
        return Self(Statement::new());
    }

    fn set_spaces(&self, builder: &mut NodeBuilder, spaces: &Tokens) {
        for token in spaces.iter() {
            if !self.is_empty() {
                builder.after_name(token);
            } else {
                builder.between(token);
            }
        }
    }
}

impl<'a> Deref for ParamsBuilder<'a> {
    type Target = Statement<'a>;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

impl<'a> DerefMut for ParamsBuilder<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.0;
    }
}
