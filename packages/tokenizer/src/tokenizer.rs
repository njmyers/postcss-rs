use crate::brackets::*;
use crate::characters::*;
use crate::comment::*;
use crate::escape::*;
use crate::line::*;
use crate::string::*;
use crate::symbol::*;
use crate::whitespace::*;
use crate::word::*;

use std::cell::RefCell;
use std::fmt;
use std::ops::Deref;

use itertools::multipeek;
use itertools::MultiPeek;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize, Copy, Clone)]
pub enum Kind {
    Space,
    Word,
    Brackets,
    String,
    AtWord,
    OpenCurly,
    CloseCurly,
    OpenSquare,
    CloseSquare,
    OpenCurvy,
    CloseCurvy,
    Semicolon,
    Colon,
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

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize, Clone)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub kind: Kind,
    pub value: String,
    pub column: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{{ kind: {}, start: {}, end: {}, value: {}, column: {} }}",
            self.kind, self.start, self.end, self.value, self.column
        )
    }
}

impl Token {
    /// Create a new token that represents a substring of this token
    /// Preserves accurate source positions from the original token
    pub fn slice(&self, start_offset: usize, end_offset: usize) -> Token {
        Token {
            kind: self.kind,
            value: self.value[start_offset..end_offset].to_string(),
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
    ///     value: ":black".to_string(),
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
    pub fn split(&self, split_index: usize, first_kind: Kind, second_kind: Kind) -> (Token, Option<Token>) {
        let first_token = Token {
            kind: first_kind,
            value: self.value[..split_index].to_string(),
            start: self.start,
            end: self.start + split_index,
            column: self.column,
        };

        let second_token = match self.value.len() {
            len if len > split_index => Some(Token {
                kind: second_kind,
                value: self.value[split_index..].to_string(),
                start: self.start + split_index,
                end: self.end,
                column: self.column + split_index,
            }),
            _ => None,
        };

        (first_token, second_token)
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Tokens(Vec<Token>);

impl Tokens {
    pub fn new() -> Self {
        return Self(Vec::new());
    }

    pub fn push(&mut self, token: Token) {
        self.0.push(token);
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn pop(&mut self) -> Option<Token> {
        self.0.pop()
    }

    pub fn last(&self) -> Option<&Token> {
        self.0.last()
    }
}

impl fmt::Display for Tokens {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        for token in self.0.iter() {
            write!(formatter, "{}", token.value)?;
        }

        return Ok(());
    }
}

impl Default for Tokens {
    fn default() -> Self {
        return Self::new();
    }
}

impl Deref for Tokens {
    type Target = Vec<Token>;

    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Tokenizer {
    css: String,
    letters: Vec<char>,
    stack: RefCell<Vec<Token>>,
    pub index: usize,
    pub lines: Lines,
}

impl Tokenizer {
    pub fn new(css: impl Into<String>) -> Self {
        let css = css.into();
        let letters: Vec<char> = css.chars().collect();

        return Self {
            css,
            index: 0,
            letters,
            stack: RefCell::new(Vec::new()),
            lines: Lines::new(),
        };
    }

    /// Peek at the next token without consuming it.
    ///
    /// This method clones the tokenizer and uses PeekableTokenizer internally.
    /// For multiple peek operations, consider using `into_peekable()` or
    /// `into_multipeek()` directly to avoid repeated cloning.
    pub fn peek(&self) -> Option<Token> {
        let mut peekable = self.clone().into_peekable();
        peekable.peek().cloned()
    }

    /// Peek ahead n tokens without consuming them.
    ///
    /// peek_nth(1) is equivalent to peek()
    /// peek_nth(2) returns the token after the next token
    ///
    /// This method clones the tokenizer and uses MultiPeekTokenizer internally.
    /// For multiple peek operations, consider using `into_multipeek()` directly
    /// to avoid repeated cloning.
    pub fn peek_nth(&self, n: usize) -> Option<Token> {
        if n == 0 {
            return None;
        }

        let mut peekable = self.clone().into_multipeek();

        // Peek n times - the last peek gives us the nth token
        let mut result = None;
        for _ in 0..n {
            result = peekable.peek().cloned();
            if result.is_none() {
                return None;
            }
        }

        result
    }

    pub fn prev(&self) -> Option<Token> {
        let stack = self.stack.borrow();
        return stack.last().cloned();
    }

    /// Convert this tokenizer into a PeekableTokenizer for simple next-token peeking.
    /// This is a convenience method equivalent to `PeekableTokenizer::new(tokenizer)`.
    ///
    /// # Example
    /// ```
    /// use postcss_rs_tokenizer::Tokenizer;
    ///
    /// let mut peekable = Tokenizer::new("a b c").into_peekable();
    /// assert_eq!(peekable.peek().map(|t| &t.value), Some(&"a".to_string()));
    /// ```
    pub fn into_peekable(self) -> PeekableTokenizer {
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
    /// assert_eq!(multi.peek().map(|t| &t.value), Some(&"a".to_string()));
    /// assert_eq!(multi.peek().map(|t| &t.value), Some(&" ".to_string()));
    /// ```
    pub fn into_multipeek(self) -> MultiPeekTokenizer {
        MultiPeekTokenizer::new(self)
    }

    fn create(&mut self, length: usize, kind: Kind) -> Token {
        let column = self.lines.offset(self.index);
        let start = self.index;
        let end = start + length;
        let index = end + 1;
        let value = &self.css[start..index];

        for (length, letter) in value.chars().enumerate() {
            if Newline::test(&letter) {
                self.lines.increment(start + length)
            }
        }

        self.index = index;

        let token = Token {
            start,
            end,
            kind,
            value: value.to_string(),
            column,
        };

        let mut stack = self.stack.borrow_mut();
        stack.push(token.clone());

        return token;
    }
}

impl Iterator for Tokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let letters = &self.letters[self.index..];
        let letter = self.letters.get(self.index)?;

        let next = match self.letters.get(self.index + 1) {
            Some(value) => value,
            None => NULL_CHARACTER,
        };

        match letter {
            letter if Whitespace::test(letter) => {
                let length = Whitespace::tokenize(letters);
                let token = self.create(length, Kind::Space);

                return Some(token);
            }

            letter if AtWord::test(letter, next) => {
                let length = AtWord::tokenize(letters);
                let token = self.create(length, Kind::AtWord);

                return Some(token);
            }

            letter if Word::test(letter, next) => {
                let length = Word::tokenize(letters);
                let token = self.create(length, Kind::Word);

                return Some(token);
            }

            letter if Bracket::test(letter, next, self.prev()) => {
                let length = Bracket::tokenize(letters);
                let token = self.create(length, Kind::Brackets);

                return Some(token);
            }

            letter if OpenCurly::test(letter) => {
                let length = OpenCurly::tokenize(letters);
                let token = self.create(length, Kind::OpenCurly);

                return Some(token);
            }

            letter if CloseCurly::test(letter) => {
                let length = CloseCurly::tokenize(letters);
                let token = self.create(length, Kind::CloseCurly);

                return Some(token);
            }

            letter if OpenSquare::test(letter) => {
                let length = OpenSquare::tokenize(letters);
                let token = self.create(length, Kind::OpenSquare);

                return Some(token);
            }

            letter if CloseSquare::test(letter) => {
                let length = CloseSquare::tokenize(letters);
                let token = self.create(length, Kind::CloseSquare);

                return Some(token);
            }

            letter if OpenCurvy::test(letter) => {
                let length = OpenCurvy::tokenize(letters);
                let token = self.create(length, Kind::OpenCurvy);

                return Some(token);
            }

            letter if CloseCurvy::test(letter) => {
                let length = CloseCurvy::tokenize(letters);
                let token = self.create(length, Kind::CloseCurvy);

                return Some(token);
            }

            letter if Semicolon::test(letter) => {
                let length = Semicolon::tokenize(letters);
                let token = self.create(length, Kind::Semicolon);

                return Some(token);
            }

            letter if Colon::test(letter) => {
                let length = Colon::tokenize(letters);
                let token = self.create(length, Kind::Colon);

                return Some(token);
            }

            letter if Comment::test(letter, next) => {
                let length = Comment::tokenize(&letters[2..]);
                let token = self.create(length + 3, Kind::Comment);

                return Some(token);
            }

            letter if Escape::test(letter) => {
                let length = Escape::tokenize(letters);
                let token = self.create(length, Kind::Word);

                return Some(token);
            }

            letter if SingleString::test(letter) => {
                let length = SingleString::tokenize(letters);
                let token = self.create(length, Kind::String);

                return Some(token);
            }

            letter if DoubleString::test(letter) => {
                let length = DoubleString::tokenize(letters);
                let token = self.create(length, Kind::String);

                return Some(token);
            }

            _ => {
                let length = Word::tokenize(letters);
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
/// assert_eq!(peekable.peek().map(|t| &t.value), Some(&"a".to_string()));
/// // Peek again returns the same token
/// assert_eq!(peekable.peek().map(|t| &t.value), Some(&"a".to_string()));
///
/// // Now consume it
/// assert_eq!(peekable.next().map(|t| t.value), Some("a".to_string()));
/// // Next token is a space
/// assert_eq!(peekable.peek().map(|t| &t.value), Some(&" ".to_string()));
/// ```
pub struct PeekableTokenizer {
    inner: std::iter::Peekable<Tokenizer>,
}

impl PeekableTokenizer {
    /// Create a new PeekableTokenizer from a Tokenizer
    pub fn new(tokenizer: Tokenizer) -> Self {
        Self {
            inner: tokenizer.peekable(),
        }
    }

    /// Peek at the next token without consuming it.
    /// Returns a reference to the next token, or None if the iterator is exhausted.
    pub fn peek(&mut self) -> Option<&Token> {
        self.inner.peek()
    }
}

impl Iterator for PeekableTokenizer {
    type Item = Token;

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
pub struct MultiPeekTokenizer {
    inner: MultiPeek<Tokenizer>,
}

impl MultiPeekTokenizer {
    /// Create a new MultiPeekTokenizer from a Tokenizer
    pub fn new(tokenizer: Tokenizer) -> Self {
        Self {
            inner: multipeek(tokenizer),
        }
    }

    /// Peek at the next token without consuming it.
    /// Can be called multiple times to peek further ahead.
    /// Call reset_peek() to reset the peek position.
    pub fn peek(&mut self) -> Option<&Token> {
        self.inner.peek()
    }

    /// Reset the peek position back to the current position.
    /// This should be called after you're done peeking and before
    /// you start consuming tokens or peeking again from the start.
    pub fn reset_peek(&mut self) {
        self.inner.reset_peek();
    }
}

impl Iterator for MultiPeekTokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    fn tokenizer(css: &str) -> Vec<Token> {
        let mut iterator = Tokenizer::new(css);
        let mut tokens: Vec<Token> = vec![];

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
                value: "\r\n \t".to_string(),
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
                value: "ab".to_string(),
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
                    value: "aa".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 4,
                    value: "!bb".to_string(),
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
                    value: "a".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 3,
                    value: " \n ".to_string(),
                    column: 2,
                },
                Token {
                    kind: Kind::Word,
                    start: 4,
                    end: 4,
                    value: "b".to_string(),
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
                    value: "{".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::Colon,
                    start: 1,
                    end: 1,
                    value: ":".to_string(),
                    column: 2,
                },
                Token {
                    kind: Kind::Semicolon,
                    start: 2,
                    end: 2,
                    value: ";".to_string(),
                    column: 3,
                },
                Token {
                    kind: Kind::CloseCurly,
                    start: 3,
                    end: 3,
                    value: "}".to_string(),
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
                    value: "\\(".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 3,
                    value: "\\{".to_string(),
                    column: 3,
                },
                Token {
                    kind: Kind::Word,
                    start: 4,
                    end: 5,
                    value: "\\\"".to_string(),
                    column: 5,
                },
                Token {
                    kind: Kind::Word,
                    start: 6,
                    end: 7,
                    value: "\\@".to_string(),
                    column: 7,
                },
                Token {
                    kind: Kind::Word,
                    start: 8,
                    end: 9,
                    value: "\\\\".to_string(),
                    column: 9,
                },
                Token {
                    kind: Kind::String,
                    start: 10,
                    end: 11,
                    value: "\"\"".to_string(),
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
                    value: "\\\\\\\\".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::OpenCurly,
                    start: 4,
                    end: 4,
                    value: "{".to_string(),
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
                value: "(ab)".to_string(),
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
                    value: "a".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::OpenSquare,
                    start: 1,
                    end: 1,
                    value: "[".to_string(),
                    column: 2,
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 3,
                    value: "bc".to_string(),
                    column: 3,
                },
                Token {
                    kind: Kind::CloseSquare,
                    start: 4,
                    end: 4,
                    value: "]".to_string(),
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
                    value: "(".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::Brackets,
                    start: 1,
                    end: 2,
                    value: "()".to_string(),
                    column: 2,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 3,
                    end: 3,
                    value: ")".to_string(),
                    column: 4,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 4,
                    end: 4,
                    value: "(".to_string(),
                    column: 5,
                },
                Token {
                    kind: Kind::String,
                    start: 5,
                    end: 6,
                    value: "\"\"".to_string(),
                    column: 6,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 7,
                    end: 7,
                    value: ")".to_string(),
                    column: 8,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 8,
                    end: 8,
                    value: "(".to_string(),
                    column: 9,
                },
                Token {
                    kind: Kind::Comment,
                    start: 9,
                    end: 12,
                    value: "/**/".to_string(),
                    column: 10,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 13,
                    end: 13,
                    value: ")".to_string(),
                    column: 14,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 14,
                    end: 14,
                    value: "(".to_string(),
                    column: 15,
                },
                Token {
                    kind: Kind::Word,
                    start: 15,
                    end: 16,
                    value: "\\\\".to_string(),
                    column: 16,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 17,
                    end: 17,
                    value: ")".to_string(),
                    column: 18,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 18,
                    end: 18,
                    value: "(".to_string(),
                    column: 19,
                },
                Token {
                    kind: Kind::Space,
                    start: 19,
                    end: 19,
                    value: "\n".to_string(),
                    column: 20,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 20,
                    end: 20,
                    value: ")".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 21,
                    end: 21,
                    value: "(".to_string(),
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
                    value: "'\"'".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::String,
                    start: 3,
                    end: 6,
                    value: "\"\\\"\"".to_string(),
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
                value: "\"\\\\\"".to_string(),
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
                    value: "\"\n\n\"".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::String,
                    start: 4,
                    end: 7,
                    value: "\"\n\n\"".to_string(),
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
                    value: "@word".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::Space,
                    start: 5,
                    end: 5,
                    value: " ".to_string(),
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
                    value: "@one".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::OpenCurly,
                    start: 4,
                    end: 4,
                    value: "{".to_string(),
                    column: 5,
                },
                Token {
                    kind: Kind::AtWord,
                    start: 5,
                    end: 8,
                    value: "@two".to_string(),
                    column: 6,
                },
                Token {
                    kind: Kind::Brackets,
                    start: 9,
                    end: 10,
                    value: "()".to_string(),
                    column: 10,
                },
                Token {
                    kind: Kind::AtWord,
                    start: 11,
                    end: 16,
                    value: "@three".to_string(),
                    column: 12,
                },
                Token {
                    kind: Kind::String,
                    start: 17,
                    end: 18,
                    value: "\"\"".to_string(),
                    column: 18,
                },
                Token {
                    kind: Kind::AtWord,
                    start: 19,
                    end: 23,
                    value: "@four".to_string(),
                    column: 20,
                },
                Token {
                    kind: Kind::Semicolon,
                    start: 24,
                    end: 24,
                    value: ";".to_string(),
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
                    value: "url".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::Brackets,
                    start: 3,
                    end: 8,
                    value: "(/*\\))".to_string(),
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
                    value: "url".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 3,
                    end: 3,
                    value: "(".to_string(),
                    column: 4,
                },
                Token {
                    kind: Kind::String,
                    start: 4,
                    end: 6,
                    value: "\")\"".to_string(),
                    column: 5,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 7,
                    end: 7,
                    value: ")".to_string(),
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
                value: "@".to_string(),
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
                value: "/* a\nb */".to_string(),
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
                    value: "a".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::Comment,
                    start: 1,
                    end: 7,
                    value: "/* \n */".to_string(),
                    column: 2,
                },
                Token {
                    kind: Kind::Word,
                    start: 8,
                    end: 8,
                    value: "b".to_string(),
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
                    value: "a".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 1,
                    value: "\u{000C}".to_string(),
                    column: 2,
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 2,
                    value: "b".to_string(),
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
                    value: "a".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 1,
                    value: "\r".to_string(),
                    column: 2,
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 2,
                    value: "b".to_string(),
                    column: 1,
                },
                Token {
                    kind: Kind::Space,
                    start: 3,
                    end: 4,
                    value: "\r\n".to_string(),
                    column: 2,
                },
                Token {
                    kind: Kind::Word,
                    start: 5,
                    end: 5,
                    value: "c".to_string(),
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
                    value: "a".to_string(),
                    column: 1
                },
                Token {
                    start: 1,
                    end: 1,
                    kind: Kind::Space,
                    value: " ".to_string(),
                    column: 2
                },
                Token {
                    start: 2,
                    end: 2,
                    kind: Kind::OpenCurly,
                    value: "{".to_string(),
                    column: 3
                },
                Token {
                    start: 3,
                    end: 5,
                    kind: Kind::Space,
                    value: "\n  ".to_string(),
                    column: 4
                },
                Token {
                    start: 6,
                    end: 12,
                    kind: Kind::Word,
                    value: "content".to_string(),
                    column: 3
                },
                Token {
                    start: 13,
                    end: 13,
                    kind: Kind::Colon,
                    value: ":".to_string(),
                    column: 10
                },
                Token {
                    start: 14,
                    end: 14,
                    kind: Kind::Space,
                    value: " ".to_string(),
                    column: 11
                },
                Token {
                    start: 15,
                    end: 17,
                    kind: Kind::String,
                    value: "\"a\"".to_string(),
                    column: 12
                },
                Token {
                    start: 18,
                    end: 18,
                    kind: Kind::Semicolon,
                    value: ";".to_string(),
                    column: 15
                },
                Token {
                    start: 19,
                    end: 21,
                    kind: Kind::Space,
                    value: "\n  ".to_string(),
                    column: 16
                },
                Token {
                    start: 22,
                    end: 26,
                    kind: Kind::Word,
                    value: "width".to_string(),
                    column: 3
                },
                Token {
                    start: 27,
                    end: 27,
                    kind: Kind::Colon,
                    value: ":".to_string(),
                    column: 8
                },
                Token {
                    start: 28,
                    end: 28,
                    kind: Kind::Space,
                    value: " ".to_string(),
                    column: 9
                },
                Token {
                    start: 29,
                    end: 32,
                    kind: Kind::Word,
                    value: "calc".to_string(),
                    column: 10
                },
                Token {
                    start: 33,
                    end: 38,
                    kind: Kind::Brackets,
                    value: "(1px;)".to_string(),
                    column: 14
                },
                Token {
                    start: 39,
                    end: 41,
                    kind: Kind::Space,
                    value: "\n  ".to_string(),
                    column: 20
                },
                Token {
                    start: 42,
                    end: 42,
                    kind: Kind::CloseCurly,
                    value: "}".to_string(),
                    column: 3
                },
                Token {
                    start: 43,
                    end: 43,
                    kind: Kind::Space,
                    value: "\n".to_string(),
                    column: 4
                },
                Token {
                    start: 44,
                    end: 61,
                    kind: Kind::Comment,
                    value: "/* small screen */".to_string(),
                    column: 1
                },
                Token {
                    start: 62,
                    end: 62,
                    kind: Kind::Space,
                    value: "\n".to_string(),
                    column: 19
                },
                Token {
                    start: 63,
                    end: 68,
                    kind: Kind::AtWord,
                    value: "@media".to_string(),
                    column: 1
                },
                Token {
                    start: 69,
                    end: 69,
                    kind: Kind::Space,
                    value: " ".to_string(),
                    column: 7
                },
                Token {
                    start: 70,
                    end: 75,
                    kind: Kind::Word,
                    value: "screen".to_string(),
                    column: 8
                },
                Token {
                    start: 76,
                    end: 76,
                    kind: Kind::Space,
                    value: " ".to_string(),
                    column: 14
                },
                Token {
                    start: 77,
                    end: 77,
                    kind: Kind::OpenCurly,
                    value: "{".to_string(),
                    column: 15
                },
                Token {
                    start: 78,
                    end: 78,
                    kind: Kind::CloseCurly,
                    value: "}".to_string(),
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

        assert_eq!(peekable.peek().map(|t| &t.value), Some(&"a".to_string()));
        assert_eq!(peekable.next().map(|t| t.value), Some("a".to_string()));
    }

    #[test]
    fn test_tokenizer_into_multipeek() {
        // Test the convenience method
        let mut multi = Tokenizer::new("a b c").into_multipeek();

        // Peek ahead multiple times
        assert_eq!(multi.peek().map(|t| &t.value), Some(&"a".to_string()));
        assert_eq!(multi.peek().map(|t| &t.value), Some(&" ".to_string()));
        assert_eq!(multi.peek().map(|t| &t.value), Some(&"b".to_string()));

        multi.reset_peek();
        assert_eq!(multi.next().map(|t| t.value), Some("a".to_string()));
    }

    #[test]
    fn test_peekable_tokenizer_basic() {
        let tokenizer = Tokenizer::new("a { color: red; }");
        let mut peekable = PeekableTokenizer::new(tokenizer);

        // Peek at next token without consuming
        let peeked = peekable.peek().map(|t| t.value.clone());
        assert_eq!(peeked, Some("a".to_string()));

        // Peek again should return the same token (not advance)
        let peeked_again = peekable.peek().map(|t| t.value.clone());
        assert_eq!(peeked_again, Some("a".to_string()));

        // Now consume the token
        let consumed = peekable.next().map(|t| t.value);
        assert_eq!(consumed, Some("a".to_string()));

        // Peek at the next token (space)
        assert_eq!(peekable.peek().map(|t| t.value.as_str()), Some(" "));
    }

    #[test]
    fn test_multi_peek_tokenizer_basic() {
        let tokenizer = Tokenizer::new("a { color: red; }");
        let mut peekable = MultiPeekTokenizer::new(tokenizer);

        // Peek at first token without consuming
        let peeked = peekable.peek().map(|t| t.value.clone());
        assert_eq!(peeked, Some("a".to_string()));

        // Peek again advances to next token
        let peeked_again = peekable.peek().map(|t| t.value.clone());
        assert_eq!(peeked_again, Some(" ".to_string()));

        // Reset and peek from start again
        peekable.reset_peek();
        let peeked_reset = peekable.peek().map(|t| t.value.clone());
        assert_eq!(peeked_reset, Some("a".to_string()));

        // Now consume the token
        peekable.reset_peek();
        let consumed = peekable.next().map(|t| t.value);
        assert_eq!(consumed, Some("a".to_string()));
    }

    #[test]
    fn test_multi_peek_tokenizer_lookahead() {
        let tokenizer = Tokenizer::new("a b c");
        let mut peekable = MultiPeekTokenizer::new(tokenizer);

        // Peek ahead multiple times
        assert_eq!(peekable.peek().map(|t| t.value.as_str()), Some("a"));
        assert_eq!(peekable.peek().map(|t| t.value.as_str()), Some(" "));
        assert_eq!(peekable.peek().map(|t| t.value.as_str()), Some("b"));
        assert_eq!(peekable.peek().map(|t| t.value.as_str()), Some(" "));
        assert_eq!(peekable.peek().map(|t| t.value.as_str()), Some("c"));

        // Reset and consume tokens normally
        peekable.reset_peek();
        assert_eq!(peekable.next().map(|t| t.value), Some("a".to_string()));
        assert_eq!(peekable.next().map(|t| t.value), Some(" ".to_string()));
        assert_eq!(peekable.next().map(|t| t.value), Some("b".to_string()));
    }
}
