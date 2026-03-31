use std::borrow::Cow;

use crate::at_rule::AtRule;
use crate::declaration::Declaration;
use crate::node::{CommentData, Data, Node, Sources};
use crate::raws::{Raw, Semicolon, ValueRaw};
use crate::rule::Rule;
use crate::sinks::parse::{ContainerClose, ContainerOpen, LeafNode, ParseSink};

#[inline]
fn apply_raw(raw: &mut Raw, value: Option<String>) {
    if let Some(s) = value {
        raw.push_str(&s);
    }
}

#[inline]
fn apply_semi(semi: &mut Semicolon, value: Option<bool>) {
    match value {
        Some(true) => semi.set_true(),
        Some(false) => semi.set_false(),
        None => {}
    }
}

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
                raws_before,
                selector,
                raws_selector_raw,
                raws_between,
            } => {
                let mut n = Node::new(Data::Rule(Rule {
                    selector: selector.clone(),
                }));
                n.source.start = source_start;
                apply_raw(&mut n.raws.before, raws_before);
                apply_raw(&mut n.raws.between, raws_between);
                n.raws.after.push_str("");
                if let Some(raw) = raws_selector_raw {
                    n.raws.selector = ValueRaw::new(selector.to_string(), raw);
                }
                n
            }
            ContainerOpen::AtRule {
                source_start,
                raws_before,
                name,
                raws_after_name,
                params,
                raws_between,
            } => {
                let mut n = Node::new(Data::AtRule(AtRule {
                    name: Cow::Borrowed(name),
                    params: params.clone(),
                    has_block: true,
                }));
                n.nodes = Some(Vec::new());
                n.source.start = source_start;
                apply_raw(&mut n.raws.before, raws_before);
                apply_raw(&mut n.raws.after_name, raws_after_name);
                apply_raw(&mut n.raws.between, raws_between);
                n.raws.after.push_str("");
                n
            }
        };
        self.stack.push(node);
    }

    fn close(&mut self, _handle: (), close: ContainerClose) {
        let ContainerClose {
            source_end,
            raws_after,
            semicolon,
            own_semicolon,
        } = close;
        let mut node = self.stack.pop().expect("close called with empty stack");
        node.source.end = Some(source_end);
        node.raws.after = Raw::default();
        apply_raw(&mut node.raws.after, raws_after);
        apply_semi(&mut node.raws.semicolon, semicolon);
        apply_raw(&mut node.raws.own_semicolon, own_semicolon);

        if self.stack.is_empty() {
            self.root = Some(node);
        } else {
            self.stack.last_mut().unwrap().append(node);
        }
    }

    fn emit(&mut self, _parent: (), leaf: LeafNode<'src>) {
        let node = match leaf {
            LeafNode::Declaration {
                source_start,
                source_end,
                raws_before,
                prop,
                raws_between,
                value,
                raws_value_raw,
                important,
                raws_important,
            } => {
                let decl = Declaration {
                    prop: prop.clone(),
                    important,
                    value: value.clone(),
                };
                let mut n = Node::new(Data::Declaration(decl));
                n.source = Sources {
                    start: source_start,
                    end: Some(source_end),
                };
                apply_raw(&mut n.raws.before, raws_before);
                apply_raw(&mut n.raws.between, raws_between);
                if let Some(raw) = raws_value_raw {
                    n.raws.value = ValueRaw::new(value.to_string(), raw);
                }
                apply_raw(&mut n.raws.important, raws_important);
                n
            }
            LeafNode::Comment {
                source_start,
                source_end,
                raws_before,
                raws_left,
                text,
                raws_right,
            } => {
                let mut n = Node::new(Data::Comment(CommentData {
                    text: Cow::Borrowed(text),
                }));
                n.source = Sources {
                    start: source_start,
                    end: Some(source_end),
                };
                apply_raw(&mut n.raws.before, raws_before);
                apply_raw(&mut n.raws.left, raws_left);
                apply_raw(&mut n.raws.right, raws_right);
                n
            }
            LeafNode::AtRuleStatement {
                source_start,
                source_end,
                raws_before,
                name,
                raws_after_name,
                params,
                raws_between,
            } => {
                let mut n = Node::new(Data::AtRule(AtRule {
                    name: Cow::Borrowed(name),
                    params: params.clone(),
                    has_block: false,
                }));
                n.nodes = None;
                n.source = Sources {
                    start: source_start,
                    end: source_end,
                };
                apply_raw(&mut n.raws.before, raws_before);
                apply_raw(&mut n.raws.after_name, raws_after_name);
                apply_raw(&mut n.raws.between, raws_between);
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
