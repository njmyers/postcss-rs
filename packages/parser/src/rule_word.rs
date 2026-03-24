use postcss_rs_tokenizer::{Kind, Token, Tokenizer};

/// Handles combining consecutive Word tokens into a single token.
///
/// This is necessary for property names that contain escape sequences,
/// where a hex escape like `\62 olor` gets tokenized as separate Word tokens
/// (`\62 ` and `olor`) but should be treated as a single property name (`bolor`).
pub struct RuleWord;

impl RuleWord {
    /// Collect consecutive Word and Brackets tokens before a colon to build property names.
    ///
    /// This handles property names with escape sequences (e.g., `\62 olor` → `bolor`)
    /// and special characters (e.g., `$(var)-size`).
    ///
    /// Collects tokens until hitting a colon (with optional spaces before it).
    /// If there's only one Word token before the colon, it's returned as-is.
    /// If there are multiple consecutive Word/Brackets tokens, they're combined into a single
    /// token with concatenated values and a span from the first to the last token.
    ///
    /// # Arguments
    /// * `first_token` - The first Word token (must be Kind::Word)
    /// * `tokenizer` - The tokenizer to read additional tokens from
    ///
    /// # Returns
    /// A combined Token representing the property name
    pub fn collect_and_combine(first_token: Token, tokenizer: &mut Tokenizer) -> Token {
        let mut tokens = vec![first_token];

        // Collect tokens that are part of the property name (Word and Brackets)
        // Stop when we encounter:
        // 1. A Word token starting with ':' (declaration value, not property name)
        // 2. A colon directly (Kind::Colon)
        // 3. Anything else (space, open brace, etc.)
        while let Some(next) = tokenizer.peek() {
            match next.kind {
                Kind::Word => {
                    // Check if this Word starts with ':' - if so, it's the start of the value
                    // not part of the property name (e.g., ":black" in "color:black")
                    if next.value.starts_with(':') {
                        break;
                    }
                    tokens.push(tokenizer.next().unwrap());
                }
                Kind::Brackets => {
                    tokens.push(tokenizer.next().unwrap());
                }
                _ => {
                    // Not part of property name - stop collecting
                    break;
                }
            }
        }

        // If only one token, return it as-is
        if tokens.len() == 1 {
            return tokens.into_iter().next().unwrap();
        }

        // Combine multiple tokens
        Self::combine_tokens(tokens)
    }

    /// Combine multiple Word tokens into a single token.
    fn combine_tokens(tokens: Vec<Token>) -> Token {
        let mut combined_value = String::new();
        for t in &tokens {
            combined_value.push_str(&t.value);
        }

        Token {
            kind: Kind::Word,
            start: tokens[0].start,
            end: tokens.last().unwrap().end,
            value: combined_value,
            column: tokens[0].column,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_word_returns_single_token_unchanged() {
        let css = "color";
        let mut tokenizer = Tokenizer::new(css);
        let first_token = tokenizer.next().unwrap();

        let result = RuleWord::collect_and_combine(first_token.clone(), &mut tokenizer);

        assert_eq!(result.value, "color");
        assert_eq!(result.start, first_token.start);
        assert_eq!(result.end, first_token.end);
    }

    #[test]
    fn rule_word_combines_consecutive_word_tokens() {
        // Simulate tokenization of `\62 olor` which produces two Word tokens
        let css = r"\62 olor";
        let mut tokenizer = Tokenizer::new(css);

        let first_token = tokenizer.next().unwrap();
        assert_eq!(first_token.kind, Kind::Word);

        let result = RuleWord::collect_and_combine(first_token, &mut tokenizer);

        // Should combine both Word tokens
        assert_eq!(result.value, "\\62 olor");
        assert_eq!(result.kind, Kind::Word);
    }

    #[test]
    fn rule_word_stops_at_non_word_token() {
        let css = "color: red";
        let mut tokenizer = Tokenizer::new(css);

        let first_token = tokenizer.next().unwrap();
        assert_eq!(first_token.kind, Kind::Word);
        assert_eq!(first_token.value, "color");

        let result = RuleWord::collect_and_combine(first_token.clone(), &mut tokenizer);

        // Should only include "color", not the colon
        assert_eq!(result.value, "color");

        // Next token should be the colon
        let next = tokenizer.next().unwrap();
        assert_eq!(next.kind, Kind::Colon);
    }

    #[test]
    fn rule_word_combines_span_correctly() {
        let css = r"\62 olor";
        let mut tokenizer = Tokenizer::new(css);

        let first_token = tokenizer.next().unwrap();
        let first_start = first_token.start;

        let result = RuleWord::collect_and_combine(first_token, &mut tokenizer);

        // Start should be from first token
        assert_eq!(result.start, first_start);
        // End should be from last token (after "olor")
        assert!(result.end > first_start);
    }
}
