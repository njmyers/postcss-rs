use crate::Node;
use crate::NodeBuilder;

use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;

pub struct Comment;

impl Comment {
    pub fn create(token: Token, tokenizer: &mut Tokenizer, spaces: &[Token]) -> Node {
        let mut builder = NodeBuilder::new(&token, tokenizer, spaces);
        builder.ensure_empty_before();

        // Extract comment text and left/right spacing from token value
        // Token value is like "/* comment */" - we need to extract "comment" and spaces
        let value = &token.value;

        // Remove /* and */ markers (2 chars each)
        let (text, left, right) = if value.len() >= 4 {
            let inner = &value[2..value.len() - 2];

            // Count leading spaces
            let left_count = inner.chars().take_while(|c| c.is_whitespace()).count();
            // Count trailing spaces
            let right_count = inner.chars().rev().take_while(|c| c.is_whitespace()).count();

            // Extract each part
            let left = &inner[..left_count];
            let right = if right_count > 0 {
                &inner[inner.len() - right_count..]
            } else {
                ""
            };
            let text = if inner.len() > left_count + right_count {
                &inner[left_count..inner.len() - right_count]
            } else {
                ""
            };

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