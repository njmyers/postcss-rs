use crate::at_rule::AtRule;
use crate::comment::Comment;
use crate::node::Source;
use crate::raws::Raws;
use crate::rule_word::RuleWord;
use crate::sinks::{ParseSink, SinkNode};
use crate::Declaration;
use crate::NodeBuilder;
use crate::Statement;
use crate::StatementTerminator;

use serde::Deserialize;
use serde::Serialize;

use std::borrow::Cow;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;
use postcss_rs_tokenizer::Tokens;

/// An empty struct used for creating a css rule.
///
/// Rules are on of the basic parts of css and consist of selectors as well as a list of property
/// declarations.
///
/// ```css
/// .rule {
///   background: white;
/// }
/// ```
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Rule<'src> {
    pub selector: Cow<'src, str>,
}

impl<'src> Rule<'src> {
    /// Parse a rule and emit it + all children to `sink`.
    ///
    /// Returns `source_end` for the caller's last-child tracking.
    pub fn create<S: ParseSink<'src>>(
        token: Token<'src>,
        tokenizer: &mut Tokenizer<'src>,
        spaces: &[Token<'src>],
        sink: &mut S,
        parent: S::NodeRef,
    ) -> Source {
        // Use a temporary NodeBuilder to drive Selector::create (which mutates builder).
        // We extract the accumulated raws afterwards.
        let mut builder = NodeBuilder::new(&token, tokenizer, spaces);
        builder.ensure_empty_before();
        builder.ensure_empty_between();
        builder.ensure_empty_after();

        let source_start = builder.source_start();
        let selector = Selector::create(token, tokenizer, &mut builder);

        let open_raws = builder.into_raws();
        let handle = sink.open(
            parent,
            SinkNode::Rule {
                source_start,
                selector,
                raws: open_raws,
            },
        );

        let mut spaces_buf = Tokens::new();
        let mut body_terminator: Option<StatementTerminator> = None;

        while let Some(body_token) = tokenizer.next() {
            match body_token.kind {
                Kind::Word => {
                    let prop = RuleWord::collect_and_combine(body_token, tokenizer);
                    if Self::peek_for_nested_rule(tokenizer) {
                        Self::create(prop, tokenizer, &spaces_buf, sink, handle);
                        body_terminator = Some(StatementTerminator::new());
                        spaces_buf.clear();
                    } else {
                        let (_, child_term) =
                            Declaration::create(prop, tokenizer, &spaces_buf, sink, handle);
                        body_terminator = Some(child_term);
                        spaces_buf.clear();
                    }
                }
                Kind::AtWord => {
                    let (_, child_term) =
                        AtRule::create(body_token, tokenizer, &spaces_buf, sink, handle);
                    body_terminator = Some(child_term);
                    spaces_buf.clear();
                }
                Kind::Comment => {
                    Comment::create(body_token, tokenizer, &spaces_buf, sink, handle);
                    spaces_buf.clear();
                }
                Kind::Space => spaces_buf.push(body_token),
                Kind::Semicolon => spaces_buf.push(body_token),
                Kind::CloseCurly => break,
                _ => break,
            }
        }

        // Check for trailing semicolon (ownSemicolon)
        if let Some((end_source, own_semicolon)) = Self::consume_trailing_semicolon(tokenizer) {
            let semicolon = body_terminator.map(|bt| bt.semicolon);
            let mut close_raws = Raws::default();
            close_raws.after.push_tokens_or_empty(&spaces_buf);
            match semicolon {
                Some(true) => close_raws.semicolon.set_true(),
                Some(false) => close_raws.semicolon.set_false(),
                None => {}
            }
            close_raws.own_semicolon.push_str(&own_semicolon);
            sink.close(
                handle,
                SinkNode::Close {
                    source_end: end_source,
                    raws: close_raws,
                },
            );
            return end_source;
        }

        let source_end = Source::create_end(tokenizer);
        let semicolon = body_terminator.map(|bt| bt.semicolon);
        let mut close_raws = Raws::default();
        close_raws.after.push_tokens_or_empty(&spaces_buf);
        match semicolon {
            Some(true) => close_raws.semicolon.set_true(),
            Some(false) => close_raws.semicolon.set_false(),
            None => {}
        }
        sink.close(
            handle,
            SinkNode::Close {
                source_end,
                raws: close_raws,
            },
        );

        source_end
    }

    /// Detect and consume trailing semicolons after rule bodies (ownSemicolon pattern)
    /// Returns (source_position, semicolon_string) if a trailing semicolon is found, None otherwise
    ///
    /// PostCSS behavior:
    /// - Column always points at the semicolon
    /// - Offset behavior:
    ///   - Just ";": offset points AT the newline
    ///   - " ;": offset points AFTER the newline (without consuming it)
    fn consume_trailing_semicolon(tokenizer: &mut Tokenizer<'_>) -> Option<(Source, String)> {
        let cp = tokenizer.checkpoint();
        let mut has_space_before = false;
        let mut own_semicolon = String::new();

        // Consume any non-newline spaces before semicolon
        while let Some(token) = tokenizer.peek() {
            if token.kind == Kind::Space && !Self::is_newline(&token) {
                has_space_before = true;
                own_semicolon.push_str(token.value);
                tokenizer.next();
            } else {
                break;
            }
        }

        // Check for semicolon and extract its column
        let semicolon_col = match tokenizer.peek() {
            Some(token) if token.kind == Kind::Semicolon => {
                let column = std::cmp::max(1, token.column);
                own_semicolon.push(';');
                tokenizer.next(); // consume semicolon
                column
            }
            _ => {
                tokenizer.rewind(cp);
                return None;
            }
        };

        // Check if next token is newline (peek, don't consume)
        let newline_len = if has_space_before {
            match tokenizer.peek() {
                Some(t) if Self::is_newline(&t) => t.value.len(),
                _ => 0,
            }
        } else {
            0
        };

        let source = Source {
            offset: tokenizer.index + newline_len,
            line: tokenizer.lines.value(),
            column: semicolon_col,
        };

        Some((source, own_semicolon))
    }

    /// Check if a token is a newline character
    fn is_newline(token: &Token<'_>) -> bool {
        token.kind == Kind::Space && (token.value == "\n" || token.value == "\r\n")
    }

    /// Peek ahead to determine if the next non-space token indicates a nested rule
    fn peek_for_nested_rule(tokenizer: &mut Tokenizer<'_>) -> bool {
        let cp = tokenizer.checkpoint();
        let result = loop {
            match tokenizer.next() {
                None => break false,
                Some(t) if t.kind == Kind::Space => continue,
                Some(t) if t.kind == Kind::OpenCurly => break true,
                Some(t) if t.kind == Kind::Brackets && t.value.starts_with('{') => break true,
                _ => break false,
            }
        };
        tokenizer.rewind(cp);
        result
    }
}

struct Selector;

impl Selector {
    /// A string value representing an empty selector
    const EMPTY_SELECTOR: &'static str = "";

    /// Check if there are selector tokens ahead, skipping over spaces
    fn has_selector_tokens_ahead(tokenizer: &mut Tokenizer<'_>) -> bool {
        let cp = tokenizer.checkpoint();
        let result = loop {
            match tokenizer.next() {
                None => break false,
                Some(t) if t.kind == Kind::Space => continue,
                Some(t)
                    if matches!(
                        t.kind,
                        Kind::Word
                            | Kind::Colon
                            | Kind::Brackets
                            | Kind::OpenSquare
                            | Kind::CloseSquare
                            | Kind::String
                            | Kind::Semicolon
                            | Kind::OpenParen
                            | Kind::CloseParen
                            | Kind::Comment
                    ) =>
                {
                    break true
                }
                _ => break false,
            }
        };
        tokenizer.rewind(cp);
        result
    }

    /// A builder struct used for creating a rule selector.
    pub fn create<'src>(
        token: Token<'src>,
        tokenizer: &mut Tokenizer<'src>,
        builder: &mut NodeBuilder<'src>,
    ) -> Cow<'src, str> {
        if token.kind == Kind::OpenCurly {
            builder.ensure_empty_after();
            builder.ensure_empty_between();

            return Cow::Borrowed(Self::EMPTY_SELECTOR);
        }

        let mut statement = Statement::new();
        statement.push(token);

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Brackets => statement.push(token),
                Kind::Colon => statement.push(token),
                Kind::Word => statement.push(token),
                Kind::OpenSquare => statement.push(token),
                Kind::CloseSquare => statement.push(token),
                Kind::String => statement.push(token),
                Kind::Semicolon => statement.push(token),
                Kind::OpenParen => statement.push(token),
                Kind::CloseParen => statement.push(token),
                Kind::Comment => {
                    // Check if there are more selector tokens after this comment
                    let has_more_selector_tokens = Self::has_selector_tokens_ahead(tokenizer);

                    // If no more selector tokens, this comment belongs in 'between'
                    if !has_more_selector_tokens {
                        builder.between(&token);
                        continue;
                    }

                    // Check if comment contains braces that would be significant
                    let has_braces = token.value.contains('{') || token.value.contains('}');

                    // Comments without braces are part of the selector as-is
                    if !has_braces {
                        statement.push(token);
                        continue;
                    }

                    // Comment with braces: keep in raw only, add spaces to both
                    statement.push_raw_only(token);

                    // Consume following spaces and push to both raw and clean
                    while let Some(next) = tokenizer.peek() {
                        match next.kind {
                            Kind::Space => {
                                let space = tokenizer.next().unwrap();
                                statement.push(space);
                            }
                            _ => break,
                        }
                    }
                }
                Kind::Space => builder.between(&token),
                Kind::OpenCurly => break,
                _ => break,
            };
        }

        // Slice raw selector directly from CSS to avoid an intermediate String.
        let css = tokenizer.source();
        let (rs, re) = statement.raw_span();
        let raw_selector: &str = &css[rs..re];

        let selector: Cow<'src, str> = match statement.clean_span() {
            Some((cs, ce)) => Cow::Borrowed(&css[cs..ce]),
            None => Cow::Owned(statement.to_string_clean()),
        };

        // If raw and clean differ, set the raw selector in builder
        if raw_selector != selector.as_ref() {
            builder.selector_raw(selector.clone(), raw_selector);
        }

        selector
    }
}
