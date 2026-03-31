use crate::node::Source;
use crate::sinks::{LeafNode, ParseSink};

use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;

/// Parser for CSS block comments.
///
/// Extracts the comment text and its surrounding whitespace from a raw
/// `/* … */` token and builds a [`Node`] with the appropriate `raws.left`
/// and `raws.right` fields set.
pub struct Comment;

impl Comment {
    /// Generic version: parse a comment token and emit it to `sink`.
    ///
    /// Returns the `source_end` of the produced node (so the caller can update
    /// any last-child tracking).
    pub fn create<'src, S: ParseSink<'src>>(
        token: Token<'src>,
        tokenizer: &mut Tokenizer<'src>,
        spaces: &[Token<'src>],
        sink: &mut S,
        parent: S::NodeRef,
    ) -> Source {
        let source_start = Source::create_start(&token, tokenizer);
        let raws_before = Self::raws_before(spaces);

        let value = &token.value;
        let (text, left, right) = if value.len() >= 4 {
            let inner = &value[2..value.len() - 2];
            let left_count = inner.chars().take_while(|c| c.is_whitespace()).count();
            let left = &inner[..left_count];
            let remaining = &inner[left_count..];
            let right_count = remaining
                .chars()
                .rev()
                .take_while(|c| c.is_whitespace())
                .count();
            let right = if right_count > 0 {
                &remaining[remaining.len() - right_count..]
            } else {
                ""
            };
            let text = &remaining[..remaining.len() - right_count];
            (text, left, right)
        } else {
            ("", "", "")
        };

        let source_end = Source::create_end(tokenizer);

        sink.emit(
            parent,
            LeafNode::Comment {
                source_start,
                source_end: source_end.clone(),
                raws_before,
                raws_left: Some(left.to_string()),
                text,
                raws_right: Some(right.to_string()),
            },
        );

        source_end
    }

    /// Build an `Option<String>` from accumulated space tokens (for `raws.before`).
    pub(crate) fn raws_before(spaces: &[Token<'_>]) -> Option<String> {
        // Always Some (ensure_empty_before semantics): even empty → Some("")
        let s: String = spaces.iter().map(|t| t.value).collect();
        Some(s)
    }
}
