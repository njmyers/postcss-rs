use crate::characters;
use crate::Node;
use crate::NodeBuilder;
use crate::Statement;
use crate::StatementTerminator;

use serde::Deserialize;
use serde::Serialize;

use postcss_rs_tokenizer::Kind;
use postcss_rs_tokenizer::Token;
use postcss_rs_tokenizer::Tokenizer;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Declaration {
    pub prop: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub important: Option<bool>,
    pub value: String,
}

impl Declaration {
    pub fn create(
        prop: Token,
        tokenizer: &mut Tokenizer,
        spaces: &[Token],
    ) -> (Node, StatementTerminator) {
        let mut builder = NodeBuilder::new(&prop, tokenizer, spaces);
        builder.ensure_empty_before();
        builder.ensure_empty_between();
        let mut statement = Statement::new();
        let mut terminator = StatementTerminator::new();
        let mut pending_spaces: Vec<Token> = Vec::new(); // Track spaces between colon and first non-space token
        let mut important_raw: Option<String> = None;

        loop {
            // Peek at next token to check for terminators
            if let Some(peeked) = tokenizer.peek() {
                match peeked.kind {
                    // CloseCurly ends declaration without being consumed (belongs to parent)
                    Kind::CloseCurly if !statement.is_open() => {
                        terminator.set_ended_by(Kind::CloseCurly);
                        break;
                    }
                    // Space with newline ends declaration when not open
                    Kind::Space if peeked.value == characters::NEWLINE && !statement.is_open() => break,

                    // Space with newline followed by comment or CloseCurly ends declaration
                    Kind::Space if peeked.value.starts_with(characters::NEWLINE) && !statement.is_open() => {
                        match tokenizer.peek_nth(2) {
                            Some(next) if next.kind == Kind::Comment || next.kind == Kind::CloseCurly => break,
                            _ => {} // Not before terminator, continue
                        }
                    }

                    // Check for trailing spaces before CloseCurly
                    // Any space directly before CloseCurly (when not in open brackets) is trailing
                    Kind::Space if !statement.is_open() => {
                        match tokenizer.peek_nth(2) {
                            Some(next) if next.kind == Kind::CloseCurly => {
                                // Space followed by CloseCurly - this is a trailing space
                                // We need to: 1) add it to value, 2) consume it, 3) use pre-consume position as end
                                // Break here to avoid consuming - will handle in post-loop
                                terminator.set_ended_by(Kind::CloseCurly);
                                break;
                            }
                            _ => {} // Not trailing, will be consumed normally
                        }
                    }
                    _ => {}
                }
            }

            // Consume the token
            let token = match tokenizer.next() {
                Some(t) => t,
                None => break,
            };

            terminator.process(&token);

            match token.kind {
                Kind::OpenCurly => statement.open(token),
                Kind::CloseCurly if statement.is_open() => statement.close(token),

                // Semicolon ends declaration and is consumed
                Kind::Semicolon if !statement.is_open() => {
                    terminator.set_ended_by(Kind::Semicolon);
                    break;
                }
                Kind::Colon if !statement.is_open() => builder.between(&token),

                // Comment handling
                Kind::Comment => {
                    // Check if clean value is empty (ignore raw-only tokens)
                    if statement.values_clean.is_empty() && !statement.is_open() {
                        // Comment before any real content - check if it's before a function call
                        match tokenizer.peek() {
                            Some(next) if next.kind == Kind::Brackets && next.value.starts_with('(') => {
                                // Comment before function - move pending to between, keep comment
                                if !pending_spaces.is_empty() {
                                    for space in pending_spaces.drain(..) {
                                        builder.between(&space);
                                    }
                                }
                                statement.push(token);
                            }
                            _ => {
                                // Regular comment - strip it but keep token order correct
                                // Add pending spaces to both raw and clean to maintain correct order
                                for space in pending_spaces.drain(..) {
                                    statement.push(space);
                                }
                                // Add comment to raw only (stripped from clean)
                                statement.push_raw_only(token);
                            }
                        }
                    } else {
                        // Comment inside value - check if it's before an opening paren
                        match tokenizer.peek() {
                            Some(next) if next.kind == Kind::Brackets && next.value.starts_with('(') => {
                                statement.push(token);
                            }
                            _ => {
                                // Regular inline comment - only add to raw, not clean
                                statement.push_raw_only(token);
                            }
                        }
                    }
                }

                Kind::Space => {
                    // Check if clean value is empty (ignore raw-only tokens like stripped comments)
                    if statement.values_clean.is_empty() && !statement.is_open() {
                        // Space before any value content - hold in pending
                        pending_spaces.push(token);
                    } else {
                        // Space within value - add to statement
                        statement.push(token);
                    }
                }

                // Check for !important
                Kind::Word if token.value.contains('!') => {
                    // Word contains !, check if it's !important
                    let (result, before_bang) = Self::check_important(&token.value, tokenizer);

                    match result {
                        ImportantResult::Found(raw) => {
                            // Found !important
                            // Collect trailing spaces from value (they should be part of important_raw)
                            let trailing_spaces = statement.collect_trailing_spaces();
                            // Add the part before ! to value (if any)
                            if !before_bang.is_empty() {
                                statement.push_value(&before_bang);
                            }
                            // Combine trailing spaces with the raw important
                            important_raw = Some(format!("{}{}", trailing_spaces, raw));
                        }
                        ImportantResult::NotImportant => {
                            // Contains ! but not important
                            if !pending_spaces.is_empty() {
                                for space in pending_spaces.drain(..) {
                                    builder.between(&space);
                                }
                            }
                            statement.push(token);
                        }
                    }
                }

                _ => {
                    // Non-space token - decide what to do with pending spaces
                    if !pending_spaces.is_empty() {
                        // Add pending spaces to between (they're before actual content)
                        for space in pending_spaces.drain(..) {
                            builder.between(&space);
                        }
                    }
                    statement.push(token);
                }
            }
        }

        // Handle trailing space before CloseCurly
        // If we broke due to CloseCurly, there may be a trailing space to handle
        if terminator.ended_by == Some(Kind::CloseCurly) {
            if let Some(space_token) = tokenizer.peek() {
                if space_token.kind == Kind::Space {
                    // Add the trailing space to pending_spaces for the value
                    pending_spaces.push(space_token);
                }
            }
        }

        // End of declaration - handle any remaining pending spaces
        if !pending_spaces.is_empty() {
            // Declaration ended with only spaces - they should be the value
            for space in pending_spaces.iter() {
                statement.push(space.clone());
            }
        }

        // Use clean value (without comments) as the main value
        let clean_value = statement.to_string_clean();
        let important = important_raw.as_ref().map(|_| true);
        let declaration = Self::new(&prop.value, &clean_value, important);
        builder.declaration(declaration);

        // If raw differs from clean (inline comments were stripped), store both in raws.value
        let raw_value = statement.to_string();
        if raw_value != clean_value {
            builder.value_raw(&clean_value, &raw_value);
        }

        // If important was found, store the raw important string
        if let Some(raw) = important_raw {
            builder.important_raw(&raw);
        }

        let node = builder.build(tokenizer, &[]);

        // Now consume the trailing space if present so the rule parser doesn't see it
        if terminator.ended_by == Some(Kind::CloseCurly) {
            if let Some(space_token) = tokenizer.peek() {
                if space_token.kind == Kind::Space {
                    tokenizer.next(); // Consume the space
                }
            }
        }

        return (node, terminator);
    }

    fn new(prop: &str, value: &str, important: Option<bool>) -> Self {
        return Self {
            prop: String::from(prop),
            important,
            value: String::from(value),
        };
    }

    /// Check if a word token contains !important pattern
    /// Returns (ImportantResult, text_before_bang)
    fn check_important(word: &str, tokenizer: &mut Tokenizer) -> (ImportantResult, String) {
        // Find the position of ! in the word
        let Some(bang_pos) = word.find('!') else {
            return (ImportantResult::NotImportant, String::new());
        };

        let before_bang = word[..bang_pos].to_string();
        let after_bang = &word[bang_pos + 1..]; // Skip the !

        // Check if everything after ! matches "important" (case-insensitive)
        if after_bang.eq_ignore_ascii_case("important") {
            // Complete !important in single token
            let mut raw = word[bang_pos..].to_string();

            // Check for trailing space after "important"
            if let Some(trailing) = tokenizer.peek() {
                if trailing.kind == Kind::Space {
                    raw.push_str(&trailing.value);
                    tokenizer.next(); // Consume the space
                }
            }

            return (ImportantResult::Found(raw), before_bang);
        }

        // Check if we have a partial match or need to look ahead
        if after_bang.is_empty() || Self::could_be_important_prefix(after_bang) {
            // Collect subsequent tokens to build complete important pattern
            let mut raw_parts = vec![word[bang_pos..].to_string()];
            let mut found = after_bang.eq_ignore_ascii_case("important");
            let mut peek_count = 0;

            while !found {
                peek_count += 1;
                let Some(next) = tokenizer.peek_nth(peek_count) else {
                    // No more tokens, not important
                    return (ImportantResult::NotImportant, String::new());
                };

                match next.kind {
                    Kind::Space | Kind::Comment => {
                        // Collect spaces and comments
                        raw_parts.push(next.value.clone());
                    }
                    Kind::Word => {
                        // Check if this word is "important" (case-insensitive)
                        if next.value.eq_ignore_ascii_case("important") {
                            raw_parts.push(next.value.clone());
                            found = true;
                            // Consume all the tokens we peeked
                            for _ in 0..peek_count {
                                tokenizer.next();
                            }
                            // Check for trailing space after "important"
                            if let Some(trailing) = tokenizer.peek() {
                                if trailing.kind == Kind::Space {
                                    raw_parts.push(trailing.value.clone());
                                    tokenizer.next(); // Consume the space
                                }
                            }
                            break;
                        } else {
                            // Not important
                            return (ImportantResult::NotImportant, String::new());
                        }
                    }
                    _ => {
                        // Hit a non-space/comment/word token, not important
                        return (ImportantResult::NotImportant, String::new());
                    }
                }
            }

            if found {
                return (ImportantResult::Found(raw_parts.join("")), before_bang);
            }
        }

        // Contains ! but not followed by important
        (ImportantResult::NotImportant, String::new())
    }

    /// Check if a string could be a prefix of "important" (case-insensitive)
    fn could_be_important_prefix(s: &str) -> bool {
        if s.is_empty() {
            return true;
        }
        "important".to_lowercase().starts_with(&s.to_lowercase())
    }
}

enum ImportantResult {
    Found(String), // Raw important string
    NotImportant,
}
