use crate::Declaration;
use crate::Node;
use crate::NodeBuilder;
use crate::Statement;
use crate::rule_word::RuleWord;

use serde::Deserialize;
use serde::Serialize;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;
use postcss_rs_tokenizer::Tokens;

/// An empty struct used for creating a css rule.
///
/// Rules are on of the basic parts of css and consist of selectors as well as a list of property
/// declarations.
///
/// ```css
/// .rule {
///   background: white;
/// }
/// ```
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub selector: String,
}

impl Rule {
    pub fn create(token: Token, tokenizer: &mut Tokenizer, spaces: &[Token]) -> Node {
        let mut builder = NodeBuilder::new(&token, tokenizer, spaces);
        let selector = Selector::create(token, tokenizer, &mut builder);

        builder.ensure_empty_before();
        builder.ensure_empty_between();
        builder.ensure_empty_after();
        builder.rule(Self::new(selector));

        let mut spaces = Tokens::new();

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Word => {
                    // Combine consecutive Word tokens for property names with escape sequences
                    let prop = RuleWord::collect_and_combine(token, tokenizer);

                    // Nested rule: word followed by opening brace (possibly with spaces)
                    if Self::peek_for_nested_rule(tokenizer) {
                        let nested_rule = Self::create(prop, tokenizer, &spaces);
                        // Nested rules don't end with semicolons, so reset the terminator
                        let terminator = crate::StatementTerminator::new();
                        builder.terminator(&terminator);
                        builder.append(nested_rule);
                        spaces.clear();
                        continue;
                    }

                    // Declaration: word followed by colon or other tokens
                    let (child, terminator) = Declaration::create(prop, tokenizer, &spaces);
                    builder.terminator(&terminator);
                    builder.append(child);
                    spaces.clear();
                }

                Kind::AtWord => {
                    let (mut node, terminator) = crate::AtRule::create(token, tokenizer, &spaces);

                    // For at-rules inside rules that end with CloseCurly, we need to:
                    // 1. Set the end source position (before trailing spaces)
                    // 2. Consume trailing spaces and add to 'between'
                    if let Some(Kind::CloseCurly) = terminator.ended_by {
                        if node.source.end.is_none() {
                            // Save end position before consuming spaces
                            node.source.end = Some(crate::Source::create_end(tokenizer));

                            // Consume trailing spaces and add to 'between'
                            let mut peekable = tokenizer.clone().into_peekable();
                            while let Some(peeked) = peekable.peek() {
                                match peeked.kind {
                                    Kind::Space => {
                                        peekable.next(); // Advance peekable
                                        let space_token = tokenizer.next().unwrap();
                                        node.raws.between.push(&space_token);
                                    }
                                    _ => break,
                                }
                            }
                        }
                    }

                    builder.terminator(&terminator);
                    builder.append(node);
                    spaces.clear();
                }

                Kind::Comment => {
                    let comment = Self::create_comment(token, &mut *tokenizer, &spaces);
                    builder.append(comment);
                    spaces.clear();
                }

                Kind::Space => spaces.push(token),
                Kind::Semicolon => spaces.push(token),
                Kind::CloseCurly => break,
                _ => break,
            };
        }

        // After the rule body ends, check for trailing semicolons (ownSemicolon in PostCSS)
        // This handles cases like: a{b:c}; or a{b:c} ;
        // Column always points at the semicolon
        // Offset behavior:
        //   - If ownSemicolon is just ";": offset points AT the newline
        //   - If ownSemicolon has space(s) " ;": offset points AFTER the newline
        let mut own_semicolon_found = false;
        let mut semicolon_col: Option<usize> = None;
        let mut has_space_before_semicolon = false;

        {
            let mut peekable = tokenizer.clone().into_peekable();
            let mut temp_count = 0;

            // Look ahead for trailing spaces and semicolon
            loop {
                if let Some(peeked) = peekable.peek() {
                    match peeked.kind {
                        Kind::Space if peeked.value != "\n" && peeked.value != "\r\n" => {
                            has_space_before_semicolon = true;
                            peekable.next();
                            temp_count += 1;
                        }
                        Kind::Semicolon => {
                            // Save the semicolon's column
                            semicolon_col = Some(std::cmp::max(1, peeked.column));
                            own_semicolon_found = true;
                            temp_count += 1;
                            break;
                        }
                        _ => break,
                    }
                } else {
                    break;
                }
            }

            // If we found a semicolon, consume the tokens
            if own_semicolon_found {
                for _ in 0..temp_count {
                    tokenizer.next();
                }
            }
        }

        // Calculate end offset and line
        let end_line = tokenizer.lines.value();
        let mut end_offset = tokenizer.index;

        // If there was a space before semicolon, offset should point AFTER the newline
        // But we don't consume the newline - it belongs to the next node's 'before'
        if own_semicolon_found && has_space_before_semicolon {
            let mut peekable2 = tokenizer.clone().into_peekable();
            if let Some(next_token) = peekable2.peek() {
                if next_token.kind == Kind::Space && (next_token.value == "\n" || next_token.value == "\r\n") {
                    // Don't consume the newline, just calculate offset as if we did
                    end_offset = tokenizer.index + next_token.value.len();
                }
            }
        }

        // If we found an ownSemicolon, build with special end position
        if own_semicolon_found {
            if let Some(col) = semicolon_col {
                // Create end position with saved line (before newline consumption) and column at semicolon
                let end_pos = crate::Source {
                    offset: end_offset,
                    line: end_line,
                    column: col,
                };

                let mut node = builder.node;
                node.raws.after.push_tokens(&spaces);
                node.source.end = Some(end_pos);
                return node;
            }
        }

        return builder.build(tokenizer, &spaces);
    }

    fn new(selector: String) -> Self {
        return Self { selector };
    }

    fn create_comment(token: Token, tokenizer: &mut Tokenizer, spaces: &[Token]) -> Node {
        return crate::Comment::create(token, tokenizer, spaces);
    }

    /// Peek ahead to determine if the next non-space token indicates a nested rule
    fn peek_for_nested_rule(tokenizer: &Tokenizer) -> bool {
        let mut multi = tokenizer.clone().into_multipeek();

        // Skip spaces and look for the next significant token
        while let Some(token) = multi.peek() {
            match token.kind {
                Kind::Space => continue, // Found space, keep looking
                Kind::OpenCurly => return true, // Found opening brace - it's a nested rule
                Kind::Brackets if token.value.starts_with('{') => return true, // Brackets starting with '{'
                _ => return false, // Found anything else - it's a declaration
            }
        }

        false // No more tokens - it's a declaration
    }
}

struct Selector;

impl Selector {
    /// A string value representing an empty selector
    const EMPTY_SELECTOR: &'static str = "";

    /// Check if there are selector tokens ahead, skipping over spaces
    fn has_selector_tokens_ahead(tokenizer: &Tokenizer) -> bool {
        let mut multi = tokenizer.clone().into_multipeek();

        while let Some(token) = multi.peek() {
            match token.kind {
                Kind::Space => continue,
                Kind::Word
                | Kind::Colon
                | Kind::Brackets
                | Kind::OpenSquare
                | Kind::CloseSquare
                | Kind::String
                | Kind::Semicolon
                | Kind::OpenCurvy
                | Kind::CloseCurvy
                | Kind::Comment => return true,
                _ => return false,
            }
        }

        false
    }

    /// A builder struct used for creating a rule selector.
    pub fn create(token: Token, tokenizer: &mut Tokenizer, builder: &mut NodeBuilder) -> String {
        if token.kind == Kind::OpenCurly {
            builder.ensure_empty_after();
            builder.ensure_empty_between();

            return String::from(Self::EMPTY_SELECTOR);
        }

        let mut statement = Statement::new();
        statement.push(token);

        while let Some(token) = tokenizer.next() {
            match token.kind {
                Kind::Brackets => statement.push(token),
                Kind::Colon => statement.push(token),
                Kind::Word => statement.push(token),
                Kind::OpenSquare => statement.push(token),
                Kind::CloseSquare => statement.push(token),
                Kind::String => statement.push(token),
                Kind::Semicolon => statement.push(token),
                Kind::OpenCurvy => statement.push(token),
                Kind::CloseCurvy => statement.push(token),
                Kind::Comment => {
                    // Check if there are more selector tokens after this comment
                    let has_more_selector_tokens = Self::has_selector_tokens_ahead(tokenizer);

                    // If no more selector tokens, this comment belongs in 'between'
                    if !has_more_selector_tokens {
                        builder.between(&token);
                        continue;
                    }

                    // Check if comment contains braces that would be significant
                    let has_braces = token.value.contains('{') || token.value.contains('}');

                    // Comments without braces are part of the selector as-is
                    if !has_braces {
                        statement.push(token);
                        continue;
                    }

                    // Comment with braces: keep in raw only, add spaces to both
                    statement.push_raw_only(token);

                    // Consume following spaces and push to both raw and clean
                    let mut peekable = tokenizer.clone().into_peekable();
                    while let Some(next) = peekable.peek() {
                        match next.kind {
                            Kind::Space => {
                                peekable.next(); // Advance peekable
                                let space = tokenizer.next().unwrap();
                                statement.push(space);
                            }
                            _ => break,
                        }
                    }
                }
                Kind::Space => builder.between(&token),
                Kind::OpenCurly => break,
                _ => break,
            };
        }

        let raw_selector = statement.to_string();
        let clean_selector = statement.to_string_clean();

        // If raw and clean differ, set the raw selector in builder
        if raw_selector != clean_selector {
            builder.selector_raw(&clean_selector, &raw_selector);
        }

        return clean_selector;
    }
}
