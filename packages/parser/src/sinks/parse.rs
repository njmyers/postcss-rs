use std::borrow::Cow;

use crate::node::{Source, Sources};
use crate::raws::Raws;

/// A single node event passed to [`ParseSink`] methods.
///
/// Variants `Root`, `Rule`, and `AtRule` are container-open events.
/// `Close` finalises the most-recently-opened container after its children
/// have been parsed.  `Declaration`, `Comment`, and `AtRuleStatement` are
/// leaf events.
pub enum SinkNode<'src> {
    // ── Container-open variants ──────────────────────────────────────────────
    /// The root node that wraps the entire stylesheet.
    Root { source_start: Source },

    /// A rule node, e.g. `.foo { … }`.
    Rule {
        source_start: Source,
        /// Comment-stripped selector string.
        selector: Cow<'src, str>,
        /// Raws: `before`, `between`, and `selector` (ValueRaw) are populated.
        raws: Raws<'src>,
    },

    /// An at-rule with a block body, e.g. `@media screen { … }`.
    AtRule {
        source_start: Source,
        /// Name without the leading `@`.
        name: &'src str,
        /// At-rule params; `None` when there are none.
        params: Option<Cow<'src, str>>,
        /// Raws: `before`, `after_name`, and `between` are populated.
        raws: Raws<'src>,
    },

    // ── Container-close variant ──────────────────────────────────────────────
    /// Finalises the most-recently-opened container node.
    ///
    /// Raws: `after`, `semicolon`, and `own_semicolon` are populated.
    Close {
        source_end: Source,
        raws: Raws<'src>,
    },

    // ── Leaf variants ────────────────────────────────────────────────────────
    /// A property declaration, e.g. `color: red`.
    Declaration {
        sources: Sources,
        prop: Cow<'src, str>,
        /// Comment-stripped value.
        value: Cow<'src, str>,
        important: Option<bool>,
        /// Raws: `before`, `between`, `value` (ValueRaw), and `important` are populated.
        raws: Raws<'src>,
    },

    /// A block comment, e.g. `/* text */`.
    Comment {
        sources: Sources,
        text: &'src str,
        /// Raws: `before`, `left`, and `right` are populated.
        raws: Raws<'src>,
    },

    /// A statement at-rule (no block), e.g. `@import "…";`.
    AtRuleStatement {
        sources: Sources,
        name: &'src str,
        params: Option<Cow<'src, str>>,
        /// Raws: `before`, `after_name`, and `between` are populated.
        raws: Raws<'src>,
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
    /// `node` must be one of the container-open variants: `Root`, `Rule`, or
    /// `AtRule`.  Returns an opaque `NodeRef` that must be passed back to
    /// [`close`] and as the `parent` argument for all child calls.
    fn open(&mut self, parent: Self::NodeRef, node: SinkNode<'src>) -> Self::NodeRef;

    /// Called after all of a container's children have been parsed.
    ///
    /// `node` must be the `Close` variant.
    fn close(&mut self, handle: Self::NodeRef, node: SinkNode<'src>);

    /// Called for each leaf node (Declaration, Comment, statement at-rule).
    fn emit(&mut self, parent: Self::NodeRef, node: SinkNode<'src>);

    /// Consume the sink and return the finished output.
    fn finish(self) -> Self::Output;
}
