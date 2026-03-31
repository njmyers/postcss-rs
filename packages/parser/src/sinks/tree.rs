use std::borrow::Cow;

use crate::at_rule::AtRule;
use crate::declaration::Declaration;
use crate::node::{CommentData, Data, Node};
use crate::rule::Rule;
use crate::sinks::parse::{ContainerClose, ContainerOpen, LeafNode, ParseSink};

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

    fn open(&mut self, _parent: (), open: ContainerOpen<'src>) {
        let node = match open {
            ContainerOpen::Root { source_start } => {
                let mut n = Node::new(Data::Root);
                n.source.start = source_start;
                n
            }
            ContainerOpen::Rule {
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
            ContainerOpen::AtRule {
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
        };
        self.stack.push(node);
    }

    fn close(&mut self, _handle: (), close: ContainerClose) {
        let mut node = self.stack.pop().expect("close called with empty stack");
        node.source.end = Some(close.source_end);
        node.raws.after = close.raws.after;
        node.raws.semicolon = close.raws.semicolon;
        node.raws.own_semicolon = close.raws.own_semicolon;

        if self.stack.is_empty() {
            self.root = Some(node);
        } else {
            self.stack.last_mut().unwrap().append(node);
        }
    }

    fn emit(&mut self, _parent: (), leaf: LeafNode<'src>) {
        let node = match leaf {
            LeafNode::Declaration {
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
            LeafNode::Comment {
                sources,
                text,
                raws,
            } => {
                let mut n = Node::new(Data::Comment(CommentData {
                    text: Cow::Borrowed(text),
                }));
                n.source = sources;
                n.raws = raws;
                n
            }
            LeafNode::AtRuleStatement {
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
        };

        if let Some(parent) = self.stack.last_mut() {
            parent.append(node);
        }
    }

    fn finish(self) -> Node<'src> {
        self.root.expect("finish called before root was closed")
    }
}
