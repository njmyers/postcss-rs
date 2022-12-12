use serde::Deserialize;
use serde::Serialize;
use serde_json;

use super::Declaration;
use super::Rule;

use crate::tokenizer::Token;
use crate::tokenizer::Tokenizer;

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

impl Source {
    pub fn create_start(token: &Token, tokenizer: &Tokenizer) -> Self {
        return Self {
            offset: token.start,
            line: tokenizer.lines.value(),
            column: 0,
        };
    }

    pub fn create_end(tokenizer: &Tokenizer) -> Self {
        return Self {
            offset: tokenizer.end(),
            line: tokenizer.lines.value(),
            column: 0,
        };
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Sources {
    pub start: Source,
    #[serde(skip_serializing_if = "Sources::skip")]
    pub end: Option<Source>,
}

impl Sources {
    pub fn skip(source: &Option<Source>) -> bool {
        return source.is_none();
    }
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
    pub raws: Raws,
    #[serde(flatten)]
    pub data: Data,
    pub nodes: Vec<Node>,
    pub sources: Sources,
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

    pub fn to_json(&self) -> serde_json::Result<String> {
        return serde_json::to_string(&self);
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
                Token::Str("type"),
                Token::Str("root"),
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
                Token::MapEnd,
            ],
        );
    }
}
