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

        // Handle IE hack prefixes (* and _) by extracting them from property name
        // and adding them to the 'before' raw field
        let (actual_prop, ie_hack_prefix) = Self::extract_ie_hack_prefix(&prop.value);
        if let Some(prefix) = ie_hack_prefix {
            builder.append_to_before(&prefix);
        }

        let mut statement = Statement::new();
        let mut terminator = StatementTerminator::new();
        let mut pending_spaces: Vec<Token> = Vec::new(); // Track spaces between colon and first non-space token
        let mut important_raw: Option<String> = None;
        let mut seen_colon = false; // Track whether we've encountered the colon

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
                Kind::Colon if !statement.is_open() => {
                    seen_colon = true;
                    builder.between(&token);
                }

                // Comment handling
                Kind::Comment => {
                    // Comments before the colon should go to 'between'
                    match seen_colon {
                        false => {
                            // Comment before colon - add to between
                            builder.between(&token);
                        }
                        true => {
                            // Comment after colon - handle as part of value
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
                    }
                }

                Kind::Space => {
                    // Spaces before the colon should go to 'between'
                    match seen_colon {
                        false => {
                            // Space before colon - add to between
                            builder.between(&token);
                        }
                        true => {
                            // Space after colon - use pending logic
                            // Check if clean value is empty (ignore raw-only tokens like stripped comments)
                            if statement.values_clean.is_empty() && !statement.is_open() {
                                // Space before any value content - hold in pending
                                pending_spaces.push(token);
                            } else {
                                // Space within value - add to statement
                                statement.push(token);
                            }
                        }
                    }
                }

                // Check for !important
                Kind::Word if token.value.contains('!') => {
                    // Word contains !, check if it's !important
                    let (result, bang_pos) = Self::check_important(&token.value, tokenizer);

                    match result {
                        ImportantResult::Found(raw) => {
                            // Found !important
                            // Collect trailing spaces from value (they should be part of important_raw)
                            let trailing_spaces = statement.collect_trailing_spaces();
                            // Add the part before ! to value (if any) using token slice
                            if bang_pos > 0 {
                                statement.push_token_slice(&token, 0, bang_pos);
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
        let declaration = Self::new(&actual_prop, &clean_value, important);
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
    /// Returns (ImportantResult, bang_position)
    fn check_important(word: &str, tokenizer: &mut Tokenizer) -> (ImportantResult, usize) {
        // Find the position of ! in the word
        let Some(bang_pos) = word.find('!') else {
            return (ImportantResult::NotImportant, 0);
        };

        let after_bang = &word[bang_pos + 1..]; // Skip the !

        // Check if everything after ! matches "important" (case-insensitive)
        if after_bang.eq_ignore_ascii_case("important") {
            let raw = Self::build_important_raw(&word[bang_pos..], tokenizer);
            return (ImportantResult::Found(raw), bang_pos);
        }

        // Not a prefix of "important", bail early
        if !after_bang.is_empty() && !Self::could_be_important_prefix(after_bang) {
            return (ImportantResult::NotImportant, 0);
        }

        // Collect subsequent tokens to build complete important pattern
        Self::collect_multitoken_important(&word[bang_pos..], after_bang, tokenizer, bang_pos)
    }

    /// Collect !important pattern that spans multiple tokens
    fn collect_multitoken_important(
        initial: &str,
        after_bang: &str,
        tokenizer: &mut Tokenizer,
        bang_pos: usize,
    ) -> (ImportantResult, usize) {
        let mut raw_parts = vec![initial.to_string()];
        let mut peek_count = 0;

        // Already complete if after_bang is "important"
        if after_bang.eq_ignore_ascii_case("important") {
            let raw = Self::build_important_raw(&raw_parts.join(""), tokenizer);
            return (ImportantResult::Found(raw), bang_pos);
        }

        loop {
            peek_count += 1;
            let Some(next) = tokenizer.peek_nth(peek_count) else {
                return (ImportantResult::NotImportant, 0);
            };

            match next.kind {
                Kind::Space | Kind::Comment => {
                    raw_parts.push(next.value.clone());
                    continue;
                }
                Kind::Word if next.value.eq_ignore_ascii_case("important") => {
                    raw_parts.push(next.value.clone());

                    // Consume all peeked tokens
                    for _ in 0..peek_count {
                        tokenizer.next();
                    }

                    let raw = Self::build_important_raw(&raw_parts.join(""), tokenizer);
                    return (ImportantResult::Found(raw), bang_pos);
                }
                _ => return (ImportantResult::NotImportant, 0),
            }
        }
    }

    /// Build the final important raw string, including trailing space if present
    fn build_important_raw(base: &str, tokenizer: &mut Tokenizer) -> String {
        let mut raw = base.to_string();

        if let Some(trailing) = tokenizer.peek() {
            if trailing.kind == Kind::Space {
                raw.push_str(&trailing.value);
                tokenizer.next();
            }
        }

        raw
    }

    /// Check if a string could be a prefix of "important" (case-insensitive)
    fn could_be_important_prefix(s: &str) -> bool {
        if s.is_empty() {
            return true;
        }
        "important".to_lowercase().starts_with(&s.to_lowercase())
    }

    /// Extract IE hack prefix (* or _) from property name
    /// Returns (actual_property_name, optional_ie_hack_prefix)
    fn extract_ie_hack_prefix(prop: &str) -> (String, Option<String>) {
        match prop.chars().next() {
            Some('*') => (prop[1..].to_string(), Some("*".to_string())),
            Some('_') => (prop[1..].to_string(), Some("_".to_string())),
            _ => (prop.to_string(), None),
        }
    }
}

enum ImportantResult {
    Found(String), // Raw important string
    NotImportant,
}
