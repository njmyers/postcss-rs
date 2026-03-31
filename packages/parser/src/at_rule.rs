use serde::Deserialize;
use serde::Serialize;

use std::borrow::Cow;

use crate::node::Source;
use crate::sink::{ContainerClose, ContainerOpen, LeafNode, ParseSink};
use crate::Declaration;
use crate::NodeBuilder;
use crate::Rule;
use crate::Statement;
use crate::StatementTerminator;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;
use postcss_rs_tokenizer::Tokens;

/// Data for a CSS at-rule, e.g. `@media screen { … }` or `@import "…";`.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct AtRule<'src> {
    /// The at-rule name without the leading `@`, e.g. `"media"`.
    pub name: Cow<'src, str>,
    /// The at-rule parameters (everything between the name and the block or
    /// semicolon), e.g. `"screen"` for `@media screen { … }`.
    pub params: Option<Cow<'src, str>>,
    /// `true` when the at-rule is followed by a `{ … }` block.
    #[serde(skip)]
    pub has_block: bool,
}

impl<'src> AtRule<'src> {
    /// Parse an at-rule and emit/open it to `sink`.
    ///
    /// Returns `(source_end, terminator)` for the caller's tracking.
    /// For at-rules with a block body, children are also emitted directly into
    /// `sink` by this method.
    pub fn create<S: ParseSink<'src>>(
        token: Token<'src>,
        tokenizer: &mut Tokenizer<'src>,
        spaces: &[Token<'src>],
        sink: &mut S,
        parent: S::NodeRef,
    ) -> (Source, StatementTerminator) {
        let source_start = Source::create_start(&token, tokenizer);
        let raws_before: Option<String> = Some(spaces.iter().map(|t| t.value).collect());
        let name = &token.value[1..]; // strip '@'

        // Build a temporary NodeBuilder just to drive Params::create (which
        // currently mutates a NodeBuilder for after_name and between).
        // We capture the raws from the builder after Params::create returns.
        let mut builder = NodeBuilder::new(&token, tokenizer, spaces);
        builder.ensure_empty_before();
        builder.ensure_empty_between();
        builder.ensure_empty_after_name();

        let (params, terminator, params_end_pos) = Params::create(&mut builder, tokenizer);

        // Extract accumulated raws from the builder
        let raws_after_name: Option<String> = builder
            .node
            .raws
            .after_name
            .as_deref()
            .map(|s| s.to_string());
        let raws_between: Option<String> =
            builder.node.raws.between.as_deref().map(|s| s.to_string());

        match terminator.ended_by {
            Some(Kind::OpenCurly) => {
                // At-rule with body block; always keep params even if empty string,
                // to match PostCSS behaviour (`params: ""` for `@page{ }`).
                let params_cow: Option<Cow<'src, str>> = Some(params);

                // Consume the OpenCurly
                tokenizer.next();

                let handle = sink.open(
                    parent,
                    ContainerOpen::AtRule {
                        source_start,
                        raws_before,
                        name,
                        raws_after_name,
                        params: params_cow,
                        raws_between,
                    },
                );

                let mut body_terminator: Option<StatementTerminator> = None;
                let mut spaces_buf = Tokens::new();

                while let Some(body_token) = tokenizer.next() {
                    match body_token.kind {
                        Kind::Word => {
                            if AtRuleDeclaration::test(tokenizer) {
                                let (_, child_term) = Declaration::create(
                                    body_token,
                                    tokenizer,
                                    &spaces_buf,
                                    sink,
                                    handle,
                                );
                                body_terminator = Some(child_term);
                                spaces_buf.clear();
                            } else {
                                Rule::create(body_token, tokenizer, &spaces_buf, sink, handle);
                                body_terminator = Some(StatementTerminator::new());
                                spaces_buf.clear();
                            }
                        }
                        Kind::AtWord => {
                            let (_, child_term) =
                                AtRule::create(body_token, tokenizer, &spaces_buf, sink, handle);
                            body_terminator = Some(child_term);
                            spaces_buf.clear();
                        }
                        Kind::OpenSquare | Kind::OpenCurly => {
                            Rule::create(body_token, tokenizer, &spaces_buf, sink, handle);
                            body_terminator = Some(StatementTerminator::new());
                            spaces_buf.clear();
                        }
                        Kind::Comment => {
                            use crate::comment::Comment;
                            Comment::create(body_token, tokenizer, &spaces_buf, sink, handle);
                            spaces_buf.clear();
                        }
                        Kind::Space => spaces_buf.push(body_token),
                        Kind::CloseCurly => break,
                        _ => break,
                    }
                }

                let source_end = Source::create_end(tokenizer);

                // raws_after = trailing spaces before '}' inside the block
                let raws_after: String = spaces_buf.iter().map(|t| t.value).collect();
                let semicolon = body_terminator.map(|bt| bt.semicolon);

                sink.close(
                    handle,
                    ContainerClose {
                        source_end: source_end.clone(),
                        raws_after: Some(raws_after),
                        semicolon,
                        own_semicolon: None,
                    },
                );

                (source_end, terminator)
            }

            Some(Kind::CloseCurly) => {
                // Statement at-rule terminated by parent's closing brace.
                // PostCSS leaves source.end absent in this case, so we pass None.
                let params_cow: Option<Cow<'src, str>> = Some(params);

                let source_end = Source::create_end(tokenizer);

                // Consume trailing spaces and add to raws_between
                let mut extra_between = String::new();
                while let Some(peeked) = tokenizer.peek() {
                    if peeked.kind == Kind::Space {
                        let space_token = tokenizer.next().unwrap();
                        extra_between.push_str(space_token.value);
                    } else {
                        break;
                    }
                }

                let raws_between_final = match (&raws_between, extra_between.is_empty()) {
                    (Some(b), false) => Some(format!("{}{}", b, extra_between)),
                    (None, false) => Some(extra_between),
                    _ => raws_between,
                };

                sink.emit(
                    parent,
                    LeafNode::AtRuleStatement {
                        source_start,
                        // params_end_pos is Some when params exist (e.g. @import "a.css"}),
                        // None when there are no params (e.g. @content}).
                        // PostCSS only sets source.end when params are present.
                        source_end: params_end_pos,
                        raws_before,
                        name,
                        raws_after_name,
                        params: params_cow,
                        raws_between: raws_between_final,
                    },
                );

                (source_end, terminator)
            }

            _ => {
                // Statement at-rule (semicolon or end-of-input)
                let params_cow: Option<Cow<'src, str>> = Some(params);

                let source_end = if let Some(end_pos) = params_end_pos {
                    end_pos
                } else {
                    Source::create_end(tokenizer)
                };

                sink.emit(
                    parent,
                    LeafNode::AtRuleStatement {
                        source_start,
                        source_end: Some(source_end.clone()),
                        raws_before,
                        name,
                        raws_after_name,
                        params: params_cow,
                        raws_between,
                    },
                );

                (source_end, terminator)
            }
        }
    }
}

/// An empty struct representing the rules for determining if a word token following an at rule is
/// part of a declaration or not
struct AtRuleDeclaration;

impl AtRuleDeclaration {
    /// Test whether the the tokens following an at word rule are part of a declaration or not
    pub fn test(tokenizer: &mut Tokenizer<'_>) -> bool {
        let cp = tokenizer.checkpoint();
        let result = loop {
            match tokenizer.next() {
                None => break true,
                Some(t) if t.kind == Kind::OpenCurly => break false,
                Some(t) if t.kind == Kind::CloseCurly => break true,
                _ => {}
            }
        };
        tokenizer.rewind(cp);
        result
    }
}

/// An empty struct used for creating a css at rule.
///
/// The struct handles concatenating word tokens as part of the `params` value as well as correctly
/// setting spaces depending on whether at rule has a parameter value
///
/// At rules are up the basic grammar of css and consist of names and parameters. They can
/// additionally include nested rules, at rules or a list of property declarations.
///
/// ```css
/// @rule {
///   .rule {
///     background: white;
///   }
/// }
/// ```
struct Params;

impl Params {
    /// Peek ahead past spaces to see if there's a terminator (CloseCurly)
    fn peek_for_terminator_after_space(tokenizer: &mut Tokenizer<'_>) -> bool {
        let cp = tokenizer.checkpoint();
        tokenizer.next(); // skip the current space token
        let result = loop {
            match tokenizer.next() {
                None => break false,
                Some(t) if t.kind == Kind::Space => continue,
                Some(t) if t.kind == Kind::CloseCurly => break true,
                _ => break false,
            }
        };
        tokenizer.rewind(cp);
        result
    }

    fn create<'src>(
        parent: &mut NodeBuilder<'src>,
        tokenizer: &mut Tokenizer<'src>,
    ) -> (Cow<'src, str>, StatementTerminator, Option<crate::Source>) {
        let mut spaces = Tokens::new();
        let mut statement = Statement::new();
        let mut terminator = StatementTerminator::new();
        let mut between_buffer = Tokens::new(); // Buffer for potential 'between' spaces
        let mut params_end_pos: Option<crate::Source> = None; // Track where params actually end

        while let Some(token) = tokenizer.peek() {
            // Check if this token should end params without being consumed
            match token.kind {
                Kind::OpenCurly if !statement.is_open() => {
                    terminator.set_ended_by(Kind::OpenCurly);
                    break;
                }
                Kind::CloseCurly if !statement.is_open() => {
                    terminator.set_ended_by(Kind::CloseCurly);
                    break;
                }
                Kind::AtWord => {
                    terminator.set_ended_by(Kind::AtWord);
                    break;
                }
                // If we see a space, peek ahead to check if it's followed by a terminator
                Kind::Space if !statement.is_open() => {
                    if Self::peek_for_terminator_after_space(tokenizer) {
                        terminator.set_ended_by(Kind::CloseCurly);
                        break;
                    }
                }
                _ => {}
            };

            // Consume the token from tokenizer
            let token = tokenizer.next().unwrap();
            terminator.process(&token);

            match token.kind {
                // Handle spacing and comments before params start
                Kind::Space | Kind::Comment if statement.is_empty() => spaces.push(token),
                // Handle spacing and comments within params - add to statement when brackets are open
                Kind::Space | Kind::Comment if statement.is_open() => {
                    statement.push(token);
                    params_end_pos = Some(crate::Source::create_end(tokenizer));
                }
                // Handle spacing and comments after params - buffer them for now
                Kind::Space | Kind::Comment => between_buffer.push(token),

                // Handle semicolon - ends params only when not nested
                Kind::Semicolon => {
                    params_end_pos = Some(crate::Source::create_end(tokenizer));
                    if statement.is_open() {
                        statement.push(token);
                    } else {
                        terminator.set_ended_by(Kind::Semicolon);
                        break;
                    }
                }

                // Handle all other content tokens - update end position after each.
                // OpenCurly/CloseCurly only reach here when statement.is_open() (guarded in peek loop above).
                Kind::Word
                | Kind::Brackets
                | Kind::Colon
                | Kind::String
                | Kind::OpenSquare
                | Kind::CloseSquare
                | Kind::OpenCurvy
                | Kind::CloseCurvy
                | Kind::OpenCurly
                | Kind::CloseCurly => {
                    statement.push(token);
                    params_end_pos = Some(crate::Source::create_end(tokenizer));
                }

                // Unreachable: AtWord causes a break in the peek loop before consuming
                Kind::AtWord => break,
            };
        }

        Self::spaces(&statement, parent, &spaces);

        // Only add buffered spaces to 'between' if there's a body (OpenCurly terminator)
        if matches!(
            terminator.ended_by,
            Some(Kind::OpenCurly) | Some(Kind::CloseCurly)
        ) {
            for token in between_buffer.iter() {
                parent.between(token);
            }
        }
        // Otherwise, buffered spaces are trailing and should not be part of the at-rule

        // Build the params Cow from the raw CSS span (avoids allocation for the
        // common case where no comments are stripped).
        let css = tokenizer.source();
        let (rs, re) = statement.raw_span();
        let params: Cow<'src, str> = Cow::Borrowed(&css[rs..re]);

        return (params, terminator, params_end_pos);
    }

    /// Set the spaces for the parent node builder depending on whether the param exists or not.
    ///
    /// If an at rule has a parameter value then the spaces will be `after_name`. If not they will
    /// be part of `between`.
    fn spaces<'src>(
        statement: &Statement<'_>,
        parent: &mut NodeBuilder<'src>,
        spaces: &Tokens<'_>,
    ) {
        for token in spaces.iter() {
            if !statement.is_empty() {
                parent.after_name(token);
            } else {
                parent.between(token);
            }
        }
    }
}
