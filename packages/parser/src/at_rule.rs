use serde::Deserialize;
use serde::Serialize;

use super::Data;
use super::Declaration;
use super::Node;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct AtRule {
    pub name: String,
    pub params: Option<String>,
}

impl AtRule {
    pub fn new(name: &str) -> Self {
        return Self {
            name: String::from(name),
            params: None,
        };
    }

    pub fn create(token: Token, tokenizer: &mut Tokenizer, spaces: &[Token]) -> Node {
        // let mut semicolon = false;
        let mut node = Node::new(Data::Root);
        let mut data = Self::new(&token.value[1..]);
        node.set_start(&token, tokenizer);
        node.raws.before.push_str("");
        node.raws.before.push_tokens(spaces);

        let mut params: Vec<Token> = Vec::new();

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Space => node.raws.between.push(&token),
                Kind::Word => params.push(token),
                Kind::Brackets => params.push(token),
                Kind::Symbol if token.value == "{" => break,
                Kind::Symbol => params.push(token),
                _ => break,
            };
        }

        data.params = Some(Self::combine(params));

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

        node.raws.after.push_tokens(&spaces);
        node.set_end(tokenizer);
        node.data = Data::AtRule(data);

        return node;
    }

    fn combine(params: Vec<Token>) -> String {
        let mut value = String::new();

        for token in params {
            value.push_str(token.value)
        }

        return value;
    }
}
