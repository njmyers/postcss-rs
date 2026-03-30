use crate::comment::Comment;
use crate::node::Source;
use crate::rule_word::RuleWord;
use crate::sink::{ContainerClose, ContainerOpen, ParseSink};
use crate::Declaration;
use crate::Node;
use crate::NodeBuilder;
use crate::Statement;

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
    pub fn create(
        token: Token<'src>,
        tokenizer: &mut Tokenizer<'src>,
        spaces: &[Token<'src>],
    ) -> Node<'src> {
        let mut builder = NodeBuilder::new(&token, tokenizer, spaces);
        let selector = Selector::create(token, tokenizer, &mut builder);

        builder.ensure_empty_before();
        builder.ensure_empty_between();
        builder.ensure_empty_after();
        builder.rule(Self::new(selector));

        let mut spaces = Tokens::new();

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Word => {
                    // Combine consecutive Word tokens for property names with escape sequences
                    let prop = RuleWord::collect_and_combine(token, tokenizer);

                    // Nested rule: word followed by opening brace (possibly with spaces)
                    if Self::peek_for_nested_rule(tokenizer) {
                        let nested_rule = Self::create(prop, tokenizer, &spaces);
                        // Nested rules don't end with semicolons, so reset the terminator
                        let terminator = crate::StatementTerminator::new();
                        builder.terminator(&terminator);
                        builder.append(nested_rule);
                        spaces.clear();
                        continue;
                    }

                    // Declaration: word followed by colon or other tokens
                    let (child, terminator) = Declaration::create(prop, tokenizer, &spaces);
                    builder.terminator(&terminator);
                    builder.append(child);
                    spaces.clear();
                }

                Kind::AtWord => {
                    let (mut node, terminator) = crate::AtRule::create(token, tokenizer, &spaces);

                    // For at-rules inside rules that end with CloseCurly, we need to:
                    // 1. Set the end source position (before trailing spaces)
                    // 2. Consume trailing spaces and add to 'between'
                    if let Some(Kind::CloseCurly) = terminator.ended_by {
                        if node.source.end.is_none() {
                            // Save end position before consuming spaces
                            node.source.end = Some(crate::Source::create_end(tokenizer));

                            // Consume trailing spaces and add to 'between'
                            while let Some(peeked) = tokenizer.peek() {
                                match peeked.kind {
                                    Kind::Space => {
                                        let space_token = tokenizer.next().unwrap();
                                        node.raws.between.push(&space_token);
                                    }
                                    _ => break,
                                }
                            }
                        }
                    }

                    builder.terminator(&terminator);
                    builder.append(node);
                    spaces.clear();
                }

                Kind::Comment => {
                    let comment = Self::create_comment(token, &mut *tokenizer, &spaces);
                    builder.append(comment);
                    spaces.clear();
                }

                Kind::Space => spaces.push(token),
                Kind::Semicolon => spaces.push(token),
                Kind::CloseCurly => break,
                _ => break,
            };
        }

        // Check for trailing semicolons (ownSemicolon in PostCSS)
        // Handles cases like: a{b:c}; or a{b:c} ;
        if let Some((end_source, own_semicolon)) = Self::consume_trailing_semicolon(tokenizer) {
            let mut node = builder.node;
            node.raws.after.push_tokens(&spaces);
            node.raws.own_semicolon = crate::Raw::new(&own_semicolon);
            node.source.end = Some(end_source);
            return node;
        }

        return builder.build(tokenizer, &spaces);
    }

    /// Generic version: parse a rule and emit it + all children to `sink`.
    ///
    /// Returns `source_end` for the caller's last-child tracking.
    pub fn create_into<S: ParseSink<'src>>(
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

        let source_start = builder.node.source.start.clone();
        let selector = Selector::create(token, tokenizer, &mut builder);

        let raws_before: Option<String> =
            builder.node.raws.before.as_deref().map(|s| s.to_string());
        let raws_selector_raw: Option<String> = builder.node.raws.selector.raw.clone();
        let raws_between: Option<String> =
            builder.node.raws.between.as_deref().map(|s| s.to_string());

        let handle = sink.open_container(
            parent,
            ContainerOpen::Rule {
                source_start,
                raws_before,
                selector,
                raws_selector_raw,
                raws_between,
            },
        );

        let mut spaces_buf = Tokens::new();
        let mut last_child_end: Option<Source> = None;
        let mut body_terminator: Option<crate::StatementTerminator> = None;

        while let Some(body_token) = tokenizer.next() {
            match body_token.kind {
                Kind::Word => {
                    let prop = RuleWord::collect_and_combine(body_token, tokenizer);
                    if Self::peek_for_nested_rule(tokenizer) {
                        let child_end =
                            Self::create_into(prop, tokenizer, &spaces_buf, sink, handle);
                        body_terminator = Some(crate::StatementTerminator::new());
                        last_child_end = Some(child_end);
                        spaces_buf.clear();
                    } else {
                        let (child_end, child_term) =
                            Declaration::create_into(prop, tokenizer, &spaces_buf, sink, handle);
                        body_terminator = Some(child_term);
                        last_child_end = Some(child_end);
                        spaces_buf.clear();
                    }
                }
                Kind::AtWord => {
                    use crate::at_rule::AtRule;
                    let (child_end, child_term) =
                        AtRule::create_into(body_token, tokenizer, &spaces_buf, sink, handle);

                    // For at-rules terminated by CloseCurly, the trailing spaces
                    // are already consumed inside AtRule::create_into.
                    if matches!(child_term.ended_by, Some(Kind::CloseCurly)) {
                        // child_end already accounts for this case
                    }

                    body_terminator = Some(child_term);
                    last_child_end = Some(child_end);
                    spaces_buf.clear();
                }
                Kind::Comment => {
                    let child_end =
                        Comment::create_into(body_token, tokenizer, &spaces_buf, sink, handle);
                    last_child_end = Some(child_end);
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
            let raws_after: String = spaces_buf.iter().map(|t| t.value).collect();
            let semicolon = body_terminator.map(|bt| bt.semicolon);
            sink.close_container(
                handle,
                ContainerClose {
                    source_end: end_source.clone(),
                    raws_after: Some(raws_after),
                    semicolon,
                    own_semicolon: Some(own_semicolon),
                },
            );
            return end_source;
        }

        let source_end = Source::create_end(tokenizer);
        // raws.after = trailing spaces before '}' (includes semicolons too per PostCSS)
        let raws_after = Some(spaces_buf.iter().map(|t| t.value).collect::<String>());
        let semicolon = body_terminator.map(|bt| bt.semicolon);

        sink.close_container(
            handle,
            ContainerClose {
                source_end: source_end.clone(),
                raws_after,
                semicolon,
                own_semicolon: None,
            },
        );

        source_end
    }

    fn new(selector: Cow<'src, str>) -> Self {
        return Self { selector };
    }

    fn create_comment(
        token: Token<'src>,
        tokenizer: &mut Tokenizer<'src>,
        spaces: &[Token<'src>],
    ) -> Node<'src> {
        return crate::Comment::create(token, tokenizer, spaces);
    }

    /// Detect and consume trailing semicolons after rule bodies (ownSemicolon pattern)
    /// Returns (source_position, semicolon_string) if a trailing semicolon is found, None otherwise
    ///
    /// PostCSS behavior:
    /// - Column always points at the semicolon
    /// - Offset behavior:
    ///   - Just ";": offset points AT the newline
    ///   - " ;": offset points AFTER the newline (without consuming it)
    fn consume_trailing_semicolon(
        tokenizer: &mut Tokenizer<'_>,
    ) -> Option<(crate::Source, String)> {
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

        let source = crate::Source {
            offset: tokenizer.index + newline_len,
            line: tokenizer.lines.value(),
            column: semicolon_col,
        };

        return Some((source, own_semicolon));
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
                            | Kind::OpenCurvy
                            | Kind::CloseCurvy
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
                Kind::OpenCurvy => statement.push(token),
                Kind::CloseCurvy => statement.push(token),
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
            builder.selector_raw(selector.as_ref(), raw_selector);
        }

        return selector;
    }
}
