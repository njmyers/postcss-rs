use crate::brackets::*;
use crate::comment::*;
use crate::escape::*;
use crate::line::*;
use crate::string::*;
use crate::symbol::*;
use crate::whitespace::*;
use crate::word::*;

use std::fmt;
use std::ops::Deref;

use itertools::multipeek;
use itertools::MultiPeek;
use serde::{Deserialize, Serialize};

/// The category of a CSS token produced by the [`Tokenizer`].
///
/// Each variant corresponds to a distinct lexical construct in CSS.
#[derive(PartialEq, Eq, Debug, Deserialize, Serialize, Copy, Clone)]
pub enum Kind {
    /// One or more CSS whitespace characters (space, tab, newline, …).
    Space,
    /// An identifier, keyword, or other word-like token.
    Word,
    /// A balanced parenthesised group, e.g. `(content)`.
    Brackets,
    /// A quoted string literal, e.g. `"value"` or `'value'`.
    String,
    /// An at-keyword, e.g. `@media`.
    AtWord,
    /// `{`
    OpenCurly,
    /// `}`
    CloseCurly,
    /// `[`
    OpenSquare,
    /// `]`
    CloseSquare,
    /// `(`
    OpenCurvy,
    /// `)`
    CloseCurvy,
    /// `;`
    Semicolon,
    /// `:`
    Colon,
    /// A block comment `/* … */`.
    Comment,
}

impl fmt::Display for Kind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Kind::Space => write!(formatter, "Space"),
            Kind::Word => write!(formatter, "Word"),
            Kind::Brackets => write!(formatter, "Brackets"),
            Kind::String => write!(formatter, "String"),
            Kind::AtWord => write!(formatter, "AtWord"),
            Kind::OpenCurly => write!(formatter, "OpenCurly"),
            Kind::CloseCurly => write!(formatter, "CloseCurly"),
            Kind::OpenSquare => write!(formatter, "OpenSquare"),
            Kind::CloseSquare => write!(formatter, "CloseSquare"),
            Kind::OpenCurvy => write!(formatter, "OpenCurvy"),
            Kind::CloseCurvy => write!(formatter, "CloseCurvy"),
            Kind::Semicolon => write!(formatter, "Semicolon"),
            Kind::Colon => write!(formatter, "Colon"),
            Kind::Comment => write!(formatter, "Comment"),
        }
    }
}

/// A single CSS token produced by the [`Tokenizer`].
#[derive(PartialEq, Eq, Debug, Serialize, Clone)]
pub struct Token<'src> {
    /// Byte offset of the first character of this token in the source.
    pub start: usize,
    /// Byte offset one past the last character of this token.
    pub end: usize,
    /// The lexical category of this token.
    pub kind: Kind,
    /// The raw source text of this token.
    pub value: &'src str,
    /// 1-based column of the first character of this token on its source line.
    pub column: usize,
}


impl<'src> fmt::Display for Token<'src> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{{ kind: {}, start: {}, end: {}, value: {}, column: {} }}",
            self.kind, self.start, self.end, self.value, self.column
        )
    }
}

impl<'src> Token<'src> {
    /// Create a new token that represents a substring of this token
    /// Preserves accurate source positions from the original token
    pub fn slice(&self, start_offset: usize, end_offset: usize) -> Token<'src> {
        Token {
            kind: self.kind,
            value: &self.value[start_offset..end_offset],
            start: self.start + start_offset,
            end: self.start + end_offset,
            column: self.column + start_offset,
        }
    }

    /// Split a token at the given byte index into two tokens with specified kinds
    ///
    /// # Arguments
    /// * `split_index` - The byte index at which to split the token's value
    /// * `first_kind` - The kind for the first token (left part)
    /// * `second_kind` - The kind for the second token (right part)
    ///
    /// # Returns
    /// A tuple of (first_token, Option<second_token>)
    /// The second token is None if split_index >= value length
    ///
    /// # Example
    /// ```
    /// use postcss_rs_tokenizer::{Token, Kind};
    ///
    /// let token = Token {
    ///     kind: Kind::Word,
    ///     value: ":black",
    ///     start: 5,
    ///     end: 11,
    ///     column: 6,
    /// };
    ///
    /// let (colon, value) = token.split(1, Kind::Colon, Kind::Word);
    ///
    /// assert_eq!(colon.kind, Kind::Colon);
    /// assert_eq!(colon.value, ":");
    /// assert_eq!(colon.start, 5);
    /// assert_eq!(colon.end, 6);
    ///
    /// assert!(value.is_some());
    /// let value_token = value.unwrap();
    /// assert_eq!(value_token.kind, Kind::Word);
    /// assert_eq!(value_token.value, "black");
    /// assert_eq!(value_token.start, 6);
    /// assert_eq!(value_token.end, 11);
    /// ```
    pub fn split(&self, split_index: usize, first_kind: Kind, second_kind: Kind) -> (Token<'src>, Option<Token<'src>>) {
        let first_token = Token {
            kind: first_kind,
            value: &self.value[..split_index],
            start: self.start,
            end: self.start + split_index,
            column: self.column,
        };

        let second_token = match self.value.len() {
            len if len > split_index => Some(Token {
                kind: second_kind,
                value: &self.value[split_index..],
                start: self.start + split_index,
                end: self.end,
                column: self.column + split_index,
            }),
            _ => None,
        };

        (first_token, second_token)
    }
}

/// An ordered collection of [`Token`]s.
///
/// Provides a thin wrapper around `Vec<Token>` with helper methods used
/// throughout the parser to accumulate token sequences.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Tokens<'src>(Vec<Token<'src>>);

impl<'src> Tokens<'src> {
    /// Creates an empty token collection.
    pub fn new() -> Self {
        return Self(Vec::new());
    }

    /// Creates an empty token collection pre-allocated for at least `capacity` tokens.
    pub fn with_capacity(capacity: usize) -> Self {
        return Self(Vec::with_capacity(capacity));
    }

    /// Appends `token` to the end of this collection.
    pub fn push(&mut self, token: Token<'src>) {
        self.0.push(token);
    }

    /// Removes all tokens from this collection.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Removes and returns the last token, or `None` if the collection is empty.
    pub fn pop(&mut self) -> Option<Token<'src>> {
        self.0.pop()
    }

    /// Returns a reference to the last token, or `None` if the collection is empty.
    pub fn last(&self) -> Option<&Token<'src>> {
        self.0.last()
    }
}

impl<'src> fmt::Display for Tokens<'src> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        for token in self.0.iter() {
            write!(formatter, "{}", token.value)?;
        }

        return Ok(());
    }
}

impl<'src> Default for Tokens<'src> {
    fn default() -> Self {
        return Self::new();
    }
}

impl<'src> Deref for Tokens<'src> {
    type Target = Vec<Token<'src>>;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

/// A saved position in a [`Tokenizer`] that can be restored with [`Tokenizer::rewind`].
///
/// Created by [`Tokenizer::checkpoint`]; used to implement lookahead without cloning.
#[derive(Debug)]
pub struct Checkpoint<'src> {
    index: usize,
    prev_value: Option<&'src str>,
    lines_len: usize,
}

/// Converts a CSS source string into a stream of [`Token`]s.
///
/// `Tokenizer` implements [`Iterator`]`<Item = Token>`, so tokens can be
/// consumed one at a time.  For lookahead, use [`Tokenizer::checkpoint`] /
/// [`Tokenizer::rewind`] / [`Tokenizer::peek`] instead of cloning.
///
/// # Example
/// ```
/// use postcss_rs_tokenizer::{Tokenizer, Kind};
///
/// let mut tokenizer = Tokenizer::new("a { color: red }");
/// let first = tokenizer.next().unwrap();
/// assert_eq!(first.kind, Kind::Word);
/// assert_eq!(first.value, "a");
/// ```
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Tokenizer<'src> {
    css: &'src str,
    /// Value of the most recently produced token, used for `url(` detection.
    prev_value: Option<&'src str>,
    /// Current byte index into the source (advances as tokens are produced).
    pub index: usize,
    /// Line-tracking state; updated whenever a newline is encountered.
    pub lines: Lines,
}

impl<'src> Tokenizer<'src> {
    /// Creates a new `Tokenizer` for the given CSS source string.
    pub fn new(css: &'src str) -> Self {
        return Self {
            css,
            index: 0,
            prev_value: None,
            lines: Lines::new(),
        };
    }

    /// Returns the value of the most recently produced token.
    pub fn prev(&self) -> Option<&str> {
        self.prev_value
    }

    /// Convert this tokenizer into a PeekableTokenizer for simple next-token peeking.
    /// This is a convenience method equivalent to `PeekableTokenizer::new(tokenizer)`.
    ///
    /// # Example
    /// ```
    /// use postcss_rs_tokenizer::Tokenizer;
    ///
    /// let mut peekable = Tokenizer::new("a b c").into_peekable();
    /// assert_eq!(peekable.peek().map(|t| t.value), Some("a"));
    /// ```
    pub fn into_peekable(self) -> PeekableTokenizer<'src> {
        PeekableTokenizer::new(self)
    }

    /// Convert this tokenizer into a MultiPeekTokenizer for multi-token lookahead.
    /// This is a convenience method equivalent to `MultiPeekTokenizer::new(tokenizer)`.
    ///
    /// # Example
    /// ```
    /// use postcss_rs_tokenizer::Tokenizer;
    ///
    /// let mut multi = Tokenizer::new("a b c").into_multipeek();
    /// assert_eq!(multi.peek().map(|t| t.value), Some("a"));
    /// assert_eq!(multi.peek().map(|t| t.value), Some(" "));
    /// ```
    pub fn into_multipeek(self) -> MultiPeekTokenizer<'src> {
        MultiPeekTokenizer::new(self)
    }

    /// Save the current tokenizer position so it can be restored with [`rewind`].
    ///
    /// Use `checkpoint` / `rewind` / `peek` instead of `clone` for lookahead —
    /// the checkpoint saves only `O(1)` state rather than copying the full line
    /// table.
    ///
    /// [`rewind`]: Tokenizer::rewind
    pub fn checkpoint(&self) -> Checkpoint<'src> {
        Checkpoint {
            index: self.index,
            prev_value: self.prev_value,
            lines_len: self.lines.len(),
        }
    }

    /// Restore the tokenizer to a position saved by [`checkpoint`].
    ///
    /// [`checkpoint`]: Tokenizer::checkpoint
    pub fn rewind(&mut self, cp: Checkpoint<'src>) {
        self.index = cp.index;
        self.prev_value = cp.prev_value;
        self.lines.truncate(cp.lines_len);
    }

    /// Return the next token without consuming it.
    ///
    /// Equivalent to saving a checkpoint, calling `next`, and rewinding.
    pub fn peek(&mut self) -> Option<Token<'src>> {
        let cp = self.checkpoint();
        let token = self.next();
        self.rewind(cp);
        token
    }

    fn create(&mut self, length: usize, kind: Kind) -> Token<'src> {
        let column = self.lines.offset(self.index);
        let start = self.index;
        let end = start + length;
        let index = end + 1;
        let value = &self.css[start..index];

        for (i, &b) in value.as_bytes().iter().enumerate() {
            if Newline::test(b) {
                self.lines.increment(start + i)
            }
        }

        self.index = index;

        self.prev_value = Some(value);

        return Token {
            start,
            end,
            kind,
            value,
            column,
        };
    }
}

impl<'src> Iterator for Tokenizer<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.css.as_bytes();
        let remaining = &bytes[self.index..];
        let byte = *remaining.first()?;
        let next = remaining.get(1).copied().unwrap_or(0);

        match byte {
            b if Whitespace::test(b) => {
                let length = Whitespace::tokenize(remaining);
                let token = self.create(length, Kind::Space);

                return Some(token);
            }

            b if AtWord::test(b, next) => {
                let length = AtWord::tokenize(remaining);
                let token = self.create(length, Kind::AtWord);

                return Some(token);
            }

            b if Word::test(b, next) => {
                let length = Word::tokenize(remaining);
                let token = self.create(length, Kind::Word);

                return Some(token);
            }

            b if Bracket::test(b, next, self.prev()) => {
                let length = Bracket::tokenize(remaining);
                let token = self.create(length, Kind::Brackets);

                return Some(token);
            }

            b if OpenCurly::test(b) => {
                let length = OpenCurly::tokenize(remaining);
                let token = self.create(length, Kind::OpenCurly);

                return Some(token);
            }

            b if CloseCurly::test(b) => {
                let length = CloseCurly::tokenize(remaining);
                let token = self.create(length, Kind::CloseCurly);

                return Some(token);
            }

            b if OpenSquare::test(b) => {
                let length = OpenSquare::tokenize(remaining);
                let token = self.create(length, Kind::OpenSquare);

                return Some(token);
            }

            b if CloseSquare::test(b) => {
                let length = CloseSquare::tokenize(remaining);
                let token = self.create(length, Kind::CloseSquare);

                return Some(token);
            }

            b if OpenCurvy::test(b) => {
                let length = OpenCurvy::tokenize(remaining);
                let token = self.create(length, Kind::OpenCurvy);

                return Some(token);
            }

            b if CloseCurvy::test(b) => {
                let length = CloseCurvy::tokenize(remaining);
                let token = self.create(length, Kind::CloseCurvy);

                return Some(token);
            }

            b if Semicolon::test(b) => {
                let length = Semicolon::tokenize(remaining);
                let token = self.create(length, Kind::Semicolon);

                return Some(token);
            }

            b if Colon::test(b) => {
                let length = Colon::tokenize(remaining);
                let token = self.create(length, Kind::Colon);

                return Some(token);
            }

            b if Comment::test(b, next) => {
                let length = Comment::tokenize(&remaining[2..]);
                let token = self.create(length + 3, Kind::Comment);

                return Some(token);
            }

            b if Escape::test(b) => {
                let length = Escape::tokenize(remaining);
                let token = self.create(length, Kind::Word);

                return Some(token);
            }

            b if SingleString::test(b) => {
                let length = SingleString::tokenize(remaining);
                let token = self.create(length, Kind::String);

                return Some(token);
            }

            b if DoubleString::test(b) => {
                let length = DoubleString::tokenize(remaining);
                let token = self.create(length, Kind::String);

                return Some(token);
            }

            _ => {
                let length = Word::tokenize(remaining);
                let token = self.create(length, Kind::Word);

                return Some(token);
            }
        }
    }
}

/// A simple peekable tokenizer using std::iter::Peekable.
///
/// Use this when you only need to peek at the next token (most common case).
/// For multi-token lookahead, use `MultiPeekTokenizer` instead.
///
/// This avoids cloning the entire tokenizer state by using Rust's standard
/// Peekable iterator adapter.
///
/// # Example
/// ```
/// use postcss_rs_tokenizer::{Tokenizer, PeekableTokenizer};
///
/// let tokenizer = Tokenizer::new("a { color: red; }");
/// let mut peekable = PeekableTokenizer::new(tokenizer);
///
/// // Peek at the next token without consuming it
/// assert_eq!(peekable.peek().map(|t| t.value), Some("a"));
/// // Peek again returns the same token
/// assert_eq!(peekable.peek().map(|t| t.value), Some("a"));
///
/// // Now consume it
/// assert_eq!(peekable.next().map(|t| t.value), Some("a"));
/// // Next token is a space
/// assert_eq!(peekable.peek().map(|t| t.value), Some(" "));
/// ```
pub struct PeekableTokenizer<'src> {
    inner: std::iter::Peekable<Tokenizer<'src>>,
}

impl<'src> PeekableTokenizer<'src> {
    /// Create a new PeekableTokenizer from a Tokenizer
    pub fn new(tokenizer: Tokenizer<'src>) -> Self {
        Self {
            inner: tokenizer.peekable(),
        }
    }

    /// Peek at the next token without consuming it.
    /// Returns a reference to the next token, or None if the iterator is exhausted.
    pub fn peek(&mut self) -> Option<&Token<'src>> {
        self.inner.peek()
    }
}

impl<'src> Iterator for PeekableTokenizer<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// A multi-peek tokenizer using itertools' MultiPeek for lookahead parsing.
///
/// Use this when you need to peek multiple tokens ahead (e.g., scanning for
/// terminators, complex lookahead logic). For simple next-token peeking,
/// use `PeekableTokenizer` instead.
///
/// This maintains a small buffer of peeked items instead of cloning the
/// entire iterator state, making it much more efficient than the old
/// clone-based peek() approach.
///
/// # Example
/// ```
/// use postcss_rs_tokenizer::{Tokenizer, MultiPeekTokenizer};
///
/// let tokenizer = Tokenizer::new("a : { color: red; }");
/// let mut peekable = MultiPeekTokenizer::new(tokenizer);
///
/// // Peek ahead multiple positions
/// let first = peekable.peek();   // "a"
/// let second = peekable.peek();  // " "
/// let third = peekable.peek();   // ":"
/// let fourth = peekable.peek();  // " "
/// let fifth = peekable.peek();   // "{"
///
/// // Reset and consume normally
/// peekable.reset_peek();
/// assert_eq!(peekable.next().unwrap().value, "a");
/// ```
pub struct MultiPeekTokenizer<'src> {
    inner: MultiPeek<Tokenizer<'src>>,
}

impl<'src> MultiPeekTokenizer<'src> {
    /// Create a new MultiPeekTokenizer from a Tokenizer
    pub fn new(tokenizer: Tokenizer<'src>) -> Self {
        Self {
            inner: multipeek(tokenizer),
        }
    }

    /// Peek at the next token without consuming it.
    /// Can be called multiple times to peek further ahead.
    /// Call reset_peek() to reset the peek position.
    pub fn peek(&mut self) -> Option<&Token<'src>> {
        self.inner.peek()
    }

    /// Reset the peek position back to the current position.
    /// This should be called after you're done peeking and before
    /// you start consuming tokens or peeking again from the start.
    pub fn reset_peek(&mut self) {
        self.inner.reset_peek();
    }
}

impl<'src> Iterator for MultiPeekTokenizer<'src> {
    type Item = Token<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    fn tokenizer(css: &str) -> Vec<Token<'_>> {
        let mut iterator = Tokenizer::new(css);
        let mut tokens: Vec<Token<'_>> = vec![];

        while let Some(token) = iterator.next() {
            tokens.push(token)
        }

        return tokens;
    }

    #[test]
    fn it_tokenizes_empty_file() {
        let empty: Vec<Token> = vec![];
        assert_eq!(tokenizer(""), empty)
    }

    #[test]
    fn it_tokenizes_space() {
        assert_eq!(
            tokenizer("\r\n \t"),
            [Token {
                kind: Kind::Space,
                start: 0,
                end: 3,
                value: "\r\n \t",
                column: 1,
            }],
        )
    }

    #[test]
    fn it_tokenizes_word() {
        assert_eq!(
            tokenizer("ab"),
            [Token {
                kind: Kind::Word,
                start: 0,
                end: 1,
                value: "ab",
                column: 1,
            }]
        );
    }

    #[test]
    fn it_splits_word_by_exclamation_mark() {
        assert_eq!(
            tokenizer("aa!bb"),
            [
                Token {
                    kind: Kind::Word,
                    start: 0,
                    end: 1,
                    value: "aa",
                    column: 1,
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 4,
                    value: "!bb",
                    column: 3,
                }
            ]
        )
    }

    #[test]
    fn it_changes_lines_in_spaces() {
        assert_eq!(
            tokenizer("a \n b"),
            [
                Token {
                    kind: Kind::Word,
                    start: 0,
                    end: 0,
                    value: "a",
                    column: 1,
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 3,
                    value: " \n ",
                    column: 2,
                },
                Token {
                    kind: Kind::Word,
                    start: 4,
                    end: 4,
                    value: "b",
                    column: 2,
                }
            ]
        )
    }

    #[test]
    fn it_tokenizes_control_characters() {
        assert_eq!(
            tokenizer("{:;}"),
            [
                Token {
                    kind: Kind::OpenCurly,
                    start: 0,
                    end: 0,
                    value: "{",
                    column: 1,
                },
                Token {
                    kind: Kind::Colon,
                    start: 1,
                    end: 1,
                    value: ":",
                    column: 2,
                },
                Token {
                    kind: Kind::Semicolon,
                    start: 2,
                    end: 2,
                    value: ";",
                    column: 3,
                },
                Token {
                    kind: Kind::CloseCurly,
                    start: 3,
                    end: 3,
                    value: "}",
                    column: 4,
                }
            ]
        )
    }

    #[test]
    fn it_escapes_control_symbols() {
        assert_eq!(
            tokenizer("\\(\\{\\\"\\@\\\\\"\""),
            [
                Token {
                    kind: Kind::Word,
                    start: 0,
                    end: 1,
                    value: "\\(",
                    column: 1,
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 3,
                    value: "\\{",
                    column: 3,
                },
                Token {
                    kind: Kind::Word,
                    start: 4,
                    end: 5,
                    value: "\\\"",
                    column: 5,
                },
                Token {
                    kind: Kind::Word,
                    start: 6,
                    end: 7,
                    value: "\\@",
                    column: 7,
                },
                Token {
                    kind: Kind::Word,
                    start: 8,
                    end: 9,
                    value: "\\\\",
                    column: 9,
                },
                Token {
                    kind: Kind::String,
                    start: 10,
                    end: 11,
                    value: "\"\"",
                    column: 11,
                }
            ]
        )
    }

    #[test]
    fn it_escapes_backslash() {
        assert_eq!(
            tokenizer("\\\\\\\\{"),
            [
                Token {
                    kind: Kind::Word,
                    start: 0,
                    end: 3,
                    value: "\\\\\\\\",
                    column: 1,
                },
                Token {
                    kind: Kind::OpenCurly,
                    start: 4,
                    end: 4,
                    value: "{",
                    column: 5,
                }
            ]
        )
    }

    #[test]
    fn it_tokenizes_simple_brackets() {
        assert_eq!(
            tokenizer("(ab)"),
            [Token {
                kind: Kind::Brackets,
                start: 0,
                end: 3,
                value: "(ab)",
                column: 1,
            }]
        )
    }

    #[test]
    fn it_tokenizes_square_brackets() {
        assert_eq!(
            tokenizer("a[bc]"),
            [
                Token {
                    kind: Kind::Word,
                    start: 0,
                    end: 0,
                    value: "a",
                    column: 1,
                },
                Token {
                    kind: Kind::OpenSquare,
                    start: 1,
                    end: 1,
                    value: "[",
                    column: 2,
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 3,
                    value: "bc",
                    column: 3,
                },
                Token {
                    kind: Kind::CloseSquare,
                    start: 4,
                    end: 4,
                    value: "]",
                    column: 5,
                }
            ]
        );
    }

    #[test]

    fn it_tokenizes_complicated_brackets() {
        assert_eq!(
            tokenizer("(())(\"\")(/**/)(\\\\)(\n)("),
            [
                Token {
                    kind: Kind::OpenCurvy,
                    start: 0,
                    end: 0,
                    value: "(",
                    column: 1,
                },
                Token {
                    kind: Kind::Brackets,
                    start: 1,
                    end: 2,
                    value: "()",
                    column: 2,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 3,
                    end: 3,
                    value: ")",
                    column: 4,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 4,
                    end: 4,
                    value: "(",
                    column: 5,
                },
                Token {
                    kind: Kind::String,
                    start: 5,
                    end: 6,
                    value: "\"\"",
                    column: 6,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 7,
                    end: 7,
                    value: ")",
                    column: 8,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 8,
                    end: 8,
                    value: "(",
                    column: 9,
                },
                Token {
                    kind: Kind::Comment,
                    start: 9,
                    end: 12,
                    value: "/**/",
                    column: 10,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 13,
                    end: 13,
                    value: ")",
                    column: 14,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 14,
                    end: 14,
                    value: "(",
                    column: 15,
                },
                Token {
                    kind: Kind::Word,
                    start: 15,
                    end: 16,
                    value: "\\\\",
                    column: 16,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 17,
                    end: 17,
                    value: ")",
                    column: 18,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 18,
                    end: 18,
                    value: "(",
                    column: 19,
                },
                Token {
                    kind: Kind::Space,
                    start: 19,
                    end: 19,
                    value: "\n",
                    column: 20,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 20,
                    end: 20,
                    value: ")",
                    column: 1,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 21,
                    end: 21,
                    value: "(",
                    column: 2,
                },
            ]
        )
    }

    #[test]
    fn it_tokenizes_string() {
        assert_eq!(
            tokenizer("'\"'\"\\\"\""),
            [
                Token {
                    kind: Kind::String,
                    start: 0,
                    end: 2,
                    value: "'\"'",
                    column: 1,
                },
                Token {
                    kind: Kind::String,
                    start: 3,
                    end: 6,
                    value: "\"\\\"\"",
                    column: 4,
                }
            ]
        )
    }

    #[test]
    fn it_tokenizes_escaped_string() {
        assert_eq!(
            tokenizer("\"\\\\\""),
            [Token {
                kind: Kind::String,
                start: 0,
                end: 3,
                value: "\"\\\\\"",
                column: 1,
            }]
        )
    }

    #[test]
    fn it_changes_lines_in_strings() {
        assert_eq!(
            tokenizer("\"\n\n\"\"\n\n\""),
            [
                Token {
                    kind: Kind::String,
                    start: 0,
                    end: 3,
                    value: "\"\n\n\"",
                    column: 1,
                },
                Token {
                    kind: Kind::String,
                    start: 4,
                    end: 7,
                    value: "\"\n\n\"",
                    column: 2,
                }
            ]
        )
    }

    #[test]
    fn it_tokenizes_at_word() {
        assert_eq!(
            tokenizer("@word "),
            [
                Token {
                    kind: Kind::AtWord,
                    start: 0,
                    end: 4,
                    value: "@word",
                    column: 1,
                },
                Token {
                    kind: Kind::Space,
                    start: 5,
                    end: 5,
                    value: " ",
                    column: 6,
                }
            ]
        )
    }

    #[test]
    fn it_tokenizes_at_word_end() {
        assert_eq!(
            tokenizer("@one{@two()@three\"\"@four;"),
            [
                Token {
                    kind: Kind::AtWord,
                    start: 0,
                    end: 3,
                    value: "@one",
                    column: 1,
                },
                Token {
                    kind: Kind::OpenCurly,
                    start: 4,
                    end: 4,
                    value: "{",
                    column: 5,
                },
                Token {
                    kind: Kind::AtWord,
                    start: 5,
                    end: 8,
                    value: "@two",
                    column: 6,
                },
                Token {
                    kind: Kind::Brackets,
                    start: 9,
                    end: 10,
                    value: "()",
                    column: 10,
                },
                Token {
                    kind: Kind::AtWord,
                    start: 11,
                    end: 16,
                    value: "@three",
                    column: 12,
                },
                Token {
                    kind: Kind::String,
                    start: 17,
                    end: 18,
                    value: "\"\"",
                    column: 18,
                },
                Token {
                    kind: Kind::AtWord,
                    start: 19,
                    end: 23,
                    value: "@four",
                    column: 20,
                },
                Token {
                    kind: Kind::Semicolon,
                    start: 24,
                    end: 24,
                    value: ";",
                    column: 25,
                }
            ]
        )
    }

    #[test]
    fn it_tokenizes_urls() {
        assert_eq!(
            tokenizer("url(/*\\))"),
            [
                Token {
                    kind: Kind::Word,
                    start: 0,
                    end: 2,
                    value: "url",
                    column: 1,
                },
                Token {
                    kind: Kind::Brackets,
                    start: 3,
                    end: 8,
                    value: "(/*\\))",
                    column: 4,
                }
            ]
        )
    }

    #[test]

    fn it_tokenizes_quoted_urls() {
        assert_eq!(
            tokenizer("url(\")\")"),
            [
                Token {
                    kind: Kind::Word,
                    start: 0,
                    end: 2,
                    value: "url",
                    column: 1,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 3,
                    end: 3,
                    value: "(",
                    column: 4,
                },
                Token {
                    kind: Kind::String,
                    start: 4,
                    end: 6,
                    value: "\")\"",
                    column: 5,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 7,
                    end: 7,
                    value: ")",
                    column: 8,
                }
            ]
        )
    }

    #[test]
    fn it_tokenizes_at_symbol() {
        assert_eq!(
            tokenizer("@"),
            [Token {
                kind: Kind::AtWord,
                start: 0,
                end: 0,
                value: "@",
                column: 1,
            }]
        )
    }

    #[test]
    fn it_tokenizes_comment() {
        assert_eq!(
            tokenizer("/* a\nb */"),
            [Token {
                kind: Kind::Comment,
                start: 0,
                end: 8,
                value: "/* a\nb */",
                column: 1,
            }]
        )
    }

    #[test]
    fn it_changes_lines_in_comments() {
        assert_eq!(
            tokenizer("a/* \n */b"),
            [
                Token {
                    kind: Kind::Word,
                    start: 0,
                    end: 0,
                    value: "a",
                    column: 1,
                },
                Token {
                    kind: Kind::Comment,
                    start: 1,
                    end: 7,
                    value: "/* \n */",
                    column: 2,
                },
                Token {
                    kind: Kind::Word,
                    start: 8,
                    end: 8,
                    value: "b",
                    column: 4,
                },
            ]
        )
    }

    #[test]
    fn it_supports_form_feed() {
        assert_eq!(
            tokenizer("a\u{000C}b"),
            [
                Token {
                    kind: Kind::Word,
                    start: 0,
                    end: 0,
                    value: "a",
                    column: 1,
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 1,
                    value: "\u{000C}",
                    column: 2,
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 2,
                    value: "b",
                    column: 3,
                }
            ]
        )
    }

    #[test]
    fn it_supports_carriage_return() {
        assert_eq!(
            tokenizer("a\rb\r\nc"),
            [
                Token {
                    kind: Kind::Word,
                    start: 0,
                    end: 0,
                    value: "a",
                    column: 1,
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 1,
                    value: "\r",
                    column: 2,
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 2,
                    value: "b",
                    column: 1,
                },
                Token {
                    kind: Kind::Space,
                    start: 3,
                    end: 4,
                    value: "\r\n",
                    column: 2,
                },
                Token {
                    kind: Kind::Word,
                    start: 5,
                    end: 5,
                    value: "c",
                    column: 1,
                },
            ]
        )
    }

    #[test]
    fn it_tokenizes_css() {
        assert_eq!(
            tokenizer(indoc! {"
                a {
                  content: \"a\";
                  width: calc(1px;)
                  }
                /* small screen */
                @media screen {}"
            }),
            [
                Token {
                    start: 0,
                    end: 0,
                    kind: Kind::Word,
                    value: "a",
                    column: 1
                },
                Token {
                    start: 1,
                    end: 1,
                    kind: Kind::Space,
                    value: " ",
                    column: 2
                },
                Token {
                    start: 2,
                    end: 2,
                    kind: Kind::OpenCurly,
                    value: "{",
                    column: 3
                },
                Token {
                    start: 3,
                    end: 5,
                    kind: Kind::Space,
                    value: "\n  ",
                    column: 4
                },
                Token {
                    start: 6,
                    end: 12,
                    kind: Kind::Word,
                    value: "content",
                    column: 3
                },
                Token {
                    start: 13,
                    end: 13,
                    kind: Kind::Colon,
                    value: ":",
                    column: 10
                },
                Token {
                    start: 14,
                    end: 14,
                    kind: Kind::Space,
                    value: " ",
                    column: 11
                },
                Token {
                    start: 15,
                    end: 17,
                    kind: Kind::String,
                    value: "\"a\"",
                    column: 12
                },
                Token {
                    start: 18,
                    end: 18,
                    kind: Kind::Semicolon,
                    value: ";",
                    column: 15
                },
                Token {
                    start: 19,
                    end: 21,
                    kind: Kind::Space,
                    value: "\n  ",
                    column: 16
                },
                Token {
                    start: 22,
                    end: 26,
                    kind: Kind::Word,
                    value: "width",
                    column: 3
                },
                Token {
                    start: 27,
                    end: 27,
                    kind: Kind::Colon,
                    value: ":",
                    column: 8
                },
                Token {
                    start: 28,
                    end: 28,
                    kind: Kind::Space,
                    value: " ",
                    column: 9
                },
                Token {
                    start: 29,
                    end: 32,
                    kind: Kind::Word,
                    value: "calc",
                    column: 10
                },
                Token {
                    start: 33,
                    end: 38,
                    kind: Kind::Brackets,
                    value: "(1px;)",
                    column: 14
                },
                Token {
                    start: 39,
                    end: 41,
                    kind: Kind::Space,
                    value: "\n  ",
                    column: 20
                },
                Token {
                    start: 42,
                    end: 42,
                    kind: Kind::CloseCurly,
                    value: "}",
                    column: 3
                },
                Token {
                    start: 43,
                    end: 43,
                    kind: Kind::Space,
                    value: "\n",
                    column: 4
                },
                Token {
                    start: 44,
                    end: 61,
                    kind: Kind::Comment,
                    value: "/* small screen */",
                    column: 1
                },
                Token {
                    start: 62,
                    end: 62,
                    kind: Kind::Space,
                    value: "\n",
                    column: 19
                },
                Token {
                    start: 63,
                    end: 68,
                    kind: Kind::AtWord,
                    value: "@media",
                    column: 1
                },
                Token {
                    start: 69,
                    end: 69,
                    kind: Kind::Space,
                    value: " ",
                    column: 7
                },
                Token {
                    start: 70,
                    end: 75,
                    kind: Kind::Word,
                    value: "screen",
                    column: 8
                },
                Token {
                    start: 76,
                    end: 76,
                    kind: Kind::Space,
                    value: " ",
                    column: 14
                },
                Token {
                    start: 77,
                    end: 77,
                    kind: Kind::OpenCurly,
                    value: "{",
                    column: 15
                },
                Token {
                    start: 78,
                    end: 78,
                    kind: Kind::CloseCurly,
                    value: "}",
                    column: 16
                }
            ]
        )
    }

    #[test]
    fn debug_escape_attribute_selector_tokens() {
        let tokens = tokenizer(r#"[foo=\"] { }"#);
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_tokenizer_into_peekable() {
        // Test the convenience method
        let mut peekable = Tokenizer::new("a b c").into_peekable();

        assert_eq!(peekable.peek().map(|t| t.value.as_ref()), Some("a"));
        assert_eq!(peekable.next().map(|t| t.value.to_string()), Some("a".to_string()));
    }

    #[test]
    fn test_tokenizer_into_multipeek() {
        // Test the convenience method
        let mut multi = Tokenizer::new("a b c").into_multipeek();

        // Peek ahead multiple times
        assert_eq!(multi.peek().map(|t| t.value.as_ref()), Some("a"));
        assert_eq!(multi.peek().map(|t| t.value.as_ref()), Some(" "));
        assert_eq!(multi.peek().map(|t| t.value.as_ref()), Some("b"));

        multi.reset_peek();
        assert_eq!(multi.next().map(|t| t.value.to_string()), Some("a".to_string()));
    }

    #[test]
    fn test_peekable_tokenizer_basic() {
        let tokenizer = Tokenizer::new("a { color: red; }");
        let mut peekable = PeekableTokenizer::new(tokenizer);

        // Peek at next token without consuming
        assert_eq!(peekable.peek().map(|t| t.value.as_ref()), Some("a"));

        // Peek again should return the same token (not advance)
        assert_eq!(peekable.peek().map(|t| t.value.as_ref()), Some("a"));

        // Now consume the token
        assert_eq!(peekable.next().map(|t| t.value.to_string()), Some("a".to_string()));

        // Peek at the next token (space)
        assert_eq!(peekable.peek().map(|t| t.value.as_ref()), Some(" "));
    }

    #[test]
    fn test_multi_peek_tokenizer_basic() {
        let tokenizer = Tokenizer::new("a { color: red; }");
        let mut peekable = MultiPeekTokenizer::new(tokenizer);

        // Peek at first token without consuming
        assert_eq!(peekable.peek().map(|t| t.value.as_ref()), Some("a"));

        // Peek again advances to next token
        assert_eq!(peekable.peek().map(|t| t.value.as_ref()), Some(" "));

        // Reset and peek from start again
        peekable.reset_peek();
        assert_eq!(peekable.peek().map(|t| t.value.as_ref()), Some("a"));

        // Now consume the token
        peekable.reset_peek();
        assert_eq!(peekable.next().map(|t| t.value.to_string()), Some("a".to_string()));
    }

    #[test]
    fn test_multi_peek_tokenizer_lookahead() {
        let tokenizer = Tokenizer::new("a b c");
        let mut peekable = MultiPeekTokenizer::new(tokenizer);

        // Peek ahead multiple times
        assert_eq!(peekable.peek().map(|t| t.value.as_ref()), Some("a"));
        assert_eq!(peekable.peek().map(|t| t.value.as_ref()), Some(" "));
        assert_eq!(peekable.peek().map(|t| t.value.as_ref()), Some("b"));
        assert_eq!(peekable.peek().map(|t| t.value.as_ref()), Some(" "));
        assert_eq!(peekable.peek().map(|t| t.value.as_ref()), Some("c"));

        // Reset and consume tokens normally
        peekable.reset_peek();
        assert_eq!(peekable.next().map(|t| t.value.to_string()), Some("a".to_string()));
        assert_eq!(peekable.next().map(|t| t.value.to_string()), Some(" ".to_string()));
        assert_eq!(peekable.next().map(|t| t.value.to_string()), Some("b".to_string()));
    }
}
