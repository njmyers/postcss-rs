use crate::brackets::*;
use crate::characters::*;
use crate::comment::*;
use crate::escape::*;
use crate::string::*;
use crate::symbol::*;
use crate::whitespace::*;
use crate::word::*;

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fmt;

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

#[derive(PartialEq, Eq, Debug, Deserialize, Serialize, Copy, Clone)]
pub struct Token<'a> {
    pub start: usize,
    pub end: usize,
    pub kind: Kind,
    pub value: &'a str,
    pub column: usize,
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{{ kind: {}, start: {}, end: {} }}",
            self.kind, self.start, self.end
        )
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Line {
    index: usize,
}

impl Line {
    pub fn new(index: usize) -> Self {
        return Self { index };
    }
}

impl Default for Line {
    fn default() -> Self {
        return Self::new(0);
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Lines {
    lines: RefCell<Vec<Line>>,
}

impl Lines {
    fn new() -> Self {
        return Self {
            lines: RefCell::new(vec![Line::default()]),
        };
    }

    fn increment(&self, index: usize) {
        let mut lines = self.lines.borrow_mut();
        lines.push(Line::new(index));
    }

    pub fn value(&self) -> usize {
        return self.lines.borrow().iter().count();
    }

    pub fn offset(&self, index: usize) -> usize {
        if let Some(line) = self.lines.borrow().last() {
            return index - line.index;
        }

        return 1;
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Tokenizer<'a> {
    css: &'a str,
    letters: Vec<char>,
    stack: RefCell<Vec<Token<'a>>>,
    pub index: usize,
    pub lines: Lines,
}

impl<'a> Tokenizer<'a> {
    pub fn new(css: &'a str) -> Self {
        let letters: Vec<char> = css.chars().collect();

        return Self {
            css,
            index: 0,
            letters,
            stack: RefCell::new(Vec::new()),
            lines: Lines::new(),
        };
    }

    pub fn peek(&self) -> Option<Token> {
        return self.clone().next();
    }

    pub fn prev(&self) -> Option<Token> {
        let mut stack = self.stack.borrow_mut();

        if let Some(token) = stack.pop() {
            stack.push(token);
            return Some(token);
        }

        return None;
    }

    fn create(&mut self, length: usize, kind: Kind) -> Token<'a> {
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
            value,
            column,
        };

        let mut stack = self.stack.borrow_mut();
        stack.push(token);

        return token;
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

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
                value: "\r\n \t",
                column: 0,
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
                column: 0,
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
                    column: 0,
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 4,
                    value: "!bb",
                    column: 2,
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
                    column: 0,
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 3,
                    value: " \n ",
                    column: 1,
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
                    column: 0,
                },
                Token {
                    kind: Kind::Colon,
                    start: 1,
                    end: 1,
                    value: ":",
                    column: 1,
                },
                Token {
                    kind: Kind::Semicolon,
                    start: 2,
                    end: 2,
                    value: ";",
                    column: 2,
                },
                Token {
                    kind: Kind::CloseCurly,
                    start: 3,
                    end: 3,
                    value: "}",
                    column: 3,
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
                    column: 0,
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 3,
                    value: "\\{",
                    column: 2,
                },
                Token {
                    kind: Kind::Word,
                    start: 4,
                    end: 5,
                    value: "\\\"",
                    column: 4,
                },
                Token {
                    kind: Kind::Word,
                    start: 6,
                    end: 7,
                    value: "\\@",
                    column: 6,
                },
                Token {
                    kind: Kind::Word,
                    start: 8,
                    end: 9,
                    value: "\\\\",
                    column: 8,
                },
                Token {
                    kind: Kind::String,
                    start: 10,
                    end: 11,
                    value: "\"\"",
                    column: 10,
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
                    column: 0,
                },
                Token {
                    kind: Kind::OpenCurly,
                    start: 4,
                    end: 4,
                    value: "{",
                    column: 4,
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
                column: 0,
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
                    column: 0,
                },
                Token {
                    kind: Kind::OpenSquare,
                    start: 1,
                    end: 1,
                    value: "[",
                    column: 1,
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 3,
                    value: "bc",
                    column: 2,
                },
                Token {
                    kind: Kind::CloseSquare,
                    start: 4,
                    end: 4,
                    value: "]",
                    column: 4,
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
                    column: 0,
                },
                Token {
                    kind: Kind::Brackets,
                    start: 1,
                    end: 2,
                    value: "()",
                    column: 1,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 3,
                    end: 3,
                    value: ")",
                    column: 3,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 4,
                    end: 4,
                    value: "(",
                    column: 4,
                },
                Token {
                    kind: Kind::String,
                    start: 5,
                    end: 6,
                    value: "\"\"",
                    column: 5,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 7,
                    end: 7,
                    value: ")",
                    column: 7,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 8,
                    end: 8,
                    value: "(",
                    column: 8,
                },
                Token {
                    kind: Kind::Comment,
                    start: 9,
                    end: 12,
                    value: "/**/",
                    column: 9,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 13,
                    end: 13,
                    value: ")",
                    column: 13,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 14,
                    end: 14,
                    value: "(",
                    column: 14,
                },
                Token {
                    kind: Kind::Word,
                    start: 15,
                    end: 16,
                    value: "\\\\",
                    column: 15,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 17,
                    end: 17,
                    value: ")",
                    column: 17,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 18,
                    end: 18,
                    value: "(",
                    column: 18,
                },
                Token {
                    kind: Kind::Space,
                    start: 19,
                    end: 19,
                    value: "\n",
                    column: 19,
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
                    column: 0,
                },
                Token {
                    kind: Kind::String,
                    start: 3,
                    end: 6,
                    value: "\"\\\"\"",
                    column: 3,
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
                column: 0,
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
                    column: 0,
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
                    column: 0,
                },
                Token {
                    kind: Kind::Space,
                    start: 5,
                    end: 5,
                    value: " ",
                    column: 5,
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
                    column: 0,
                },
                Token {
                    kind: Kind::OpenCurly,
                    start: 4,
                    end: 4,
                    value: "{",
                    column: 4,
                },
                Token {
                    kind: Kind::AtWord,
                    start: 5,
                    end: 8,
                    value: "@two",
                    column: 5,
                },
                Token {
                    kind: Kind::Brackets,
                    start: 9,
                    end: 10,
                    value: "()",
                    column: 9,
                },
                Token {
                    kind: Kind::AtWord,
                    start: 11,
                    end: 16,
                    value: "@three",
                    column: 11,
                },
                Token {
                    kind: Kind::String,
                    start: 17,
                    end: 18,
                    value: "\"\"",
                    column: 17,
                },
                Token {
                    kind: Kind::AtWord,
                    start: 19,
                    end: 23,
                    value: "@four",
                    column: 19,
                },
                Token {
                    kind: Kind::Semicolon,
                    start: 24,
                    end: 24,
                    value: ";",
                    column: 24,
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
                    column: 0,
                },
                Token {
                    kind: Kind::Brackets,
                    start: 3,
                    end: 8,
                    value: "(/*\\))",
                    column: 3,
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
                    column: 0,
                },
                Token {
                    kind: Kind::OpenCurvy,
                    start: 3,
                    end: 3,
                    value: "(",
                    column: 3,
                },
                Token {
                    kind: Kind::String,
                    start: 4,
                    end: 6,
                    value: "\")\"",
                    column: 4,
                },
                Token {
                    kind: Kind::CloseCurvy,
                    start: 7,
                    end: 7,
                    value: ")",
                    column: 7,
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
                column: 0,
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
                column: 0,
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
                    column: 0,
                },
                Token {
                    kind: Kind::Comment,
                    start: 1,
                    end: 7,
                    value: "/* \n */",
                    column: 1,
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
                    column: 0,
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 1,
                    value: "\u{000C}",
                    column: 1,
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 2,
                    value: "b",
                    column: 2,
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
                    column: 0,
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 1,
                    value: "\r",
                    column: 1,
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
                    column: 0
                },
                Token {
                    start: 1,
                    end: 1,
                    kind: Kind::Space,
                    value: " ",
                    column: 1
                },
                Token {
                    start: 2,
                    end: 2,
                    kind: Kind::OpenCurly,
                    value: "{",
                    column: 2
                },
                Token {
                    start: 3,
                    end: 5,
                    kind: Kind::Space,
                    value: "\n  ",
                    column: 3
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
}
