use serde::{Deserialize, Serialize};

pub struct Declaration {
    pub prop: String,
    pub value: String,
}

pub struct Rule {
    pub selector: String,
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum Type {
    #[serde(rename = "document")]
    Document,
    #[serde(rename = "root")]
    Root,
    #[serde(rename = "rule")]
    Rule,
    #[serde(rename = "decl")]
    Declaration,
    #[serde(rename = "atrule")]
    AtRule,
    #[serde(rename = "comment")]
    Comment,
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Raws {
    pub before: Option<String>,
    pub between: Option<String>,
    pub semicolon: bool,
    pub after: Option<String>,
}

impl Default for Raws {
    fn default() -> Self {
        return Self {
            before: None,
            between: None,
            semicolon: false,
            after: None,
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
pub struct Node {
    #[serde(rename = "type")]
    pub kind: Type,
    pub nodes: Vec<Node>,
    pub raws: Raws,
    pub sources: Sources,
}

impl Node {
    pub fn new(kind: Type) -> Node {
        return Node {
            kind,
            raws: Raws::default(),
            nodes: Vec::new(),
            sources: Sources::default(),
        };
    }

    pub fn append(&mut self, child: Node) {
        self.nodes.push(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::Token;
    use serde_test::*;

    #[test]
    fn it_serializes_node() {
        let node = Node::new(Type::Root);

        assert_ser_tokens(
            &node,
            &[
                Token::Struct {
                    name: "Node",
                    len: 4,
                },
                Token::Str("type"),
                Token::UnitVariant {
                    name: "Type",
                    variant: "root",
                },
                Token::Str("nodes"),
                Token::Seq { len: Some(0) },
                Token::SeqEnd,
                Token::Str("raws"),
                Token::Struct {
                    name: "Raws",
                    len: 4,
                },
                Token::Str("before"),
                Token::None,
                Token::Str("between"),
                Token::None,
                Token::Str("semicolon"),
                Token::Bool(false),
                Token::Str("after"),
                Token::None,
                Token::StructEnd,
                Token::Str("sources"),
                Token::Struct {
                    name: "Sources",
                    len: 2,
                },
                Token::Str("start"),
                Token::Struct {
                    name: "Source",
                    len: 3
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
                Token::StructEnd,
            ],
        );
    }
}
