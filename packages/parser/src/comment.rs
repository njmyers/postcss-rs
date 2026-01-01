use crate::Node;
use crate::NodeBuilder;

use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;

pub struct Comment;

impl Comment {
    pub fn create(token: Token, tokenizer: &mut Tokenizer, spaces: &[Token]) -> Node {
        Self::create_impl(token, tokenizer, spaces, false)
    }

    pub fn create_first(token: Token, tokenizer: &mut Tokenizer, spaces: &[Token]) -> Node {
        Self::create_impl(token, tokenizer, spaces, true)
    }

    fn create_impl(token: Token, tokenizer: &mut Tokenizer, spaces: &[Token], is_first: bool) -> Node {
        let mut builder = match is_first {
            true => NodeBuilder::new_first_node(&token, tokenizer, spaces),
            false => NodeBuilder::new(&token, tokenizer, spaces),
        };
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