use serde::Deserialize;
use serde::Serialize;

use std::borrow::Cow;

use crate::comment::Comment;
use crate::node::{Source, Sources};
use crate::raws::Raws;
use crate::sinks::{ParseSink, SinkNode};
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
        let name = &token.value[1..]; // strip '@'

        // Build a temporary NodeBuilder just to drive Params::create (which
        // mutates a NodeBuilder for after_name and between).
        // We move the accumulated raws from the builder after Params::create returns.
        let mut builder = NodeBuilder::new(&token, tokenizer, spaces);
        builder.ensure_empty_before();
        builder.ensure_empty_between();
        builder.ensure_empty_after_name();

        let (params, terminator, params_end_pos) = Params::create(&mut builder, tokenizer);

        let source_start = builder.node.source.start.clone();
        let open_raws = builder.node.raws;

        match terminator.ended_by {
            Some(Kind::OpenCurly) => {
                // At-rule with body block; always keep params even if empty string,
                // to match PostCSS behaviour (`params: ""` for `@page{ }`).
                let params_cow: Option<Cow<'src, str>> = Some(params);

                // Consume the OpenCurly
                tokenizer.next();

                let handle = sink.open(
                    parent,
                    SinkNode::AtRule {
                        source_start,
                        name,
                        params: params_cow,
                        raws: open_raws,
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
                let mut close_raws = Raws::default();
                close_raws.after.push_str(&raws_after);
                match semicolon {
                    Some(true) => close_raws.semicolon.set_true(),
                    Some(false) => close_raws.semicolon.set_false(),
                    None => {}
                }

                sink.close(
                    handle,
                    SinkNode::Close {
                        source_end: source_end.clone(),
                        raws: close_raws,
                    },
                );

                (source_end, terminator)
            }

            Some(Kind::CloseCurly) => {
                // Statement at-rule terminated by parent's closing brace.
                // PostCSS leaves source.end absent in this case, so we pass None.
                let params_cow: Option<Cow<'src, str>> = Some(params);

                let source_end = Source::create_end(tokenizer);

                // Consume trailing spaces and append to raws.between
                let mut leaf_raws = open_raws;
                while let Some(peeked) = tokenizer.peek() {
                    if peeked.kind == Kind::Space {
                        let space_token = tokenizer.next().unwrap();
                        leaf_raws.between.push_str(space_token.value);
                    } else {
                        break;
                    }
                }

                sink.emit(
                    parent,
                    SinkNode::AtRuleStatement {
                        // params_end_pos is Some when params exist (e.g. @import "a.css"}),
                        // None when there are no params (e.g. @content}).
                        // PostCSS only sets source.end when params are present.
                        sources: Sources {
                            start: source_start,
                            end: params_end_pos,
                        },
                        name,
                        params: params_cow,
                        raws: leaf_raws,
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
                    SinkNode::AtRuleStatement {
                        sources: Sources {
                            start: source_start,
                            end: Some(source_end.clone()),
                        },
                        name,
                        params: params_cow,
                        raws: open_raws,
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
    ) -> (Cow<'src, str>, StatementTerminator, Option<Source>) {
        let mut spaces = Tokens::new();
        let mut statement = Statement::new();
        let mut terminator = StatementTerminator::new();
        let mut between_buffer = Tokens::new(); // Buffer for potential 'between' spaces
        let mut params_end_pos: Option<Source> = None; // Track where params actually end

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
                    params_end_pos = Some(Source::create_end(tokenizer));
                }
                // Handle spacing and comments after params - buffer them for now
                Kind::Space | Kind::Comment => between_buffer.push(token),

                // Handle semicolon - ends params only when not nested
                Kind::Semicolon => {
                    params_end_pos = Some(Source::create_end(tokenizer));
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
                    params_end_pos = Some(Source::create_end(tokenizer));
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
