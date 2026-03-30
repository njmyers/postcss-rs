use crate::node::Source;
use crate::sink::{LeafNode, ParseSink};
use crate::Node;
use crate::NodeBuilder;

use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;

/// Parser for CSS block comments.
///
/// Extracts the comment text and its surrounding whitespace from a raw
/// `/* … */` token and builds a [`Node`] with the appropriate `raws.left`
/// and `raws.right` fields set.
pub struct Comment;

impl Comment {
    /// Parses a comment token into a [`Node`].
    ///
    /// `token` is the raw comment token (value like `"/* text */"`).
    /// `spaces` are any whitespace tokens that appear before the comment in the
    /// source; they are placed in `raws.before`.
    pub fn create<'src>(token: Token<'src>, tokenizer: &mut Tokenizer<'src>, spaces: &[Token<'src>]) -> Node<'src> {
        let mut builder = NodeBuilder::new(&token, tokenizer, spaces);
        builder.ensure_empty_before();

        // Extract comment text and left/right spacing from token value
        // Token value is like "/* comment */" - we need to extract "comment" and spaces
        let value = &token.value;

        // Remove /* and */ markers (2 chars each)
        let (text, left, right) = if value.len() >= 4 {
            let inner = &value[2..value.len() - 2];

            // Count leading spaces first
            let left_count = inner.chars().take_while(|c| c.is_whitespace()).count();
            let left = &inner[..left_count];

            // Count trailing spaces from what remains after left spaces
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

            // Text is what's between left and right spaces
            let text = &remaining[..remaining.len() - right_count];

            (text, left, right)
        } else {
            ("", "", "")
        };

        // Set the left and right raws
        builder.comment_left(left);
        builder.comment_right(right);
        builder.comment_text(text);
        builder.comment();

        return builder.build(tokenizer, &[]);
    }

    /// Generic version: parse a comment token and emit it to `sink`.
    ///
    /// Returns the `source_end` of the produced node (so the caller can update
    /// any last-child tracking).
    pub fn create_into<'src, S: ParseSink<'src>>(
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
            let right_count = remaining.chars().rev().take_while(|c| c.is_whitespace()).count();
            let right = if right_count > 0 { &remaining[remaining.len() - right_count..] } else { "" };
            let text = &remaining[..remaining.len() - right_count];
            (text, left, right)
        } else {
            ("", "", "")
        };

        let source_end = Source::create_end(tokenizer);

        sink.emit_leaf(parent, LeafNode::Comment {
            source_start,
            source_end: source_end.clone(),
            raws_before,
            raws_left: Some(left.to_string()),
            text,
            raws_right: Some(right.to_string()),
        });

        source_end
    }

    /// Build an `Option<String>` from accumulated space tokens (for `raws.before`).
    pub(crate) fn raws_before(spaces: &[Token<'_>]) -> Option<String> {
        // Always Some (ensure_empty_before semantics): even empty → Some("")
        let s: String = spaces.iter().map(|t| t.value).collect();
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Data;
    use postcss_rs_tokenizer::{Kind, Tokenizer};

    fn comment_token<'a>(value: &'a str) -> Token<'a> {
        Token {
            kind: Kind::Comment,
            value,
            start: 0,
            end: value.len(),
            column: 1,
        }
    }

    #[test]
    fn test_comment_plain_text() {
        let mut tokenizer = Tokenizer::new("");
        let node = Comment::create(comment_token("/* hello */"), &mut tokenizer, &[]);
        if let Data::Comment(data) = &node.data {
            assert_eq!(data.text, "hello");
        } else {
            panic!("expected comment node");
        }
        assert_eq!(node.raws.left.as_deref(), Some(" "));
        assert_eq!(node.raws.right.as_deref(), Some(" "));
    }

    #[test]
    fn test_comment_no_whitespace() {
        let mut tokenizer = Tokenizer::new("");
        let node = Comment::create(comment_token("/*hello*/"), &mut tokenizer, &[]);
        if let Data::Comment(data) = &node.data {
            assert_eq!(data.text, "hello");
        } else {
            panic!("expected comment node");
        }
        assert_eq!(node.raws.left.as_deref(), Some(""));
        assert_eq!(node.raws.right.as_deref(), Some(""));
    }

    #[test]
    fn test_comment_empty() {
        let mut tokenizer = Tokenizer::new("");
        let node = Comment::create(comment_token("/* */"), &mut tokenizer, &[]);
        if let Data::Comment(data) = &node.data {
            assert_eq!(data.text, "");
        } else {
            panic!("expected comment node");
        }
    }

    #[test]
    fn test_comment_asymmetric_whitespace() {
        let mut tokenizer = Tokenizer::new("");
        let node = Comment::create(comment_token("/*  text */"), &mut tokenizer, &[]);
        if let Data::Comment(data) = &node.data {
            assert_eq!(data.text, "text");
        } else {
            panic!("expected comment node");
        }
        assert_eq!(node.raws.left.as_deref(), Some("  "));
        assert_eq!(node.raws.right.as_deref(), Some(" "));
    }
}
