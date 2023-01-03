use crate::Declaration;
use crate::Node;
use crate::NodeBuilder;

use serde::Deserialize;
use serde::Serialize;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;
use postcss_rs_tokenizer::Tokens;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub selector: String,
}

impl Rule {
    pub fn create(token: Token, tokenizer: &mut Tokenizer, spaces: &[Token]) -> Node {
        let mut builder = NodeBuilder::new(&token, tokenizer, spaces);
        let rule = Self::selector(token, tokenizer, &mut builder);

        builder.ensure_empty_before();
        builder.rule(Self::new(&rule));

        let mut spaces: Vec<Token> = Vec::new();

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Word => {
                    let (child, semicolon) = Declaration::create(token, tokenizer, &spaces);

                    if semicolon {
                        builder.semicolon();
                    }

                    builder.append(child);
                    spaces.clear();
                }

                Kind::Space => spaces.push(token),
                Kind::CloseCurly => break,
                _ => break,
            };
        }

        return builder.build(tokenizer, &spaces);
    }

    fn new(selector: &str) -> Self {
        return Self {
            selector: String::from(selector),
        };
    }

    fn selector(token: Token, tokenizer: &mut Tokenizer, builder: &mut NodeBuilder) -> String {
        if token.kind == Kind::OpenCurly {
            builder.ensure_empty_after();
            builder.ensure_empty_between();

            return String::from("");
        }

        let mut rules = Tokens::new();
        rules.push(token);

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Brackets => rules.push(token),
                Kind::Space => builder.between(&token),
                Kind::OpenCurly => break,
                _ => break,
            };
        }

        return rules.to_string();
    }
}
