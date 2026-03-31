use crate::characters;
use crate::node::Source;
use crate::sink::{LeafNode, ParseSink};
use crate::Statement;
use crate::StatementTerminator;

use serde::Deserialize;
use serde::Serialize;

use std::borrow::Cow;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;

/// Data for a CSS property declaration, e.g. `color: red`.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Declaration<'src> {
    /// The property name, e.g. `"color"`.
    pub prop: Cow<'src, str>,
    /// `Some(true)` when the declaration contains `!important`, `None` otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub important: Option<bool>,
    /// The property value, stripped of inline comments, e.g. `"red"`.
    pub value: Cow<'src, str>,
}

impl<'src> Declaration<'src> {
    /// Parses a declaration and emits it to `sink`.
    ///
    /// Returns `(source_end, terminator)` so the caller can update the parent
    /// container's position and semicolon tracking.
    pub fn create<S: ParseSink<'src>>(
        prop: Token<'src>,
        tokenizer: &mut Tokenizer<'src>,
        spaces: &[Token<'src>],
        sink: &mut S,
        parent: S::NodeRef,
    ) -> (Source, StatementTerminator) {
        let source_start = Source::create_start(&prop, tokenizer);
        let mut raws_before: String = spaces.iter().map(|t| t.value).collect();
        let mut raws_between = String::new();

        let (actual_prop, ie_hack_prefix) = Self::extract_ie_hack_prefix(prop.value);
        if let Some(prefix) = ie_hack_prefix {
            raws_before.push_str(prefix);
        }

        // Phase 1: consume up to and including the colon
        let first_value_token: Option<Token<'src>> = {
            let mut result = None;
            for token in tokenizer.by_ref() {
                match token.kind {
                    Kind::Colon => {
                        raws_between.push_str(token.value);
                        break;
                    }
                    Kind::Word if token.value.starts_with(':') => {
                        let (colon_token, value_token) = token.split(1, Kind::Colon, Kind::Word);
                        raws_between.push_str(colon_token.value);
                        result = value_token;
                        break;
                    }
                    _ => raws_between.push_str(token.value),
                }
            }
            result
        };

        // Phase 2: collect value tokens
        let mut statement = Statement::new();
        let mut terminator = StatementTerminator::new();
        let mut pending_spaces: Vec<Token<'src>> = Vec::new();
        let mut important_raw: Option<String> = None;

        if let Some(value_token) = first_value_token {
            statement.push(value_token);
        }

        while let Some(peeked) = tokenizer.peek() {
            match peeked.kind {
                Kind::CloseCurly if !statement.is_open() => {
                    terminator.set_ended_by(Kind::CloseCurly);
                    break;
                }
                Kind::Space if peeked.value == characters::NEWLINE && !statement.is_open() => break,
                Kind::Space
                    if peeked.value.starts_with(characters::NEWLINE) && !statement.is_open() =>
                {
                    if matches!(
                        Self::peek_second_kind(tokenizer),
                        Some(Kind::Comment) | Some(Kind::CloseCurly)
                    ) {
                        break;
                    }
                }
                Kind::Space if !statement.is_open() => {
                    if Self::peek_second_kind(tokenizer) == Some(Kind::CloseCurly) {
                        terminator.set_ended_by(Kind::CloseCurly);
                        break;
                    }
                }
                _ => {}
            }

            let token = tokenizer.next().unwrap();
            terminator.process(&token);

            match token.kind {
                Kind::OpenCurly => statement.open(token),
                Kind::CloseCurly if statement.is_open() => statement.close(token),
                Kind::Semicolon if !statement.is_open() => {
                    terminator.set_ended_by(Kind::Semicolon);
                    break;
                }
                Kind::Comment => {
                    let before_content = statement.clean_is_empty() && !statement.is_open();
                    let next_is_fn = Self::next_is_open_bracket(tokenizer);
                    match (before_content, next_is_fn) {
                        (true, true) => {
                            for space in pending_spaces.drain(..) {
                                raws_between.push_str(space.value);
                            }
                            statement.push(token);
                        }
                        (true, false) => {
                            for space in pending_spaces.drain(..) {
                                statement.push(space);
                            }
                            statement.push_raw_only(token);
                        }
                        (false, true) => statement.push(token),
                        (false, false) => statement.push_raw_only(token),
                    }
                }
                Kind::Space => {
                    if statement.clean_is_empty() && !statement.is_open() {
                        pending_spaces.push(token);
                    } else {
                        statement.push(token);
                    }
                }
                Kind::Word if token.value.contains('!') => {
                    let (result, bang_pos) = Self::check_important(token.value, tokenizer);
                    match result {
                        ImportantResult::Found(raw) => {
                            let trailing_spaces = statement.collect_trailing_spaces();
                            if bang_pos > 0 {
                                statement.push_token_slice(&token, 0, bang_pos);
                            }
                            let mut imp = String::with_capacity(trailing_spaces.len() + raw.len());
                            imp.push_str(&trailing_spaces);
                            imp.push_str(&raw);
                            important_raw = Some(imp);
                        }
                        ImportantResult::NotImportant => {
                            for space in pending_spaces.drain(..) {
                                raws_between.push_str(space.value);
                            }
                            statement.push(token);
                        }
                    }
                }
                _ => {
                    for space in pending_spaces.drain(..) {
                        raws_between.push_str(space.value);
                    }
                    statement.push(token);
                }
            }
        }

        if terminator.ended_by == Some(Kind::CloseCurly) {
            if let Some(space_token) = tokenizer.peek() {
                if space_token.kind == Kind::Space {
                    pending_spaces.push(space_token);
                }
            }
        }
        if !pending_spaces.is_empty() {
            for space in pending_spaces.iter() {
                statement.push(*space);
            }
        }

        let css = tokenizer.source();
        let (raw_start, raw_end) = statement.raw_span();
        let raw_value: &str = &css[raw_start..raw_end];

        let clean_value: Cow<'src, str> = match statement.clean_span() {
            Some((cs, ce)) => Cow::Borrowed(&css[cs..ce]),
            None => {
                let clean = statement.to_string_clean();
                let should_trim =
                    terminator.ended_by == Some(Kind::Semicolon) && !clean.trim().is_empty();
                if should_trim {
                    Cow::Owned(clean.trim_end().to_string())
                } else {
                    Cow::Owned(clean)
                }
            }
        };

        let raws_value_raw: Option<String> = if raw_value != clean_value.as_ref() {
            Some(raw_value.to_string())
        } else {
            None
        };

        let important = important_raw.as_ref().map(|_| true);

        let source_end = Source::create_end(tokenizer);

        // Consume the trailing space after CloseCurly
        if terminator.ended_by == Some(Kind::CloseCurly) {
            if let Some(space_token) = tokenizer.peek() {
                if space_token.kind == Kind::Space {
                    tokenizer.next();
                }
            }
        }

        sink.emit_leaf(
            parent,
            LeafNode::Declaration {
                source_start,
                source_end: source_end.clone(),
                raws_before: Some(raws_before),
                prop: Cow::Borrowed(actual_prop),
                raws_between: Some(raws_between),
                value: clean_value,
                raws_value_raw,
                important,
                raws_important: important_raw,
            },
        );

        (source_end, terminator)
    }

    /// Check if a word token contains !important pattern
    /// Returns (ImportantResult, bang_position)
    fn check_important(word: &str, tokenizer: &mut Tokenizer<'_>) -> (ImportantResult, usize) {
        // Find the position of ! in the word
        let Some(bang_pos) = word.find('!') else {
            return (ImportantResult::NotImportant, 0);
        };

        let after_bang = &word[bang_pos + 1..]; // Skip the !

        // Check if everything after ! matches "important" (case-insensitive)
        if after_bang.eq_ignore_ascii_case("important") {
            let raw = Self::build_important_raw(&word[bang_pos..], tokenizer);
            return (ImportantResult::Found(raw), bang_pos);
        }

        // Not a prefix of "important", bail early
        if !after_bang.is_empty() && !Self::could_be_important_prefix(after_bang) {
            return (ImportantResult::NotImportant, 0);
        }

        // Collect subsequent tokens to build complete important pattern
        Self::collect_multitoken_important(&word[bang_pos..], after_bang, tokenizer, bang_pos)
    }

    /// Collect !important pattern that spans multiple tokens
    fn collect_multitoken_important(
        initial: &str,
        after_bang: &str,
        tokenizer: &mut Tokenizer<'_>,
        bang_pos: usize,
    ) -> (ImportantResult, usize) {
        let mut raw_parts = vec![initial.to_string()];

        // Already complete if after_bang is "important"
        if after_bang.eq_ignore_ascii_case("important") {
            let raw = Self::build_important_raw(&raw_parts.join(""), tokenizer);
            return (ImportantResult::Found(raw), bang_pos);
        }

        let cp = tokenizer.checkpoint();

        loop {
            match tokenizer.next() {
                None => break,
                Some(t) if matches!(t.kind, Kind::Space | Kind::Comment) => {
                    raw_parts.push(t.value.to_string());
                    continue;
                }
                Some(t) if t.kind == Kind::Word && t.value.eq_ignore_ascii_case("important") => {
                    raw_parts.push(t.value.to_string());
                    // Tokens already consumed — no rewind needed
                    let raw = Self::build_important_raw(&raw_parts.join(""), tokenizer);
                    return (ImportantResult::Found(raw), bang_pos);
                }
                _ => break,
            }
        }

        tokenizer.rewind(cp);
        (ImportantResult::NotImportant, 0)
    }

    /// Build the final important raw string, including trailing space if present
    fn build_important_raw(base: &str, tokenizer: &mut Tokenizer<'_>) -> String {
        let mut raw = base.to_string();

        if let Some(trailing) = tokenizer.peek() {
            if trailing.kind == Kind::Space {
                raw.push_str(trailing.value);
                tokenizer.next();
            }
        }

        raw
    }

    /// Check if a string could be a prefix of "important" (case-insensitive, zero-alloc).
    fn could_be_important_prefix(s: &str) -> bool {
        let n = s.len();
        n == 0 || (n <= 9 && "important"[..n].eq_ignore_ascii_case(s))
    }

    /// Return the kind of the token after the next one (skips one token ahead).
    /// Used to look past the current peeked token to detect terminators.
    fn peek_second_kind(tokenizer: &mut Tokenizer<'_>) -> Option<Kind> {
        let cp = tokenizer.checkpoint();
        tokenizer.next(); // skip current token
        let kind = tokenizer.next().map(|t| t.kind);
        tokenizer.rewind(cp);
        kind
    }

    /// Check if the next token is an opening bracket `(`.
    fn next_is_open_bracket(tokenizer: &mut Tokenizer<'_>) -> bool {
        matches!(tokenizer.peek(), Some(t) if t.kind == Kind::Brackets && t.value.starts_with('('))
    }

    /// Extract IE hack prefix (* or _) from property name.
    /// Returns `(actual_property_name, optional_ie_hack_prefix)` as `&str` slices
    /// — zero allocation for the common case of no IE prefix.
    fn extract_ie_hack_prefix(prop: &str) -> (&str, Option<&'static str>) {
        match prop.as_bytes().first() {
            Some(b'*') => (&prop[1..], Some("*")),
            Some(b'_') => (&prop[1..], Some("_")),
            _ => (prop, None),
        }
    }
}

enum ImportantResult {
    Found(String), // Raw important string
    NotImportant,
}
