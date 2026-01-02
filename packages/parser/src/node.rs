use crate::AtRule;
use crate::Declaration;
use crate::Raws;
use crate::Rule;
use crate::StatementTerminator;

use serde::Deserialize;
use serde::Serialize;

use std::cmp;

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
    Comment(CommentData),
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct CommentData {
    pub text: String,
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

    pub fn to_json(&self) -> serde_json::Result<String> {
        return serde_json::to_string(&self);
    }
}

impl Default for Node {
    fn default() -> Self {
        return Self::new(Data::default());
    }
}

pub struct NodeBuilder {
    pub node: Node,
}

impl NodeBuilder {
    pub fn new(token: &Token, tokenizer: &mut Tokenizer, spaces: &[Token]) -> Self {
        let mut node = Node::default();
        node.source.start = Source::create_start(token, tokenizer);
        node.raws.before.push_tokens(spaces);

        return Self { node };
    }

    pub fn append(&mut self, child: Node) {
        self.node.append(child);
    }

    pub fn at_rule(&mut self, at_rule: AtRule) {
        self.node.data = Data::AtRule(at_rule);
    }

    pub fn rule(&mut self, rule: Rule) {
        self.node.data = Data::Rule(rule);
    }

    pub fn declaration(&mut self, data: Declaration) {
        self.node.data = Data::Declaration(data);
    }

    pub fn comment(&mut self) {
        // Comment data is already set by comment_text
    }

    pub fn comment_text(&mut self, text: &str) {
        self.node.data = Data::Comment(CommentData {
            text: String::from(text),
        });
    }

    pub fn comment_left(&mut self, left: &str) {
        self.node.raws.left.push_str(left);
    }

    pub fn comment_right(&mut self, right: &str) {
        self.node.raws.right.push_str(right);
    }

    pub fn value_raw(&mut self, value: &str, raw: &str) {
        self.node.raws.value = crate::ValueRaw::new(String::from(value), String::from(raw));
    }

    pub fn selector_raw(&mut self, value: &str, raw: &str) {
        self.node.raws.selector = crate::SelectorRaw::new(String::from(value), String::from(raw));
    }

    pub fn important_raw(&mut self, value: &str) {
        self.node.raws.important = crate::Raw::new(value);
    }

    pub fn ensure_empty_before(&mut self) {
        self.node.raws.before.push_str("");
    }

    pub fn before(&mut self, token: &Token) {
        self.node.raws.before.push(token);
    }

    pub fn append_to_before(&mut self, text: &str) {
        self.node.raws.before.push_str(text);
    }

    pub fn ensure_empty_between(&mut self) {
        self.node.raws.between.push_str("");
    }

    pub fn between(&mut self, token: &Token) {
        self.node.raws.between.push(token);
    }

    pub fn ensure_empty_after(&mut self) {
        self.node.raws.after.push_str("");
    }

    pub fn after(&mut self, token: &Token) {
        self.node.raws.after.push(token);
    }

    pub fn ensure_empty_after_name(&mut self) {
        self.node.raws.after_name.push_str("");
    }

    pub fn after_name(&mut self, token: &Token) {
        self.node.raws.after_name.push(token);
    }

    pub fn terminator(&mut self, terminator: &StatementTerminator) {
        if terminator.semicolon {
            self.node.raws.semicolon.set_true();
        } else {
            self.node.raws.semicolon.set_false();
        }
    }

    pub fn semicolon(&mut self) {
        self.node.raws.semicolon.set_true();
    }

    pub fn build(mut self, tokenizer: &mut Tokenizer, after: &[Token]) -> Node {
        self.node.raws.after.push_tokens(after);
        self.node.source.end = Some(Source::create_end(tokenizer));

        return self.node;
    }

    pub fn build_root(mut self, after: &[Token]) -> Node {
        let semicolon = self.node.raws.semicolon.unwrap_or(false);

        if !self.node.nodes.is_empty() && !semicolon {
            self.node.raws.semicolon.set_false();
        }

        self.node.raws.after.push_tokens(after);

        return self.node;
    }
}

impl Default for NodeBuilder {
    fn default() -> Self {
        return Self {
            node: Node::new(Data::Root),
        };
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
                    len: 0,
                },
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
