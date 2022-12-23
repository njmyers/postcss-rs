use serde::Deserialize;
use serde::Serialize;
use std::cmp;

use super::AtRule;
use super::Declaration;
use super::Rule;
use crate::Raws;

use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
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
            column: cmp::max(1, token.column),
        };
    }

    pub fn create_end(tokenizer: &Tokenizer) -> Self {
        return Self {
            offset: tokenizer.index - 1,
            line: tokenizer.lines.value(),
            column: tokenizer.lines.offset(tokenizer.index - 1),
        };
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
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

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Data {
    #[serde(rename = "root")]
    Root,
    #[serde(rename = "rule")]
    Rule(Rule),
    #[serde(rename = "decl")]
    Declaration(Declaration),
    #[serde(rename = "atrule")]
    AtRule(AtRule),
    #[serde(rename = "comment")]
    Comment,
}

impl Default for Data {
    fn default() -> Self {
        return Self::Root;
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub raws: Raws,
    #[serde(flatten)]
    pub data: Data,
    #[serde(default)]
    pub nodes: Vec<Node>,
    pub source: Sources,
}

impl Node {
    pub fn new(data: Data) -> Node {
        return Node {
            raws: Raws::default(),
            nodes: Vec::new(),
            source: Sources::default(),
            data,
        };
    }

    pub fn append(&mut self, child: Node) {
        self.nodes.push(child);
    }

    pub fn set_start(&mut self, token: &Token, tokenizer: &mut Tokenizer) {
        self.source.start = Source::create_start(token, tokenizer);
    }

    pub fn set_end(&mut self, tokenizer: &mut Tokenizer) {
        self.source.end = Some(Source::create_end(tokenizer));
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
                Token::Str("raws"),
                Token::Struct {
                    name: "Raws",
                    len: 1,
                },
                Token::Str("after"),
                Token::NewtypeStruct { name: "Raw" },
                Token::Some,
                Token::Str(""),
                Token::StructEnd,
                Token::Str("type"),
                Token::Str("root"),
                Token::Str("nodes"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("source"),
                Token::Struct {
                    name: "Sources",
                    len: 1,
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
                Token::StructEnd,
                Token::MapEnd,
            ],
        );
    }
}
