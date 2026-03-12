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

        // Phase 1: Process tokens before and including the colon
        // All spaces and comments before the colon go to 'between'
        // Returns: Optional value token if we split a Word starting with ':'
        let first_value_token: Option<Token> = {
            let mut result = None;
            for token in tokenizer.by_ref() {
                match token.kind {
                    Kind::Colon => {
                        builder.between(&token);
                        break; // Move to value processing phase
                    }
                    Kind::Word if token.value.starts_with(':') => {
                        // Word starting with ':' (e.g., ":black" in "color:black")
                        // Split it: first char is the colon, rest is the value
                        let (colon_token, value_token) = token.split(1, Kind::Colon, Kind::Word);
                        builder.between(&colon_token);
                        result = value_token; // Move to value processing phase
                        break;
                    }
                    Kind::Space | Kind::Comment => {
                        builder.between(&token);
                    }
                    _ => {
                        // Unexpected token before colon - this shouldn't happen in well-formed CSS
                        // but we'll add it to between and continue
                        builder.between(&token);
                    }
                }
            }
            result
        };

        // Phase 2: Process the value tokens
        let mut statement = Statement::new();
        let mut terminator = StatementTerminator::new();
        let mut pending_spaces: Vec<Token> = Vec::new(); // Track spaces between colon and first non-space token
        let mut important_raw: Option<String> = None;

        // If we have a first value token (from splitting a Word starting with ':'),
        // add it to the statement first
        if let Some(value_token) = first_value_token {
            statement.push(value_token);
        }

        // Create peekable once outside the loop for efficiency
        let mut peekable = tokenizer.clone().into_peekable();

        while let Some(peeked) = peekable.peek() {
            // Check if this token should end value processing without being consumed
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
                    if matches!(Self::peek_second_kind(tokenizer), Some(Kind::Comment) | Some(Kind::CloseCurly)) {
                        break;
                    }
                }

                // Space directly before CloseCurly (trailing space) ends declaration
                Kind::Space if !statement.is_open() => {
                    if Self::peek_second_kind(tokenizer) == Some(Kind::CloseCurly) {
                        terminator.set_ended_by(Kind::CloseCurly);
                        break;
                    }
                }
                _ => {}
            };

            // Consume the token from both peekable and original tokenizer
            peekable.next();
            let token = tokenizer.next().unwrap();

            terminator.process(&token);

            match token.kind {
                Kind::OpenCurly => statement.open(token),
                Kind::CloseCurly if statement.is_open() => statement.close(token),

                // Semicolon ends declaration and is consumed
                Kind::Semicolon if !statement.is_open() => {
                    terminator.set_ended_by(Kind::Semicolon);
                    break;
                }

                // Comment handling in value
                Kind::Comment => {
                    let before_content = statement.values_clean.is_empty() && !statement.is_open();
                    let next_is_fn = Self::next_is_open_bracket(tokenizer);
                    match (before_content, next_is_fn) {
                        (true, true) => {
                            // Comment before function - move pending to between, keep comment
                            for space in pending_spaces.drain(..) {
                                builder.between(&space);
                            }
                            statement.push(token);
                        }
                        (true, false) => {
                            // Comment before content - flush pending to raw, strip comment
                            for space in pending_spaces.drain(..) {
                                statement.push(space);
                            }
                            statement.push_raw_only(token);
                        }
                        (false, true) => statement.push(token),
                        (false, false) => statement.push_raw_only(token),
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
                            for space in pending_spaces.drain(..) {
                                builder.between(&space);
                            }
                            statement.push(token);
                        }
                    }
                }

                _ => {
                    // Non-space token - flush any pending spaces to between (they're before actual content)
                    for space in pending_spaces.drain(..) {
                        builder.between(&space);
                    }
                    statement.push(token);
                }
            }
        }

        // Handle trailing space before CloseCurly
        // If we broke due to CloseCurly, there may be a trailing space to handle
        if terminator.ended_by == Some(Kind::CloseCurly) {
            let mut peekable = tokenizer.clone().into_peekable();
            if let Some(space_token) = peekable.peek() {
                if space_token.kind == Kind::Space {
                    // Add the trailing space to pending_spaces for the value
                    pending_spaces.push(space_token.clone());
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

        // Get raw value first (preserves all spacing and comments)
        let raw_value = statement.to_string();

        // Use clean value (without comments) as the main value
        let clean_value_base = statement.to_string_clean();

        // Trim trailing whitespace only when:
        // 1. Comments were removed (raw != clean)
        // 2. Declaration ends with semicolon
        // 3. The value has actual content (not just whitespace)
        // This handles cases like "a/* comment */ b  " where removing the comment leaves trailing spaces
        // But preserves whitespace-only values like "/**/ " which should become " "
        let clean_value = if raw_value != clean_value_base
            && terminator.ended_by == Some(Kind::Semicolon)
            && !clean_value_base.trim().is_empty()
        {
            clean_value_base.trim_end().to_string()
        } else {
            clean_value_base
        };

        let important = important_raw.as_ref().map(|_| true);
        let declaration = Self::new(&actual_prop, &clean_value, important);
        builder.declaration(declaration);

        // If raw differs from clean (inline comments were stripped or whitespace differs), store both in raws.value
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
            let mut peekable = tokenizer.clone().into_peekable();
            if let Some(space_token) = peekable.peek() {
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

        // Create multipeek once and reuse it for efficiency
        let mut multi = tokenizer.clone().into_multipeek();

        while let Some(next) = multi.peek() {
            peek_count += 1;

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

        (ImportantResult::NotImportant, 0)
    }

    /// Build the final important raw string, including trailing space if present
    fn build_important_raw(base: &str, tokenizer: &mut Tokenizer) -> String {
        let mut raw = base.to_string();

        let mut peekable = tokenizer.clone().into_peekable();
        if let Some(trailing) = peekable.peek() {
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

    /// Return the kind of the token after the next one (skips one token ahead).
    /// Used to look past the current peeked token to detect terminators.
    fn peek_second_kind(tokenizer: &Tokenizer) -> Option<Kind> {
        let mut multi = tokenizer.clone().into_multipeek();
        multi.peek(); // skip current token
        multi.peek().map(|t| t.kind)
    }

    /// Check if the next token is an opening bracket `(`.
    fn next_is_open_bracket(tokenizer: &Tokenizer) -> bool {
        let mut peekable = tokenizer.clone().into_peekable();
        matches!(peekable.peek(), Some(t) if t.kind == Kind::Brackets && t.value.starts_with('('))
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
