use std::borrow::Cow;

use crate::at_rule::AtRule;
use crate::comment::Comment;
use crate::declaration::Declaration;
use crate::node::{Data, Node};
use crate::rule::Rule;
use crate::sinks::parse::{ParseSink, SinkNode};

/// A [`ParseSink`] that builds the `Node<'src>` tree.
pub struct TreeSink<'src> {
    stack: Vec<Node<'src>>,
    root: Option<Node<'src>>,
}

impl<'src> TreeSink<'src> {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            root: None,
        }
    }
}

impl<'src> Default for TreeSink<'src> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'src> ParseSink<'src> for TreeSink<'src> {
    type NodeRef = ();
    type Output = Node<'src>;

    fn open(&mut self, _parent: (), node: SinkNode<'src>) {
        let tree_node = match node {
            SinkNode::Root { source_start } => {
                let mut n = Node::new(Data::Root);
                n.source.start = source_start;
                n
            }
            SinkNode::Rule {
                source_start,
                selector,
                raws,
            } => {
                let mut n = Node::new(Data::Rule(Rule {
                    selector: selector.clone(),
                }));
                n.source.start = source_start;
                n.raws = raws;
                n
            }
            SinkNode::AtRule {
                source_start,
                name,
                params,
                raws,
            } => {
                let mut n = Node::new(Data::AtRule(AtRule {
                    name: Cow::Borrowed(name),
                    params: params.clone(),
                    has_block: true,
                }));
                n.nodes = Some(Vec::new());
                n.source.start = source_start;
                n.raws = raws;
                n
            }
            _ => return,
        };
        self.stack.push(tree_node);
    }

    fn close(&mut self, _handle: (), node: SinkNode<'src>) {
        let SinkNode::Close { source_end, raws } = node else {
            return;
        };
        let mut tree_node = self.stack.pop().expect("close called with empty stack");
        tree_node.source.end = Some(source_end);
        tree_node.raws.after = raws.after;
        tree_node.raws.semicolon = raws.semicolon;
        tree_node.raws.own_semicolon = raws.own_semicolon;

        if self.stack.is_empty() {
            self.root = Some(tree_node);
        } else {
            self.stack.last_mut().unwrap().append(tree_node);
        }
    }

    fn emit(&mut self, _parent: (), node: SinkNode<'src>) {
        let tree_node = match node {
            SinkNode::Declaration {
                sources,
                prop,
                value,
                important,
                raws,
            } => {
                let decl = Declaration {
                    prop: prop.clone(),
                    important,
                    value: value.clone(),
                };
                let mut n = Node::new(Data::Declaration(decl));
                n.source = sources;
                n.raws = raws;
                n
            }
            SinkNode::Comment {
                sources,
                text,
                raws,
            } => {
                let mut n = Node::new(Data::Comment(Comment {
                    text: Cow::Borrowed(text),
                }));
                n.source = sources;
                n.raws = raws;
                n
            }
            SinkNode::AtRuleStatement {
                sources,
                name,
                params,
                raws,
            } => {
                let mut n = Node::new(Data::AtRule(AtRule {
                    name: Cow::Borrowed(name),
                    params: params.clone(),
                    has_block: false,
                }));
                n.nodes = None;
                n.source = sources;
                n.raws = raws;
                n
            }
            _ => return,
        };

        if let Some(parent) = self.stack.last_mut() {
            parent.append(tree_node);
        }
    }

    fn finish(self) -> Node<'src> {
        self.root.expect("finish called before root was closed")
    }
}
