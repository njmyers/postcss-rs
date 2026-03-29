use std::fmt;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokens;

/// Accumulates tokens that form a single CSS statement value or selector.
///
/// A raw token list (`values`) is always maintained.  A separate clean list
/// (`values_clean_inner`) is only created on demand — the first time a
/// raw-only (comment) token is pushed via [`Statement::push_raw_only`].
/// For the common case of no inline comments this avoids all token clones
/// and the second `Vec` allocation entirely.
///
/// A `BracketDepth` tracker is kept in sync so callers can test whether the
/// current position is inside a nested bracket group ([`Statement::is_open`]).
#[derive(PartialEq, Eq, Debug)]
pub struct Statement<'src> {
    /// Tracks the nesting depth of `{`, `[`, and `(` bracket pairs.
    pub brackets: BracketDepth,
    /// Raw token sequence (includes comments).
    pub values: Tokens<'src>,
    /// Clean token sequence (comments excluded).
    /// `None` until the first [`push_raw_only`] call; when `None` the clean
    /// sequence is identical to `values`.
    values_clean_inner: Option<Tokens<'src>>,
}

impl<'src> Statement<'src> {
    /// Creates a new, empty `Statement`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes an opening bracket token, updating the bracket depth.
    pub fn open(&mut self, token: Token<'src>) {
        self.brackets.process(&token);
        if let Some(clean) = &mut self.values_clean_inner {
            clean.push(token);
        }
        self.values.push(token);
    }

    /// Pushes a closing bracket token, updating the bracket depth.
    pub fn close(&mut self, token: Token<'src>) {
        self.brackets.process(&token);
        if let Some(clean) = &mut self.values_clean_inner {
            clean.push(token);
        }
        self.values.push(token);
    }

    /// Pushes a token to both the raw and clean sequences.
    pub fn push(&mut self, token: Token<'src>) {
        self.brackets.process(&token);
        if let Some(clean) = &mut self.values_clean_inner {
            clean.push(token);
        }
        self.values.push(token);
    }

    /// Pushes a token to the raw sequence only (excluded from the clean copy).
    ///
    /// Used for inline comment tokens that must be preserved in the raw output
    /// but stripped from the semantic value.  This call triggers lazy
    /// initialisation of `values_clean_inner` if it has not been created yet.
    pub fn push_raw_only(&mut self, token: Token<'src>) {
        self.brackets.process(&token);
        // Lazily initialise the clean Vec on first raw-only token.
        if self.values_clean_inner.is_none() {
            let mut clean = Tokens::with_capacity(self.values.len() + 4);
            for &tok in self.values.iter() {
                clean.push(tok);
            }
            self.values_clean_inner = Some(clean);
        }
        self.values.push(token);
    }

    /// Returns `true` if no tokens have been added yet.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns `true` if the clean value is empty.
    ///
    /// When no raw-only tokens have been seen this is the same as `is_empty`.
    #[inline]
    pub fn clean_is_empty(&self) -> bool {
        match &self.values_clean_inner {
            None => self.values.is_empty(),
            Some(clean) => clean.is_empty(),
        }
    }

    /// Returns `true` if the current position is inside an open bracket group.
    pub fn is_open(&self) -> bool {
        self.brackets.is_open()
    }

    /// Returns the comment-stripped value as a `String`.
    pub fn to_string_clean(&self) -> String {
        match &self.values_clean_inner {
            None => self.values.to_string(),
            Some(clean) => clean.to_string(),
        }
    }

    /// Returns the (start, end_exclusive) byte span of the raw token sequence
    /// within the original CSS source.  Since tokens are produced sequentially
    /// by the tokenizer they are always contiguous, so `css[start..end]` equals
    /// `self.to_string()` without any allocation.
    ///
    /// Returns `(0, 0)` when there are no tokens.
    #[inline]
    pub fn raw_span(&self) -> (usize, usize) {
        match (self.values.first(), self.values.last()) {
            (Some(first), Some(last)) => (first.start, last.start + last.value.len()),
            _ => (0, 0),
        }
    }

    /// Returns the (start, end_exclusive) byte span of the clean token sequence.
    /// When no `push_raw_only` has been called this is identical to `raw_span`.
    ///
    /// Returns `None` when a clean string cannot be represented as a single
    /// contiguous CSS span (i.e. comments were stripped leaving gaps).
    #[inline]
    pub fn clean_span(&self) -> Option<(usize, usize)> {
        match &self.values_clean_inner {
            // No comments stripped → clean == raw, same span
            None => Some(self.raw_span()),
            // Comments stripped → gaps exist, cannot use a single span
            Some(_) => None,
        }
    }

    /// Collect and remove trailing space tokens, returning them as a string.
    ///
    /// Avoids intermediate allocations by building the result string directly.
    pub fn collect_trailing_spaces(&mut self) -> String {
        // Find where trailing spaces start (scanning from end)
        let start = self
            .values
            .iter()
            .rposition(|t| t.kind != Kind::Space)
            .map(|i| i + 1)
            .unwrap_or(0);

        let total_len: usize = self.values[start..].iter().map(|t| t.value.len()).sum();
        let mut result = String::with_capacity(total_len);
        for tok in &self.values[start..] {
            result.push_str(tok.value);
        }

        // Truncate values to remove the trailing spaces
        self.values.truncate(start);

        // Also remove from values_clean if it exists
        if let Some(clean) = &mut self.values_clean_inner {
            let cs = clean
                .iter()
                .rposition(|t| t.kind != Kind::Space)
                .map(|i| i + 1)
                .unwrap_or(0);
            clean.truncate(cs);
        }

        result
    }

    /// Push a sliced portion of a token to both values and values_clean
    /// Preserves accurate source positions from the original token
    pub fn push_token_slice(&mut self, token: &Token<'src>, start: usize, end: usize) {
        let sliced = token.slice(start, end);
        if let Some(clean) = &mut self.values_clean_inner {
            clean.push(sliced);
        }
        self.values.push(sliced);
    }
}

impl<'src> Default for Statement<'src> {
    fn default() -> Self {
        Self {
            brackets: BracketDepth::new(),
            values: Tokens::with_capacity(8),
            values_clean_inner: None,
        }
    }
}

impl<'src> fmt::Display for Statement<'src> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.values)
    }
}

/// Records how a statement's token sequence was terminated.
///
/// The parser uses this to communicate to callers whether a declaration or
/// at-rule ended with a semicolon and which token kind actually closed it.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct StatementTerminator {
    /// `true` if a `;` was encountered during parsing.
    pub semicolon: bool,
    /// The [`Kind`] of the token that caused parsing to stop, if known.
    pub ended_by: Option<Kind>,
}

impl StatementTerminator {
    /// Creates a new `StatementTerminator` with no semicolon and no end token.
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates the terminator based on `token`; sets `semicolon` if the token
    /// is a [`Kind::Semicolon`].
    pub fn process(&mut self, token: &Token) {
        self.semicolon = self.semicolon || token.kind == Kind::Semicolon;
    }

    /// Records that parsing was ended by a token of the given `kind`.
    pub fn set_ended_by(&mut self, kind: Kind) {
        self.ended_by = Some(kind);
    }
}

impl Default for StatementTerminator {
    fn default() -> Self {
        Self {
            semicolon: false,
            ended_by: None,
        }
    }
}

/// Tracks the depth of different bracket types
#[derive(PartialEq, Eq, Debug, Default)]
pub struct BracketDepth {
    /// Represents the bracket depth of curly or {} type brackets
    curly: i32,
    /// Represents the bracket depth of square or [] type brackets
    square: i32,
    /// Represents the bracket depth of round or () type brackets
    round: i32,
}

impl BracketDepth {
    /// Creates a new `BracketDepth` with all depths at zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if any bracket type has an unclosed opening.
    pub fn is_open(&self) -> bool {
        self.curly > 0 || self.square > 0 || self.round > 0
    }

    /// Updates bracket depth counts based on `token`'s kind.
    pub fn process(&mut self, token: &Token) {
        match token.kind {
            Kind::OpenCurly => self.curly += 1,
            Kind::CloseCurly => self.curly = (self.curly - 1).max(0),
            Kind::OpenCurvy => self.round += 1,
            Kind::CloseCurvy => self.round = (self.round - 1).max(0),
            Kind::OpenSquare => self.square += 1,
            Kind::CloseSquare => self.square = (self.square - 1).max(0),
            Kind::Brackets => {
                // Brackets token contains both open and close
                // For now, treat as balanced (no net change)
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcss_rs_tokenizer::{Kind, Token};

    fn make_token<'a>(kind: Kind, value: &'a str) -> Token<'a> {
        Token {
            kind,
            value,
            start: 0,
            end: value.len(),
            column: 1,
        }
    }

    // ---- BracketDepth tests ----

    #[test]
    fn test_bracket_depth_starts_closed() {
        let depth = BracketDepth::new();
        assert!(!depth.is_open());
    }

    #[test]
    fn test_bracket_depth_curly() {
        let mut depth = BracketDepth::new();
        depth.process(&make_token(Kind::OpenCurly, "{"));
        assert!(depth.is_open());
        depth.process(&make_token(Kind::CloseCurly, "}"));
        assert!(!depth.is_open());
    }

    #[test]
    fn test_bracket_depth_round() {
        let mut depth = BracketDepth::new();
        depth.process(&make_token(Kind::OpenCurvy, "("));
        assert!(depth.is_open());
        depth.process(&make_token(Kind::CloseCurvy, ")"));
        assert!(!depth.is_open());
    }

    #[test]
    fn test_bracket_depth_square() {
        let mut depth = BracketDepth::new();
        depth.process(&make_token(Kind::OpenSquare, "["));
        assert!(depth.is_open());
        depth.process(&make_token(Kind::CloseSquare, "]"));
        assert!(!depth.is_open());
    }

    #[test]
    fn test_bracket_depth_does_not_go_negative() {
        let mut depth = BracketDepth::new();
        // Extra close without matching open should not underflow
        depth.process(&make_token(Kind::CloseCurly, "}"));
        assert!(!depth.is_open());
    }

    // ---- StatementTerminator tests ----

    #[test]
    fn test_terminator_initial_state() {
        let t = StatementTerminator::new();
        assert!(!t.semicolon);
        assert!(t.ended_by.is_none());
    }

    #[test]
    fn test_terminator_process_semicolon() {
        let mut t = StatementTerminator::new();
        t.process(&make_token(Kind::Semicolon, ";"));
        assert!(t.semicolon);
    }

    #[test]
    fn test_terminator_process_non_semicolon() {
        let mut t = StatementTerminator::new();
        t.process(&make_token(Kind::Word, "color"));
        assert!(!t.semicolon);
    }

    #[test]
    fn test_terminator_set_ended_by() {
        let mut t = StatementTerminator::new();
        t.set_ended_by(Kind::CloseCurly);
        assert_eq!(t.ended_by, Some(Kind::CloseCurly));
    }

    // ---- Statement tests ----

    #[test]
    fn test_statement_starts_empty() {
        let s = Statement::new();
        assert!(s.is_empty());
        assert!(!s.is_open());
    }

    #[test]
    fn test_statement_push_adds_to_both() {
        let mut s = Statement::new();
        s.push(make_token(Kind::Word, "red"));
        assert!(!s.is_empty());
        assert_eq!(s.to_string(), "red");
        assert_eq!(s.to_string_clean(), "red");
    }

    #[test]
    fn test_statement_push_raw_only_excluded_from_clean() {
        let mut s = Statement::new();
        s.push(make_token(Kind::Word, "red"));
        s.push_raw_only(make_token(Kind::Comment, "/* note */"));
        assert_eq!(s.to_string(), "red/* note */");
        assert_eq!(s.to_string_clean(), "red");
    }

    #[test]
    fn test_statement_open_close_tracks_depth() {
        let mut s = Statement::new();
        s.open(make_token(Kind::OpenCurly, "{"));
        assert!(s.is_open());
        s.close(make_token(Kind::CloseCurly, "}"));
        assert!(!s.is_open());
    }

    #[test]
    fn test_statement_collect_trailing_spaces() {
        let mut s = Statement::new();
        s.push(make_token(Kind::Word, "red"));
        s.push(make_token(Kind::Space, " "));
        s.push(make_token(Kind::Space, "  "));
        let spaces = s.collect_trailing_spaces();
        assert_eq!(spaces, "   ");
        // The trailing spaces should be removed from values
        assert_eq!(s.to_string(), "red");
    }

    #[test]
    fn test_clean_is_empty_without_raw_only() {
        let mut s = Statement::new();
        assert!(s.clean_is_empty());
        s.push(make_token(Kind::Word, "red"));
        assert!(!s.clean_is_empty());
    }

    #[test]
    fn test_clean_is_empty_with_raw_only() {
        let mut s = Statement::new();
        s.push_raw_only(make_token(Kind::Comment, "/* note */"));
        // values_clean_inner is now Some(empty vec) since we had no prior tokens
        assert!(s.clean_is_empty());
    }
}
