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

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Symbol if token.value == ";" => break,
                Kind::Symbol => node.raws.between.push(token.value),
                Kind::Space => node.raws.between.push(token.value),
                Kind::Word => node.data = Data::Declaration(Self::new(prop.value, token.value)),
                _ => break,
            }
        }

        return node;
    }
}

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

        return node;
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Raw(Option<String>);

impl Raw {
    pub fn push(&mut self, raw: &str) {
        match self.0 {
            Some(ref mut prev) => prev.push_str(raw),
            None => self.0 = Some(String::from(raw)),
        };
    }

    pub fn skip(&self) -> bool {
        return self.0.is_none();
    }
}

impl Default for Raw {
    fn default() -> Self {
        return Self(None);
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Raws {
    #[serde(skip_serializing_if = "Raw::skip")]
    pub before: Raw,
    #[serde(skip_serializing_if = "Raw::skip")]
    pub between: Raw,
    pub semicolon: bool,
    #[serde(skip_serializing_if = "Raw::skip")]
    pub after: Raw,
}

impl Default for Raws {
    fn default() -> Self {
        return Self {
            before: Raw::default(),
            between: Raw::default(),
            semicolon: false,
            after: Raw::default(),
        };
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Source {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

impl Default for Source {
    fn default() -> Self {
        return Self {
            offset: 0,
            line: 1,
            column: 1,
        };
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Sources {
    pub start: Source,
    pub end: Option<Source>,
}

impl Default for Sources {
    fn default() -> Self {
        return Self {
            start: Source::default(),
            end: None,
        };
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Data {
    Root,
    Rule(Rule),
    Document,
    #[serde(rename = "decl")]
    Declaration(Declaration),
    AtRule,
    Comment,
}

impl Default for Data {
    fn default() -> Self {
        return Self::Root;
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub nodes: Vec<Node>,
    pub raws: Raws,
    pub sources: Sources,
    #[serde(flatten)]
    pub data: Data,
}

impl Node {
    pub fn new(data: Data) -> Node {
        return Node {
            raws: Raws::default(),
            nodes: Vec::new(),
            sources: Sources::default(),
            data,
        };
    }

    pub fn append(&mut self, child: Node) {
        self.nodes.push(child);
    }
}

impl Default for Node {
    fn default() -> Self {
        return Self::new(Data::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::Token;
    use serde_test::*;

    #[test]
    fn it_serializes_node() {
        let node = Node::new(Data::Root);

        assert_ser_tokens(
            &node,
            &[
                Token::Map { len: None },
                Token::Str("nodes"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("raws"),
                Token::Struct {
                    name: "Raws",
                    len: 1,
                },
                Token::Str("semicolon"),
                Token::Bool(false),
                Token::StructEnd,
                Token::Str("sources"),
                Token::Struct {
                    name: "Sources",
                    len: 2,
                },
                Token::Str("start"),
                Token::Struct {
                    name: "Source",
                    len: 3,
                },
                Token::Str("offset"),
                Token::U64(0),
                Token::Str("line"),
                Token::U64(1),
                Token::Str("column"),
                Token::U64(1),
                Token::StructEnd,
                Token::Str("end"),
                Token::None,
                Token::StructEnd,
                Token::Str("type"),
                Token::Str("root"),
                Token::MapEnd,
            ],
        );
    }
}
