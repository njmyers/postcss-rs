use crate::node::{Source, Sources};
use crate::raws::{Raw, Raws};
use crate::sinks::{ParseSink, SinkNode};

use serde::Deserialize;
use serde::Serialize;

use std::borrow::Cow;

use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;

/// Text payload for a CSS comment node, e.g. `/* text */`.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Comment<'src> {
    /// The comment text, without the surrounding `/*` and `*/` delimiters or
    /// any leading/trailing whitespace (which is stored in `raws.left` /
    /// `raws.right`).
    pub text: Cow<'src, str>,
}

impl<'src> Comment<'src> {
    /// Parse a comment token and emit it to `sink`.
    ///
    /// Returns the `source_end` of the produced node (so the caller can update
    /// any last-child tracking).
    pub fn create<S: ParseSink<'src>>(
        token: Token<'src>,
        tokenizer: &mut Tokenizer<'src>,
        spaces: &[Token<'src>],
        sink: &mut S,
        parent: S::NodeRef,
    ) -> Source {
        let source_start = Source::create_start(&token, tokenizer);
        let mut raws_before = Raw::default();
        raws_before.push_tokens_or_empty(spaces);

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

        let mut emit_raws = Raws::default();
        emit_raws.before = raws_before;
        emit_raws.left.push_str(left);
        emit_raws.right.push_str(right);

        sink.emit(
            parent,
            SinkNode::Comment {
                sources: Sources {
                    start: source_start,
                    end: Some(source_end),
                },
                text,
                raws: emit_raws,
            },
        );

        source_end
    }

}
