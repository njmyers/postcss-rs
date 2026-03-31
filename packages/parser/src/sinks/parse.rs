use std::borrow::Cow;

use crate::node::Source;

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
