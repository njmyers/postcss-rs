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

    pub fn create(token: Token, tokenizer: &mut Tokenizer) -> Node {
        let mut node = Node::new(Data::Root);
        let mut data = Self::new(&token.value[1..]);
        node.set_start(&token, tokenizer);

        let mut params: Vec<Token> = Vec::new();

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Space => node.raws.between.push(token.value),
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
                Kind::Space => spaces.push(token),
                Kind::Word => node.append(Declaration::create(token, tokenizer)),
                Kind::Symbol => break,
                _ => break,
            };
        }

        node.set_end(tokenizer);
        node.data = Data::AtRule(data);

        return node;
    }

    fn combine(params: Vec<Token>) -> String {
        return params
            .iter()
            .flat_map(|token| token.value.chars())
            .collect();
    }
}
