use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
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

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Raws<'a> {
    pub before: Option<&'a str>,
    pub between: Option<&'a str>,
    pub semicolon: bool,
    pub after: Option<&'a str>,
}

impl<'a> Raws<'a> {
    pub fn before(&mut self, value: &'a str) {
        self.before = Some(value);
    }

    pub fn after(&mut self, value: &'a str) {
        self.after = Some(value);
    }

    pub fn between(&mut self, value: &'a str) {
        self.between = Some(value);
    }

    pub fn semicolon(&mut self) {
        self.semicolon = false;
    }
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Source {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Sources {
    pub start: Option<Source>,
    pub end: Option<Source>,
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Node<'a> {
    #[serde(rename = "type")]
    pub r#type: Type,
    pub nodes: Vec<Node<'a>>,
    #[serde(borrow)]
    pub raws: Raws<'a>,
    pub sources: Sources,
}

impl<'a> Node<'a> {
    pub fn new(r#type: Type, raws: Raws<'a>, sources: Sources) -> Node<'a> {
        return Node {
            r#type,
            raws,
            nodes: Vec::new(),
            sources,
        };
    }

    pub fn append(&mut self, child: Node<'a>) {
        self.nodes.push(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::Token as SToken;
    use serde_test::*;

    #[test]
    fn it_serializes_type() {
        assert_ser_tokens(
            &Type::Root,
            &[SToken::UnitVariant {
                name: "Type",
                variant: "root",
            }],
        )
    }

    #[test]
    fn it_serializes_node() {
        let node = Node::new(
            Type::Root,
            Raws {
                before: None,
                after: None,
                between: None,
                semicolon: false,
            },
            Sources {
                start: None,
                end: None,
            },
        );

        assert_ser_tokens(
            &node,
            &[
                SToken::Struct {
                    name: "Node",
                    len: 4,
                },
                SToken::Str("type"),
                SToken::UnitVariant {
                    name: "Type",
                    variant: "root",
                },
                SToken::Str("nodes"),
                SToken::Seq { len: Some(0) },
                SToken::SeqEnd,
                SToken::Str("raws"),
                SToken::Struct {
                    name: "Raws",
                    len: 4,
                },
                SToken::Str("before"),
                SToken::None,
                SToken::Str("between"),
                SToken::None,
                SToken::Str("semicolon"),
                SToken::Bool(false),
                SToken::Str("after"),
                SToken::None,
                SToken::StructEnd,
                SToken::Str("sources"),
                SToken::Struct {
                    name: "Sources",
                    len: 2,
                },
                SToken::Str("start"),
                SToken::None,
                SToken::Str("end"),
                SToken::None,
                SToken::StructEnd,
                SToken::StructEnd,
            ],
        );
    }
}
