// Whitespace
const HORIZONTAL_TAB: &char = &'\u{0009}';
const NEWLINE: &char = &'\u{000A}';
const VERTICAL_TAB: &char = &'\u{000B}';
const FORM_FEED: &char = &'\u{000C}';
const CARRIAGE_RETURN: &char = &'\u{000D}';
const SPACE: &char = &' ';

const EXCLAMATION_POINT: &char = &'!';
const AT_CHAR: &char = &'@';

const SINGLE_QUOTE: &char = &'\'';
const DOUBLE_QUOTE: &char = &'"';
const BACKWARD_SLASH: &char = &'\\';
// const FORWARD_SLASH: &char = &'/';

// const OPEN_SQUARE: &char = &'[';
// const CLOSE_SQUARE: &char = &']';
// const OPEN_PARENTHESES: &char = &'(';
// const CLOSE_PARENTHESES: &char = &')';
const OPEN_CURLY: &char = &'{';
const CLOSE_CURLY: &char = &'}';
const SEMICOLON: &char = &';';
const COLON: &char = &':';
// const ASTERISK: &char = &'*';

#[derive(Eq, PartialEq, Debug)]
pub enum TokenType {
    Space,
    Word,
    Brackets,
    String,
    AtWord,
    Symbol,
}

#[derive(Eq, PartialEq, Debug)]
pub struct Token {
    start: usize,
    end: usize,
    token_type: TokenType,
}

pub fn tokenizer(css: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];
    let mut last_end: usize = 0;
    let mut loops: usize = 0;
    let letters: Vec<char> = css.chars().collect();
    let count = letters.iter().count();

    while let Some(char) = letters.iter().skip(last_end).next() {
        let current_slice = &letters[last_end..];

        if loops > count {
            break;
        }

        match char {
            char if is_whitespace(char) => {
                let advance = tokenize_space(current_slice);
                let (token, next_end) = create_token(last_end, advance, TokenType::Space);

                tokens.push(token);
                last_end = next_end;
            }

            char if is_leading_word_token(char) => {
                let advance = tokenize_word(&current_slice[1..]);
                let (token, next_end) = create_token(last_end, advance + 1, TokenType::Word);

                tokens.push(token);
                last_end = next_end;
            }

            char if is_control(char) => {
                let advance = 0;
                let (token, next_end) = create_token(last_end, advance, TokenType::Symbol);

                tokens.push(token);
                last_end = next_end;
            }

            BACKWARD_SLASH => {
                let advance = 1;
                let (token, next_end) = create_token(last_end, advance, TokenType::Word);

                tokens.push(token);
                last_end = next_end;
            }

            SINGLE_QUOTE | DOUBLE_QUOTE => {
                let advance = tokenize_string(&current_slice[1..]);
                let (token, next_end) = create_token(last_end, advance + 1, TokenType::String);

                tokens.push(token);
                last_end = next_end;
            }

            _ => {
                let advance = tokenize_word(current_slice);
                let (token, next_end) = create_token(last_end, advance, TokenType::Word);

                tokens.push(token);
                last_end = next_end;
            }
        }

        loops = loops + 1;
    }

    return tokens;
}

fn create_token(last_end: usize, advance: usize, token_type: TokenType) -> (Token, usize) {
    let start = last_end;
    let end = start + advance;
    let token = Token {
        start,
        end,
        token_type,
    };

    return (token, end + 1);
}

fn tokenize_word(letters: &[char]) -> usize {
    let result = letters.iter().position(|char| {
        return match char as &char {
            char if is_whitespace(char) => true,
            EXCLAMATION_POINT | AT_CHAR => true,
            _ => false,
        };
    });

    return unwrap_position(result, letters);
}

fn tokenize_string(letters: &[char]) -> usize {
    let result = letters.iter().position(|char| {
        return match char as &char {
            SINGLE_QUOTE | DOUBLE_QUOTE => true,
            _ => false,
        };
    });

    return unwrap_position(result, letters);
}

fn tokenize_space(letters: &[char]) -> usize {
    let result = letters.iter().position(|char| match char {
        char if is_whitespace(char) => false,
        _ => true,
    });

    return unwrap_position(result, letters);
}

fn unwrap_position(result: Option<usize>, letters: &[char]) -> usize {
    let position = match result {
        Some(value) => value,
        None => letters.iter().count(),
    };

    if position == 0 {
        return position;
    }

    return position - 1;
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
                token_type: TokenType::Space,
                start: 0,
                end: 3
            }]
        )
    }

    #[test]
    fn it_tokenizes_word() {
        assert_eq!(
            tokenizer("ab"),
            [Token {
                token_type: TokenType::Word,
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
                    token_type: TokenType::Word,
                    start: 0,
                    end: 1
                },
                Token {
                    token_type: TokenType::Word,
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
                    token_type: TokenType::Word,
                    start: 0,
                    end: 0
                },
                Token {
                    token_type: TokenType::Space,
                    start: 1,
                    end: 3
                },
                Token {
                    token_type: TokenType::Word,
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
                    token_type: TokenType::Symbol,
                    start: 0,
                    end: 0
                },
                Token {
                    token_type: TokenType::Symbol,
                    start: 1,
                    end: 1
                },
                Token {
                    token_type: TokenType::Symbol,
                    start: 2,
                    end: 2
                },
                Token {
                    token_type: TokenType::Symbol,
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
                    token_type: TokenType::Word,
                    start: 0,
                    end: 1
                },
                Token {
                    token_type: TokenType::Word,
                    start: 2,
                    end: 3
                },
                Token {
                    token_type: TokenType::Word,
                    start: 4,
                    end: 5
                },
                Token {
                    token_type: TokenType::Word,
                    start: 6,
                    end: 7
                },
                Token {
                    token_type: TokenType::Word,
                    start: 8,
                    end: 9
                },
                Token {
                    token_type: TokenType::String,
                    start: 10,
                    end: 11
                }
            ]
        )
    }
}
