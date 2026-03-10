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
        // PostCSS end positions: offset points AFTER last char, column points AT last char
        // PostCSS uses 1-indexed columns (minimum is 1)
        let column_index = if tokenizer.index > 0 {
            tokenizer.index - 1
        } else {
            0
        };
        let raw_column = tokenizer.lines.offset(column_index);
        return Self {
            offset: tokenizer.index,  // Points after the last character
            line: tokenizer.lines.value(),
            column: cmp::max(1, raw_column),  // Ensure column is at least 1
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<Node>>,
    pub source: Sources,
}

impl Node {
    pub fn new(data: Data) -> Node {
        // Container nodes (Root, Rule) always have a block; leaf nodes never do.
        // AtRule is determined later during parsing.
        let nodes = match &data {
            Data::Root | Data::Rule(_) => Some(Vec::new()),
            Data::AtRule(_) | Data::Declaration(_) | Data::Comment(_) => None,
        };

        return Node {
            raws: Raws::default(),
            nodes,
            source: Sources::default(),
            data,
        };
    }

    pub fn append(&mut self, child: Node) {
        if let Some(ref mut nodes) = self.nodes {
            nodes.push(child);
        }
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        return serde_json::to_string(&self);
    }

    /// Returns true if this node has a block (`{ }`), even if empty.
    pub fn has_block(&self) -> bool {
        return self.nodes.is_some();
    }

    /// Returns true if this node is a statement without a block.
    pub fn is_statement(&self) -> bool {
        return self.nodes.is_none();
    }

    /// Returns a slice of child nodes, or `None` if this node has no block.
    pub fn children(&self) -> Option<&[Node]> {
        return self.nodes.as_deref();
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
        // Start with Some(vec![]) so children can be collected before the final type is known.
        // Leaf-node builder methods (declaration, comment) will clear this to None.
        node.nodes = Some(Vec::new());
        node.source.start = Source::create_start(token, tokenizer);
        node.raws.before.push_tokens(spaces);

        return Self { node };
    }

    pub fn append(&mut self, child: Node) {
        self.node.append(child);
    }

    pub fn at_rule(&mut self, at_rule: AtRule) {
        // Keep nodes as-is - at-rules with blocks (even empty) keep Some(vec![])
        // Only statement at-rules will have nodes set to None elsewhere
        self.node.data = Data::AtRule(at_rule);
    }

    pub fn rule(&mut self, rule: Rule) {
        // Rules always have a nodes field - keep existing children
        // nodes should already be Some(vec![...]) from new()
        self.node.data = Data::Rule(rule);
    }

    pub fn declaration(&mut self, data: Declaration) {
        self.node.nodes = None;
        self.node.data = Data::Declaration(data);
    }

    pub fn comment(&mut self) {
        self.node.nodes = None;
    }

    pub fn comment_text(&mut self, text: &str) {
        self.node.nodes = None;
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

    pub fn build_with_end_index(mut self, tokenizer: &Tokenizer, after: &[Token], end_index: usize) -> Node {
        self.node.raws.after.push_tokens(after);
        // Create end position at the specified index instead of current tokenizer position
        let column_index = if end_index > 0 { end_index - 1 } else { 0 };
        self.node.source.end = Some(Source {
            offset: end_index,
            line: tokenizer.lines.value(),
            column: cmp::max(1, tokenizer.lines.offset(column_index)),
        });

        return self.node;
    }

    pub fn build_root(mut self, tokenizer: &Tokenizer, after: &[Token]) -> Node {
        let semicolon = self.node.raws.semicolon.unwrap_or(false);
        let has_children = self.node.nodes.as_ref().map_or(false, |n| !n.is_empty());

        if has_children && !semicolon {
            self.node.raws.semicolon.set_false();
        }

        // PostCSS: root.raws.after is empty when there are child nodes,
        // but contains trailing whitespace when root is empty
        if has_children {
            self.node.raws.after.push_str("");
        } else {
            self.node.raws.after.push_tokens(after);
        }

        // Set source.end based on last child node
        if self.node.source.end.is_none() {
            if has_children {
                // Non-empty root: extend one position past the last child's end
                if let Some(nodes) = &self.node.nodes {
                    if let Some(last_node) = nodes.last() {
                        if let Some(ref child_end) = last_node.source.end {
                            self.node.source.end = Some(Source {
                                offset: child_end.offset,
                                line: child_end.line,
                                column: child_end.column + 1,
                            });
                        }
                    }
                }
            } else {
                // Empty root: use current tokenizer position (end of file)
                self.node.source.end = Some(Source::create_end(tokenizer));
            }
        }

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
                Token::Some,
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
