use std::borrow::Cow;

use crate::at_rule::AtRule;
use crate::declaration::Declaration;
use crate::node::{CommentData, Data, Node, Source, Sources};
use crate::raws::{Raw, Semicolon, ValueRaw};
use crate::rule::Rule;

// ── Helper ─────────────────────────────────────────────────────────────────

/// Apply `value` to `raw`: if `Some(s)`, push `s`; if `None`, leave as `Raw(None)`.
#[inline]
fn apply_raw(raw: &mut Raw, value: Option<String>) {
    if let Some(s) = value {
        raw.push_str(&s);
    }
}

/// Apply an optional bool to a `Semicolon`.
#[inline]
fn apply_semi(semi: &mut Semicolon, value: Option<bool>) {
    match value {
        Some(true) => semi.set_true(),
        Some(false) => semi.set_false(),
        None => {}
    }
}

// ── Public data types passed through the trait ────────────────────────────

/// Data carried when a container node is opened (its children have not yet been parsed).
pub enum ContainerOpen<'src> {
    /// The root node that wraps the entire stylesheet.
    Root { source_start: Source },

    /// A rule node, e.g. `.foo { … }`.
    Rule {
        source_start: Source,
        /// Whitespace/comments that appear before the selector; `None` = absent.
        raws_before: Option<String>,
        /// Comment-stripped selector string.
        selector: Cow<'src, str>,
        /// Raw selector as it appears in CSS (with inline comments); `None` when
        /// identical to `selector`.
        raws_selector_raw: Option<String>,
        /// Whitespace between the selector and `{`.
        raws_between: Option<String>,
    },

    /// An at-rule with a block body, e.g. `@media screen { … }`.
    AtRule {
        source_start: Source,
        raws_before: Option<String>,
        /// Name without the leading `@`.
        name: &'src str,
        /// Whitespace between the name and the params.
        raws_after_name: Option<String>,
        /// At-rule params; `None` when there are none.
        params: Option<Cow<'src, str>>,
        /// Whitespace between the params and `{`.
        raws_between: Option<String>,
    },
}

/// Data carried when a container node is closed (all children have been parsed).
pub struct ContainerClose {
    pub source_end: Source,
    /// Trailing whitespace inside the block (before `}`); `None` = absent.
    pub raws_after: Option<String>,
    /// Whether the last child was followed by a semicolon.
    pub semicolon: Option<bool>,
    /// Trailing semicolon after `}` (Rule-only); `None` = absent.
    pub own_semicolon: Option<String>,
}

/// A leaf (non-container) node.
pub enum LeafNode<'src> {
    Declaration {
        source_start: Source,
        source_end: Source,
        raws_before: Option<String>,
        prop: Cow<'src, str>,
        raws_between: Option<String>,
        /// Comment-stripped value.
        value: Cow<'src, str>,
        /// Raw value as it appears in CSS; `None` when identical to `value`.
        raws_value_raw: Option<String>,
        important: Option<bool>,
        raws_important: Option<String>,
    },
    Comment {
        source_start: Source,
        source_end: Source,
        raws_before: Option<String>,
        raws_left: Option<String>,
        text: &'src str,
        raws_right: Option<String>,
    },
    /// A statement at-rule (no block), e.g. `@import "…";`.
    AtRuleStatement {
        source_start: Source,
        /// `None` when the at-rule was terminated by a parent `CloseCurly`
        /// (PostCSS leaves `source.end` absent in that case).
        source_end: Option<Source>,
        raws_before: Option<String>,
        name: &'src str,
        raws_after_name: Option<String>,
        params: Option<Cow<'src, str>>,
        raws_between: Option<String>,
    },
}

// ── Trait ─────────────────────────────────────────────────────────────────

/// Output abstraction for the CSS parser.
///
/// Implement this trait to receive the parsed CSS in a form other than the
/// default [`Node`] tree.  Two implementations are provided:
///
/// - [`TreeSink`] — builds the `Node<'src>` tree (the existing behaviour).
/// - `OffsetSink` (in `postcss-rs-node-postcss`) — emits flat `NodeSlot`s.
pub trait ParseSink<'src> {
    /// Opaque handle used to identify a container node.
    ///
    /// Must implement `Copy` and `Default` (the default is used as a dummy
    /// parent for the root `open_container` call).
    type NodeRef: Copy + Default;

    /// The value returned by [`finish`].
    type Output;

    /// Called when a container node (Root, Rule, block at-rule) is opened,
    /// *before* its children are parsed.
    ///
    /// Returns an opaque `NodeRef` that must be passed back to
    /// [`close_container`] and as the `parent` argument for all child calls.
    fn open_container(&mut self, parent: Self::NodeRef, open: ContainerOpen<'src>)
        -> Self::NodeRef;

    /// Called after all of a container's children have been parsed.
    fn close_container(&mut self, handle: Self::NodeRef, close: ContainerClose);

    /// Called for each leaf node (Declaration, Comment, statement at-rule).
    fn emit_leaf(&mut self, parent: Self::NodeRef, leaf: LeafNode<'src>);

    /// Consume the sink and return the finished output.
    fn finish(self) -> Self::Output;
}

// ── TreeSink ───────────────────────────────────────────────────────────────

/// A [`ParseSink`] that builds the `Node<'src>` tree (original behaviour).
pub struct TreeSink<'src> {
    /// In-progress container nodes, outermost first.
    stack: Vec<Node<'src>>,
    /// The finished root node; set by the final `close_container` call.
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
    /// Unit type: we use the stack implicitly, so no explicit handle is needed.
    type NodeRef = ();
    type Output = Node<'src>;

    fn open_container(&mut self, _parent: (), open: ContainerOpen<'src>) {
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
                // ensure_empty_after is applied at close_container time
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
                // Block at-rules always have a nodes collection (possibly empty).
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

    fn close_container(&mut self, _handle: (), close: ContainerClose) {
        let ContainerClose {
            source_end,
            raws_after,
            semicolon,
            own_semicolon,
        } = close;
        let mut node = self
            .stack
            .pop()
            .expect("close_container called with empty stack");
        node.source.end = Some(source_end);
        // Override raws.after (may have been pre-initialized to "")
        node.raws.after = Raw::default();
        apply_raw(&mut node.raws.after, raws_after);
        apply_semi(&mut node.raws.semicolon, semicolon);
        apply_raw(&mut node.raws.own_semicolon, own_semicolon);

        if self.stack.is_empty() {
            // This was the root
            self.root = Some(node);
        } else {
            self.stack.last_mut().unwrap().append(node);
        }
    }

    fn emit_leaf(&mut self, _parent: (), leaf: LeafNode<'src>) {
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
                // Statement at-rules have no nodes block
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
