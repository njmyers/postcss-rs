use std::fmt;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokens;

/// Accumulates tokens that form a single CSS statement value or selector.
///
/// Two parallel token lists are maintained:
/// - `values` — the raw token stream, including inline comments.
/// - `values_clean` — the same stream with comment tokens omitted.
///
/// A `BracketDepth` tracker is kept in sync so callers can test whether the
/// current position is inside a nested bracket group ([`Statement::is_open`]).
#[derive(PartialEq, Eq, Debug)]
pub struct Statement {
    /// Tracks the nesting depth of `{`, `[`, and `(` bracket pairs.
    pub brackets: BracketDepth,
    /// Raw token sequence (includes comments).
    pub values: Tokens,
    /// Clean token sequence (comments excluded via [`Statement::push_raw_only`]).
    pub values_clean: Tokens,
}

impl Statement {
    /// Creates a new, empty `Statement`.
    pub fn new() -> Self {
        return Self::default();
    }

    /// Pushes an opening bracket token, updating the bracket depth.
    pub fn open(&mut self, token: Token) {
        self.brackets.process(&token);
        self.values_clean.push(token.clone());
        self.values.push(token);
    }

    /// Pushes a closing bracket token, updating the bracket depth.
    pub fn close(&mut self, token: Token) {
        self.brackets.process(&token);
        self.values_clean.push(token.clone());
        self.values.push(token);
    }

    /// Pushes a token to both the raw and clean sequences.
    pub fn push(&mut self, token: Token) {
        self.brackets.process(&token);
        self.values_clean.push(token.clone());
        self.values.push(token);
    }

    /// Pushes a token to the raw sequence only (excluded from the clean copy).
    ///
    /// Used for inline comment tokens that must be preserved in the raw output
    /// but stripped from the semantic value.
    pub fn push_raw_only(&mut self, token: Token) {
        self.brackets.process(&token);
        self.values.push(token);
    }

    /// Returns `true` if no tokens have been added yet.
    pub fn is_empty(&self) -> bool {
        return self.values.is_empty();
    }

    /// Returns `true` if the current position is inside an open bracket group.
    pub fn is_open(&self) -> bool {
        return self.brackets.is_open();
    }

    /// Returns the comment-stripped value as a `String`.
    pub fn to_string_clean(&self) -> String {
        return self.values_clean.to_string();
    }

    /// Collect and remove trailing space tokens, returning them as a string
    pub fn collect_trailing_spaces(&mut self) -> String {
        let mut spaces = Vec::new();

        // Collect from values (raw) - in reverse order
        while let Some(last) = self.values.last() {
            if last.kind == Kind::Space {
                spaces.push(self.values.pop().unwrap().value);
            } else {
                break;
            }
        }

        // Also remove from values_clean
        while let Some(last) = self.values_clean.last() {
            if last.kind == Kind::Space {
                self.values_clean.pop();
            } else {
                break;
            }
        }

        // Reverse to get correct order and join
        spaces.reverse();
        spaces.join("")
    }

    /// Push a sliced portion of a token to both values and values_clean
    /// Preserves accurate source positions from the original token
    pub fn push_token_slice(&mut self, token: &Token, start: usize, end: usize) {
        let sliced = token.slice(start, end);
        self.values_clean.push(sliced.clone());
        self.values.push(sliced);
    }
}

impl Default for Statement {
    fn default() -> Self {
        return Self {
            brackets: BracketDepth::new(),
            values: Tokens::with_capacity(8),
            values_clean: Tokens::with_capacity(8),
        };
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        return write!(formatter, "{}", self.values);
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
        return Self::default();
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
        return Self {
            semicolon: false,
            ended_by: None,
        };
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
        return Self::default();
    }

    /// Returns `true` if any bracket type has an unclosed opening.
    pub fn is_open(&self) -> bool {
        return self.curly > 0 || self.square > 0 || self.round > 0;
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

    fn make_token(kind: Kind, value: &str) -> Token {
        Token { kind, value: value.into(), start: 0, end: value.len(), column: 1 }
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
}
