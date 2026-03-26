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

/// A single source-map position (line, column, and byte offset).
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Source {
    /// Byte offset from the start of the CSS source.
    pub offset: usize,
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number on `line`.
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
    pub fn create_start(token: &Token<'_>, tokenizer: &Tokenizer<'_>) -> Self {
        return Self {
            offset: token.start,
            line: tokenizer.lines.value(),
            column: cmp::max(1, token.column),
        };
    }

    pub fn create_end(tokenizer: &Tokenizer<'_>) -> Self {
        // PostCSS end positions: offset points AFTER last char, column points AT last char
        // PostCSS uses 1-indexed columns (minimum is 1)
        let column_index = if tokenizer.index > 0 {
            tokenizer.index - 1
        } else {
            0
        };
        let raw_column = tokenizer.lines.offset(column_index);
        return Self {
            offset: tokenizer.index, // Points after the last character
            line: tokenizer.lines.value(),
            column: cmp::max(1, raw_column), // Ensure column is at least 1
        };
    }
}

/// The start and (optional) end source positions of a node.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Sources {
    /// Position of the first character of the node.
    pub start: Source,
    /// Position of the last character of the node; `None` before the node is
    /// fully parsed.
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

/// The type-specific payload of a [`Node`].
///
/// Serialises to/from JSON with a `"type"` discriminator field, matching the
/// PostCSS JSON output format.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Data {
    /// The root node wrapping the entire stylesheet.
    #[serde(rename = "root")]
    Root,
    /// A rule with a selector, e.g. `.foo { … }`.
    #[serde(rename = "rule")]
    Rule(Rule),
    /// A property declaration, e.g. `color: red`.
    #[serde(rename = "decl")]
    Declaration(Declaration),
    /// An at-rule, e.g. `@media screen { … }`.
    #[serde(rename = "atrule")]
    AtRule(AtRule),
    /// A block comment, e.g. `/* text */`.
    #[serde(rename = "comment")]
    Comment(CommentData),
}

/// Text payload for a CSS comment node.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct CommentData {
    /// The comment text, without the surrounding `/*` and `*/` delimiters or
    /// any leading/trailing whitespace (which is stored in `raws.left` /
    /// `raws.right`).
    pub text: String,
}

impl Default for Data {
    fn default() -> Self {
        return Self::Root;
    }
}

/// A single node in the PostCSS-compatible CSS abstract syntax tree.
///
/// Each node carries:
/// - `raws` — original whitespace and formatting information.
/// - `data` — the type-specific payload (rule, declaration, comment, …).
/// - `nodes` — child nodes for container types (root, rule, at-rule with a block).
/// - `source` — source position information.
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
        // Container nodes (Root, Rule) start with Some(vec![])
        // Leaf nodes (Declaration, Comment) have None
        // AtRule: will be determined later based on whether it has children
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

    /// Returns true if this node has child nodes (i.e., a non-empty block).
    pub fn has_nodes(&self) -> bool {
        return self.nodes.as_ref().is_some_and(|n| !n.is_empty());
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
    pub fn new(token: &Token<'_>, tokenizer: &mut Tokenizer<'_>, spaces: &[Token<'_>]) -> Self {
        let mut node = Node::default();
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
        // Declarations never have a nodes field
        self.node.nodes = None;
        self.node.data = Data::Declaration(data);
    }

    pub fn comment(&mut self) {
        // Comment data is already set by comment_text
        // Comments never have a nodes field
        self.node.nodes = None;
    }

    pub fn comment_text(&mut self, text: &str) {
        // Comments never have a nodes field
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
        self.node.raws.selector = crate::ValueRaw::new(String::from(value), String::from(raw));
    }

    pub fn important_raw(&mut self, value: &str) {
        self.node.raws.important = crate::Raw::new(value);
    }

    pub fn ensure_empty_before(&mut self) {
        self.node.raws.before.push_str("");
    }

    pub fn before(&mut self, token: &Token<'_>) {
        self.node.raws.before.push(token);
    }

    pub fn append_to_before(&mut self, text: &str) {
        self.node.raws.before.push_str(text);
    }

    pub fn ensure_empty_between(&mut self) {
        self.node.raws.between.push_str("");
    }

    pub fn between(&mut self, token: &Token<'_>) {
        self.node.raws.between.push(token);
    }

    pub fn ensure_empty_after(&mut self) {
        self.node.raws.after.push_str("");
    }

    pub fn after(&mut self, token: &Token<'_>) {
        self.node.raws.after.push(token);
    }

    pub fn ensure_empty_after_name(&mut self) {
        self.node.raws.after_name.push_str("");
    }

    pub fn after_name(&mut self, token: &Token<'_>) {
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

    pub fn build(mut self, tokenizer: &mut Tokenizer<'_>, after: &[Token<'_>]) -> Node {
        self.node.raws.after.push_tokens(after);
        self.node.source.end = Some(Source::create_end(tokenizer));

        return self.node;
    }

    pub fn build_with_end_index(
        mut self,
        tokenizer: &Tokenizer<'_>,
        after: &[Token<'_>],
        end_index: usize,
    ) -> Node {
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

    pub fn build_root(mut self, tokenizer: &Tokenizer<'_>, after: &[Token<'_>]) -> Node {
        let semicolon = self.node.raws.semicolon.unwrap_or(false);

        if self.node.has_nodes() && !semicolon {
            self.node.raws.semicolon.set_false();
        }

        // PostCSS: root.raws.after is empty when there are child nodes,
        // but contains trailing whitespace when root is empty
        if self.node.has_nodes() {
            self.node.raws.after.push_str("");
        } else {
            self.node.raws.after.push_tokens(after);
        }

        // Set source.end based on last child node
        if self.node.source.end.is_none() {
            self.node.source.end = if self.node.has_nodes() {
                // Non-empty root: extend one position past the last child's end
                self.node.nodes
                    .as_ref()
                    .and_then(|nodes| nodes.last())
                    .and_then(|last| last.source.end.as_ref())
                    .map(|end| Source {
                        offset: end.offset,
                        line: end.line,
                        column: end.column + 1,
                    })
            } else {
                // Empty root: use current tokenizer position (end of file)
                Some(Source::create_end(tokenizer))
            };
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
                // nodes field (Some(vec![]))
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
