use super::brackets;
use super::comment;
use super::string;
use std::fmt;

/** CSS Whitespace Characters */
const HORIZONTAL_TAB: &char = &'\u{0009}';
const NEWLINE: &char = &'\u{000A}';
const VERTICAL_TAB: &char = &'\u{000B}';
const FORM_FEED: &char = &'\u{000C}';
const CARRIAGE_RETURN: &char = &'\u{000D}';
const SPACE: &char = &' ';

/** CSS Leading Word Markers */
const EXCLAMATION_POINT: &char = &'!';
const AT_CHAR: &char = &'@';

const BACKWARD_SLASH: &char = &'\\';

/** CSS Control Characters */
const OPEN_SQUARE_BRACKET: &char = &'[';
const CLOSE_SQUARE_BRACKET: &char = &']';
const OPEN_CURLY: &char = &'{';
const CLOSE_CURLY: &char = &'}';
const SEMICOLON: &char = &';';
const COLON: &char = &':';

const NULL_CHARACTER: &char = &'\u{0000}';

#[derive(Eq, PartialEq, Debug)]
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

#[derive(Eq, PartialEq, Debug)]
pub struct Token {
    start: usize,
    end: usize,
    kind: Kind,
}

impl fmt::Display for Token {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{{ kind: {}, start: {}, end: {} }}",
            self.kind, self.start, self.end
        )
    }
}

pub fn tokenizer(css: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let mut last_end: usize = 0;
    let mut loops: usize = 0;
    let letters: Vec<char> = css.chars().collect();
    let count = letters.iter().count();

    while let Some(letter) = letters.iter().skip(last_end).next() {
        let current_slice = &letters[last_end..];
        let next_letter = match letters.iter().skip(last_end + 1).next() {
            Some(value) => value,
            None => NULL_CHARACTER,
        };

        if loops > count {
            break;
        }

        match letter {
            letter if is_whitespace(letter) => {
                let advance = tokenize_space(current_slice);
                let (token, next_end) = create_token(last_end, advance, Kind::Space);

                tokens.push(token);
                last_end = next_end;
            }

            letter if is_leading_word_token(letter) => {
                let advance = tokenize_word(&current_slice[1..]);
                let kind = match letter {
                    AT_CHAR => Kind::AtWord,
                    _ => Kind::Word,
                };
                let (token, next_end) = create_token(last_end, advance + 1, kind);

                tokens.push(token);
                last_end = next_end;
            }

            letter if is_control(letter) => {
                let advance = 0;
                let (token, next_end) = create_token(last_end, advance, Kind::Symbol);

                tokens.push(token);
                last_end = next_end;
            }

            letter if comment::is_opening_comment_token(letter, next_letter) => {
                let advance = comment::tokenize_comment(&current_slice[2..]);
                let (token, next_end) = create_token(last_end, advance + 3, Kind::Comment);

                tokens.push(token);
                last_end = next_end;
            }

            BACKWARD_SLASH => {
                let advance = tokenize_backslash(&current_slice[1..]);
                let (token, next_end) = create_token(last_end, advance + 1, Kind::Word);

                tokens.push(token);
                last_end = next_end;
            }

            letter if brackets::is_opening_bracket_token(letter) => {
                let advance = brackets::tokenize_brackets(&current_slice);
                let (token, next_end) = create_token(last_end, advance, Kind::Brackets);

                tokens.push(token);
                last_end = next_end;
            }

            letter if string::is_single_string_token(letter) => {
                let advance = string::tokenize_single_string(&current_slice);
                let (token, next_end) = create_token(last_end, advance, Kind::String);

                tokens.push(token);
                last_end = next_end;
            }

            letter if string::is_double_string_token(letter) => {
                let advance = string::tokenize_double_string(&current_slice);
                let (token, next_end) = create_token(last_end, advance, Kind::String);

                tokens.push(token);
                last_end = next_end;
            }

            _ => {
                let advance = tokenize_word(current_slice);
                let (token, next_end) = create_token(last_end, advance, Kind::Word);

                tokens.push(token);
                last_end = next_end;
            }
        }

        loops = loops + 1;
    }

    return tokens;
}

fn create_token(last_end: usize, advance: usize, kind: Kind) -> (Token, usize) {
    let start = last_end;
    let end = start + advance;
    let token = Token { start, end, kind };

    println!("Creating a token, {}", token.to_string());

    return (token, end + 1);
}

fn tokenize_word(letters: &[char]) -> usize {
    let result = letters.iter().position(|letter| {
        return match letter {
            letter if is_whitespace(letter) => true,
            letter if is_control(letter) => true,
            letter if string::is_single_string_token(letter) => true,
            letter if string::is_double_string_token(letter) => true,
            letter if brackets::is_opening_bracket_token(letter) => true,
            letter if comment::is_forward_slash(letter) => true,
            EXCLAMATION_POINT | AT_CHAR => true,
            _ => false,
        };
    });

    let position = unwrap_position(result, letters);
    return back(position);
}

fn tokenize_backslash(letters: &[char]) -> usize {
    let result = letters.iter().position(|letter| {
        return match letter {
            BACKWARD_SLASH => false,
            _ => true,
        };
    });

    let position = unwrap_position(result, letters);
    return back(position);
}

fn tokenize_space(letters: &[char]) -> usize {
    let result = letters.iter().position(|letter| {
        return match letter {
            letter if is_whitespace(letter) => false,
            _ => true,
        };
    });

    let position = unwrap_position(result, letters);
    return back(position);
}

fn unwrap_position(result: Option<usize>, letters: &[char]) -> usize {
    let position = match result {
        Some(value) => value,
        None => letters.iter().count(),
    };

    return position;
}

fn back(index: usize) -> usize {
    if index == 0 {
        return index;
    }

    return index - 1;
}

fn is_whitespace(letter: &char) -> bool {
    return match letter {
        HORIZONTAL_TAB => true,
        NEWLINE => true,
        VERTICAL_TAB => true,
        FORM_FEED => true,
        CARRIAGE_RETURN => true,
        SPACE => true,
        _ => false,
    };
}

fn is_control(letter: &char) -> bool {
    return match letter {
        OPEN_CURLY => true,
        CLOSE_CURLY => true,
        OPEN_SQUARE_BRACKET => true,
        CLOSE_SQUARE_BRACKET => true,
        SEMICOLON => true,
        COLON => true,
        _ => false,
    };
}

fn is_leading_word_token(letter: &char) -> bool {
    return match letter {
        EXCLAMATION_POINT => true,
        AT_CHAR => true,
        _ => false,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn it_tokenizes_empty_file() {
        let empty: Vec<Token> = vec![];
        assert_eq!(tokenizer(""), empty)
    }

    #[test]
    fn it_tokenizes_empty_space() {
        assert_eq!(
            tokenizer("\r\n \t"),
            [Token {
                kind: Kind::Space,
                start: 0,
                end: 3,
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
                end: 1
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
                    end: 1
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 4
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
                    end: 0
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 3
                },
                Token {
                    kind: Kind::Word,
                    start: 4,
                    end: 4
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
                    end: 0
                },
                Token {
                    kind: Kind::Symbol,
                    start: 1,
                    end: 1
                },
                Token {
                    kind: Kind::Symbol,
                    start: 2,
                    end: 2
                },
                Token {
                    kind: Kind::Symbol,
                    start: 3,
                    end: 3
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
                    end: 1
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 3
                },
                Token {
                    kind: Kind::Word,
                    start: 4,
                    end: 5
                },
                Token {
                    kind: Kind::Word,
                    start: 6,
                    end: 7
                },
                Token {
                    kind: Kind::Word,
                    start: 8,
                    end: 9
                },
                Token {
                    kind: Kind::String,
                    start: 10,
                    end: 11
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
                    end: 3
                },
                Token {
                    kind: Kind::Symbol,
                    start: 4,
                    end: 4
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
                    end: 0
                },
                Token {
                    kind: Kind::Symbol,
                    start: 1,
                    end: 1
                },
                Token {
                    kind: Kind::Word,
                    start: 2,
                    end: 3
                },
                Token {
                    kind: Kind::Symbol,
                    start: 4,
                    end: 4
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
                    end: 0
                },
                Token {
                    kind: Kind::Brackets,
                    start: 1,
                    end: 2
                },
                Token {
                    kind: Kind::Symbol,
                    start: 3,
                    end: 3
                },
                Token {
                    kind: Kind::Symbol,
                    start: 4,
                    end: 4
                },
                Token {
                    kind: Kind::String,
                    start: 5,
                    end: 6
                },
                Token {
                    kind: Kind::Symbol,
                    start: 7,
                    end: 7
                },
                Token {
                    kind: Kind::Symbol,
                    start: 8,
                    end: 8
                },
                Token {
                    kind: Kind::Comment,
                    start: 9,
                    end: 12
                },
                Token {
                    kind: Kind::Symbol,
                    start: 13,
                    end: 13
                },
                Token {
                    kind: Kind::Symbol,
                    start: 14,
                    end: 14
                },
                Token {
                    kind: Kind::Word,
                    start: 15,
                    end: 16
                },
                Token {
                    kind: Kind::Symbol,
                    start: 17,
                    end: 17
                },
                Token {
                    kind: Kind::Symbol,
                    start: 18,
                    end: 18
                },
                Token {
                    kind: Kind::Space,
                    start: 19,
                    end: 19
                },
                Token {
                    kind: Kind::Symbol,
                    start: 20,
                    end: 20
                },
                Token {
                    kind: Kind::Symbol,
                    start: 21,
                    end: 21
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
                    end: 2
                },
                Token {
                    kind: Kind::String,
                    start: 3,
                    end: 6
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
                end: 3
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
                    end: 3
                },
                Token {
                    kind: Kind::String,
                    start: 4,
                    end: 7
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
                    end: 4
                },
                Token {
                    kind: Kind::Space,
                    start: 5,
                    end: 5
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
                    end: 3
                },
                Token {
                    kind: Kind::Symbol,
                    start: 4,
                    end: 4
                },
                Token {
                    kind: Kind::AtWord,
                    start: 5,
                    end: 8
                },
                Token {
                    kind: Kind::Brackets,
                    start: 9,
                    end: 10
                },
                Token {
                    kind: Kind::AtWord,
                    start: 11,
                    end: 16
                },
                Token {
                    kind: Kind::String,
                    start: 17,
                    end: 18
                },
                Token {
                    kind: Kind::AtWord,
                    start: 19,
                    end: 23
                },
                Token {
                    kind: Kind::Symbol,
                    start: 24,
                    end: 24
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
                    end: 2
                },
                Token {
                    kind: Kind::Brackets,
                    start: 3,
                    end: 8
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
                    end: 2
                },
                Token {
                    kind: Kind::Symbol,
                    start: 3,
                    end: 3
                },
                Token {
                    kind: Kind::String,
                    start: 4,
                    end: 6
                },
                Token {
                    kind: Kind::Symbol,
                    start: 7,
                    end: 7
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
                end: 0
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
                end: 8
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
                    end: 0
                },
                Token {
                    kind: Kind::Comment,
                    start: 1,
                    end: 7
                },
                Token {
                    kind: Kind::Word,
                    start: 8,
                    end: 8
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
                    end: 0
                },
                Token {
                    kind: Kind::Space,
                    start: 1,
                    end: 1,
                },
                Token {
                    kind: Kind::Symbol,
                    start: 2,
                    end: 2
                },
                Token {
                    kind: Kind::Space,
                    start: 3,
                    end: 5
                },
                Token {
                    kind: Kind::Word,
                    start: 6,
                    end: 12
                },
                Token {
                    kind: Kind::Symbol,
                    start: 13,
                    end: 13
                },
                Token {
                    kind: Kind::Space,
                    start: 14,
                    end: 14
                },
                Token {
                    kind: Kind::String,
                    start: 15,
                    end: 17
                },
                Token {
                    kind: Kind::Symbol,
                    start: 18,
                    end: 18
                },
                Token {
                    kind: Kind::Space,
                    start: 19,
                    end: 21
                },
                Token {
                    kind: Kind::Word,
                    start: 22,
                    end: 26
                },
                Token {
                    kind: Kind::Symbol,
                    start: 27,
                    end: 27
                },
                Token {
                    kind: Kind::Space,
                    start: 28,
                    end: 28
                },
                Token {
                    kind: Kind::Word,
                    start: 29,
                    end: 32
                },
                Token {
                    kind: Kind::Brackets,
                    start: 33,
                    end: 38
                },
                Token {
                    kind: Kind::Space,
                    start: 39,
                    end: 41
                },
                Token {
                    kind: Kind::Symbol,
                    start: 42,
                    end: 42
                },
                Token {
                    kind: Kind::Space,
                    start: 43,
                    end: 43
                },
                Token {
                    kind: Kind::Comment,
                    start: 44,
                    end: 61
                },
                Token {
                    kind: Kind::Space,
                    start: 62,
                    end: 62
                },
                Token {
                    kind: Kind::AtWord,
                    start: 63,
                    end: 68
                },
                Token {
                    kind: Kind::Space,
                    start: 69,
                    end: 69
                },
                Token {
                    kind: Kind::Word,
                    start: 70,
                    end: 75
                },
                Token {
                    kind: Kind::Space,
                    start: 76,
                    end: 76
                },
                Token {
                    kind: Kind::Symbol,
                    start: 77,
                    end: 77
                },
                Token {
                    kind: Kind::Symbol,
                    start: 78,
                    end: 78
                }
            ]
        )
    }
}
