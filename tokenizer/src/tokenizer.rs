use super::brackets::*;
use super::comment::*;
use super::escape::*;
use super::string::*;
use super::symbol::*;
use super::whitespace::*;
use super::word::*;

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fmt;

const NULL_CHARACTER: &char = &'\u{0000}';

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Copy, Clone)]
pub enum Kind {
    Space,
    Word,
    Brackets,
    String,
    AtWord,
    Symbol,
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
            Kind::Symbol => write!(formatter, "Symbol"),
            Kind::Comment => write!(formatter, "Comment"),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Copy, Clone)]
pub struct Token<'a> {
    pub start: usize,
    pub end: usize,
    pub kind: Kind,
    pub value: &'a str,
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

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Lines(RefCell<usize>);

impl Lines {
    fn new() -> Self {
        return Self(RefCell::new(1));
    }

    fn increment(&self) {
        let mut lines = self.0.borrow_mut();
        *lines += 1;
    }

    pub fn value(&self) -> usize {
        return self.0.borrow().to_owned();
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Tokenizer<'a> {
    index: usize,
    css: &'a str,
    letters: Vec<char>,
    count: usize,
    pub lines: Lines,
}

impl<'a> Tokenizer<'a> {
    pub fn new(css: &'a str) -> Self {
        let letters: Vec<char> = css.chars().collect();
        let count = letters.iter().count();

        return Self {
            css,
            index: 0,
            letters,
            count,
            lines: Lines::new(),
        };
    }

    fn create(&mut self, advance: usize, kind: Kind) -> Token<'a> {
        let start = self.index;
        let end = start + advance;
        self.index = end + 1;

        let token = Token {
            start,
            end,
            kind,
            value: &self.css[start..end + 1],
        };

        return token;
    }

    pub fn peek(&self) -> Option<Token> {
        return self.clone().next();
    }

    pub fn end(&self) -> usize {
        return self.index.to_owned() - 1;
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let letters = &self.letters[self.index..];
        let next = self.letters.iter().skip(self.index).next();

        if next.is_none() {
            return None;
        }

        let letter = next.unwrap();
        let next = match self.letters.iter().skip(self.index + 1).next() {
            Some(value) => value,
            None => NULL_CHARACTER,
        };

        match letter {
            letter if Whitespace::test(letter) => {
                if Newline::test(letter) {
                    self.lines.increment();
                }

                let advance = Whitespace::tokenize(&letters);
                let token = self.create(advance, Kind::Space);

                return Some(token);
            }

            letter if AtWord::test(letter, next) => {
                let advance = AtWord::tokenize(&letters);

                let token = self.create(advance, Kind::AtWord);
                return Some(token);
            }

            letter if Word::test(letter, next) => {
                let advance = Word::tokenize(&letters);
                let token = self.create(advance, Kind::Word);
                return Some(token);
            }

            letter if Symbol::test(letter) => {
                let advance = 0;
                let token = self.create(advance, Kind::Symbol);
                return Some(token);
            }

            letter if Comment::test(letter, next) => {
                let advance = Comment::tokenize(&letters[2..]);
                let token = self.create(advance + 3, Kind::Comment);
                return Some(token);
            }

            letter if Escape::test(letter) => {
                let advance = Escape::tokenize(&letters);
                let token = self.create(advance, Kind::Word);
                return Some(token);
            }

            letter if Bracket::test(letter) => {
                let advance = Bracket::tokenize(&letters);
                let token = self.create(advance, Kind::Brackets);
                return Some(token);
            }

            letter if SingleString::test(letter) => {
                let advance = SingleString::tokenize(&letters);
                let token = self.create(advance, Kind::String);
                return Some(token);
            }

            letter if DoubleString::test(letter) => {
                let advance = DoubleString::tokenize(&letters);
                let token = self.create(advance, Kind::String);
                return Some(token);
            }

            _ => {
                let advance = Word::tokenize(&letters);
                let token = self.create(advance, Kind::Word);
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
                value: "\r\n \t"
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
                value: "ab"
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
                    value: "aa"
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 4,
                    value: "!bb"
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
                    value: "a"
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 3,
                    value: " \n "
                },
                Token {
                    kind: Kind::Word,
                    start: 4,
                    end: 4,
                    value: "b"
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
                    kind: Kind::Symbol,
                    start: 0,
                    end: 0,
                    value: "{"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 1,
                    end: 1,
                    value: ":"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 2,
                    end: 2,
                    value: ";"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 3,
                    end: 3,
                    value: "}"
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
                    value: "\\("
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 3,
                    value: "\\{"
                },
                Token {
                    kind: Kind::Word,
                    start: 4,
                    end: 5,
                    value: "\\\""
                },
                Token {
                    kind: Kind::Word,
                    start: 6,
                    end: 7,
                    value: "\\@"
                },
                Token {
                    kind: Kind::Word,
                    start: 8,
                    end: 9,
                    value: "\\\\"
                },
                Token {
                    kind: Kind::String,
                    start: 10,
                    end: 11,
                    value: "\"\""
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
                    value: "\\\\\\\\"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 4,
                    end: 4,
                    value: "{"
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
                value: "(ab)"
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
                    value: "a"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 1,
                    end: 1,
                    value: "["
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 3,
                    value: "bc"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 4,
                    end: 4,
                    value: "]"
                }
            ]
        );
    }

    #[test]
    #[ignore]

    fn it_tokenizes_complicated_brackets() {
        assert_eq!(
            tokenizer("(())(\"\")(/**/)(\\\\)(\n)("),
            [
                Token {
                    kind: Kind::Symbol,
                    start: 0,
                    end: 0,
                    value: "("
                },
                Token {
                    kind: Kind::Brackets,
                    start: 1,
                    end: 2,
                    value: "()"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 3,
                    end: 3,
                    value: ")"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 4,
                    end: 4,
                    value: "("
                },
                Token {
                    kind: Kind::String,
                    start: 5,
                    end: 6,
                    value: "\"\""
                },
                Token {
                    kind: Kind::Symbol,
                    start: 7,
                    end: 7,
                    value: ")"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 8,
                    end: 8,
                    value: "("
                },
                Token {
                    kind: Kind::Comment,
                    start: 9,
                    end: 12,
                    value: "/**/"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 13,
                    end: 13,
                    value: ")"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 14,
                    end: 14,
                    value: "("
                },
                Token {
                    kind: Kind::Word,
                    start: 15,
                    end: 16,
                    value: "\\\\"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 17,
                    end: 17,
                    value: ")"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 18,
                    end: 18,
                    value: "("
                },
                Token {
                    kind: Kind::Space,
                    start: 19,
                    end: 19,
                    value: "\n"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 20,
                    end: 20,
                    value: ")"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 21,
                    end: 21,
                    value: "("
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
                    value: "'\"'"
                },
                Token {
                    kind: Kind::String,
                    start: 3,
                    end: 6,
                    value: "\"\\\"\""
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
                value: "\"\\\\\""
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
                    value: "\"\n\n\""
                },
                Token {
                    kind: Kind::String,
                    start: 4,
                    end: 7,
                    value: "\"\n\n\""
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
                    value: "@word"
                },
                Token {
                    kind: Kind::Space,
                    start: 5,
                    end: 5,
                    value: " "
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
                    value: "@one"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 4,
                    end: 4,
                    value: "{"
                },
                Token {
                    kind: Kind::AtWord,
                    start: 5,
                    end: 8,
                    value: "@two"
                },
                Token {
                    kind: Kind::Brackets,
                    start: 9,
                    end: 10,
                    value: "()",
                },
                Token {
                    kind: Kind::AtWord,
                    start: 11,
                    end: 16,
                    value: "@three",
                },
                Token {
                    kind: Kind::String,
                    start: 17,
                    end: 18,
                    value: "\"\""
                },
                Token {
                    kind: Kind::AtWord,
                    start: 19,
                    end: 23,
                    value: "@four"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 24,
                    end: 24,
                    value: ";"
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
                    value: "url"
                },
                Token {
                    kind: Kind::Brackets,
                    start: 3,
                    end: 8,
                    value: "(/*\\))"
                }
            ]
        )
    }

    #[test]
    #[ignore]

    fn it_tokenizes_quoted_urls() {
        assert_eq!(
            tokenizer("url(\")\")"),
            [
                Token {
                    kind: Kind::Word,
                    start: 0,
                    end: 2,
                    value: "url"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 3,
                    end: 3,
                    value: "("
                },
                Token {
                    kind: Kind::String,
                    start: 4,
                    end: 6,
                    value: "\")\""
                },
                Token {
                    kind: Kind::Symbol,
                    start: 7,
                    end: 7,
                    value: ")"
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
                value: "@"
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
                value: "/* a\nb */"
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
                    value: "a"
                },
                Token {
                    kind: Kind::Comment,
                    start: 1,
                    end: 7,
                    value: "/* \n */"
                },
                Token {
                    kind: Kind::Word,
                    start: 8,
                    end: 8,
                    value: "b"
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
                    value: "a"
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 1,
                    value: "\u{000C}"
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 2,
                    value: "b"
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
                    value: "a"
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 1,
                    value: "\r",
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 2,
                    value: "b"
                },
                Token {
                    kind: Kind::Space,
                    start: 3,
                    end: 4,
                    value: "\r\n"
                },
                Token {
                    kind: Kind::Word,
                    start: 5,
                    end: 5,
                    value: "c"
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
                    kind: Kind::Word,
                    start: 0,
                    end: 0,
                    value: "a",
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 1,
                    value: " "
                },
                Token {
                    kind: Kind::Symbol,
                    start: 2,
                    end: 2,
                    value: "{"
                },
                Token {
                    kind: Kind::Space,
                    start: 3,
                    end: 5,
                    value: "\n  "
                },
                Token {
                    kind: Kind::Word,
                    start: 6,
                    end: 12,
                    value: "content"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 13,
                    end: 13,
                    value: ":"
                },
                Token {
                    kind: Kind::Space,
                    start: 14,
                    end: 14,
                    value: " "
                },
                Token {
                    kind: Kind::String,
                    start: 15,
                    end: 17,
                    value: "\"a\""
                },
                Token {
                    kind: Kind::Symbol,
                    start: 18,
                    end: 18,
                    value: ";"
                },
                Token {
                    kind: Kind::Space,
                    start: 19,
                    end: 21,
                    value: "\n  "
                },
                Token {
                    kind: Kind::Word,
                    start: 22,
                    end: 26,
                    value: "width"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 27,
                    end: 27,
                    value: ":"
                },
                Token {
                    kind: Kind::Space,
                    start: 28,
                    end: 28,
                    value: " "
                },
                Token {
                    kind: Kind::Word,
                    start: 29,
                    end: 32,
                    value: "calc"
                },
                Token {
                    kind: Kind::Brackets,
                    start: 33,
                    end: 38,
                    value: "(1px;)"
                },
                Token {
                    kind: Kind::Space,
                    start: 39,
                    end: 41,
                    value: "\n  "
                },
                Token {
                    kind: Kind::Symbol,
                    start: 42,
                    end: 42,
                    value: "}"
                },
                Token {
                    kind: Kind::Space,
                    start: 43,
                    end: 43,
                    value: "\n"
                },
                Token {
                    kind: Kind::Comment,
                    start: 44,
                    end: 61,
                    value: "/* small screen */"
                },
                Token {
                    kind: Kind::Space,
                    start: 62,
                    end: 62,
                    value: "\n"
                },
                Token {
                    kind: Kind::AtWord,
                    start: 63,
                    end: 68,
                    value: "@media"
                },
                Token {
                    kind: Kind::Space,
                    start: 69,
                    end: 69,
                    value: " "
                },
                Token {
                    kind: Kind::Word,
                    start: 70,
                    end: 75,
                    value: "screen"
                },
                Token {
                    kind: Kind::Space,
                    start: 76,
                    end: 76,
                    value: " "
                },
                Token {
                    kind: Kind::Symbol,
                    start: 77,
                    end: 77,
                    value: "{"
                },
                Token {
                    kind: Kind::Symbol,
                    start: 78,
                    end: 78,
                    value: "}"
                }
            ]
        )
    }
}
