use std::borrow::Cow;

use crate::node::{Source, Sources};
use crate::raws::Raws;

/// Data carried when a container node is opened (its children have not yet been parsed).
pub enum ContainerOpen<'src> {
    /// The root node that wraps the entire stylesheet.
    Root { source_start: Source },

    /// A rule node, e.g. `.foo { … }`.
    Rule {
        source_start: Source,
        /// Comment-stripped selector string.
        selector: Cow<'src, str>,
        /// Node raws: `before`, `between`, and `selector` (ValueRaw) are populated.
        raws: Raws,
    },

    /// An at-rule with a block body, e.g. `@media screen { … }`.
    AtRule {
        source_start: Source,
        /// Name without the leading `@`.
        name: &'src str,
        /// At-rule params; `None` when there are none.
        params: Option<Cow<'src, str>>,
        /// Node raws: `before`, `after_name`, and `between` are populated.
        raws: Raws,
    },
}

/// Data carried when a container node is closed (all children have been parsed).
pub struct ContainerClose {
    pub source_end: Source,
    /// Node raws: `after`, `semicolon`, and `own_semicolon` are populated.
    pub raws: Raws,
}

/// A leaf (non-container) node.
pub enum LeafNode<'src> {
    Declaration {
        sources: Sources,
        prop: Cow<'src, str>,
        /// Comment-stripped value.
        value: Cow<'src, str>,
        important: Option<bool>,
        /// Node raws: `before`, `between`, `value` (ValueRaw), and `important` are populated.
        raws: Raws,
    },
    Comment {
        sources: Sources,
        text: &'src str,
        /// Node raws: `before`, `left`, and `right` are populated.
        raws: Raws,
    },
    /// A statement at-rule (no block), e.g. `@import "…";`.
    AtRuleStatement {
        sources: Sources,
        name: &'src str,
        params: Option<Cow<'src, str>>,
        /// Node raws: `before`, `after_name`, and `between` are populated.
        raws: Raws,
    },
}

/// Output abstraction for the CSS parser.
///
/// Implement this trait to receive the parsed CSS in any form.
/// Two implementations are provided:
///
/// - [`TreeSink`] — builds the `Node<'src>` tree.
/// - [`OffsetSink`] — emits flat `NodeSlot`s without an intermediate tree.
pub trait ParseSink<'src> {
    /// Opaque handle used to identify a container node.
    ///
    /// Must implement `Copy` and `Default` (the default is used as a dummy
    /// parent for the root `open` call).
    type NodeRef: Copy + Default;

    /// The value returned by [`finish`].
    type Output;

    /// Called when a container node (Root, Rule, block at-rule) is opened,
    /// *before* its children are parsed.
    ///
    /// Returns an opaque `NodeRef` that must be passed back to
    /// [`close`] and as the `parent` argument for all child calls.
    fn open(&mut self, parent: Self::NodeRef, open: ContainerOpen<'src>) -> Self::NodeRef;

    /// Called after all of a container's children have been parsed.
    fn close(&mut self, handle: Self::NodeRef, close: ContainerClose);

    /// Called for each leaf node (Declaration, Comment, statement at-rule).
    fn emit(&mut self, parent: Self::NodeRef, leaf: LeafNode<'src>);

    /// Consume the sink and return the finished output.
    fn finish(self) -> Self::Output;
}
