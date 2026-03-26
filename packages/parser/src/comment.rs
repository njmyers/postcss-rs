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
    pub fn create(token: Token<'_>, tokenizer: &mut Tokenizer<'_>, spaces: &[Token<'_>]) -> Node {
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
            let right_count = remaining.chars().rev().take_while(|c| c.is_whitespace()).count();
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