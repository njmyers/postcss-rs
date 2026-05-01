use std::borrow::Cow;

use postcss_rs_tokenizer::Token;

use serde::Deserialize;
use serde::Serialize;

/// A struct representing the raws value from the post css grammar
///
/// For specification please see postcss api documentation <https://postcss.org/api/#node-raws>
#[derive(PartialEq, Eq, Debug, Deserialize, Serialize, Clone, Default)]
#[serde(bound(deserialize = "'de: 'src"))]
pub struct Raws<'src> {
    #[serde(default, skip_serializing_if = "Raw::skip")]
    pub before: Raw<'src>,
    #[serde(default, skip_serializing_if = "Raw::skip")]
    pub between: Raw<'src>,
    #[serde(default, rename = "afterName", skip_serializing_if = "Raw::skip")]
    pub after_name: Raw<'src>,
    #[serde(default, skip_serializing_if = "Semicolon::skip")]
    pub semicolon: Semicolon,
    #[serde(default, skip_serializing_if = "Raw::skip")]
    pub after: Raw<'src>,
    #[serde(default, skip_serializing_if = "Raw::skip")]
    pub left: Raw<'src>,
    #[serde(default, skip_serializing_if = "Raw::skip")]
    pub right: Raw<'src>,
    #[serde(default, skip_serializing_if = "Raw::skip")]
    pub important: Raw<'src>,
    #[serde(default, rename = "ownSemicolon", skip_serializing_if = "Raw::skip")]
    pub own_semicolon: Raw<'src>,
    #[serde(default, skip_serializing_if = "ValueRaw::skip")]
    pub value: ValueRaw<'src>,
    #[serde(default, skip_serializing_if = "ValueRaw::skip")]
    pub selector: ValueRaw<'src>,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Default)]
pub struct Raw<'src>(#[serde(borrow)] Option<Cow<'src, str>>);

impl<'src> Raw<'src> {
    /// Return a new raw struct, borrowing from the source CSS when possible.
    ///
    /// Pass a `&'src str` (from CSS source) to get zero-copy `Cow::Borrowed`,
    /// or a `String` / `Cow` for owned data.
    pub fn new(raw: impl Into<Cow<'src, str>>) -> Self {
        Self(Some(raw.into()))
    }

    /// Push a new token value to a raw value.
    ///
    /// When the raw is currently empty, borrows the token value directly
    /// (zero-copy). When appending to an existing value, converts to owned.
    ///
    /// # Example
    ///
    /// ```
    /// use postcss_rs_parser::Raw;
    /// use postcss_rs_tokenizer::Token;
    /// use postcss_rs_tokenizer::Kind;
    ///
    /// let mut raw = Raw::default();
    /// raw.push(&Token {
    ///     kind: Kind::Space,
    ///     start: 0,
    ///     end: 1,
    ///     value: "\n",
    ///     column: 0,
    /// });
    ///
    /// assert_eq!(&raw, &"\n");
    /// ```
    pub fn push(&mut self, token: &Token<'src>) {
        match &mut self.0 {
            Some(cow) => cow.to_mut().push_str(token.value),
            None => self.0 = Some(Cow::Borrowed(token.value)),
        }
    }

    /// Push a new string slice to a raw value.
    ///
    /// # Example
    ///
    /// ```
    /// use postcss_rs_parser::Raw;
    ///
    /// let mut raw = Raw::default();
    /// raw.push_str("\n");
    ///
    /// assert_eq!(&raw, &"\n");
    /// ```
    pub fn push_str(&mut self, raws: &str) {
        match &mut self.0 {
            Some(cow) => cow.to_mut().push_str(raws),
            None => self.0 = Some(Cow::Owned(raws.to_string())),
        };
    }

    /// Push tokens, or ensure `Some("")` when the slice is empty.
    ///
    /// When the slice has exactly one token and the raw is currently empty,
    /// borrows the token value directly (zero-copy). Uses `Cow::Borrowed("")`
    /// for the empty case to avoid even a zero-capacity `String` allocation.
    pub fn push_tokens_or_empty(&mut self, tokens: &[Token<'src>]) {
        if tokens.is_empty() {
            if self.0.is_none() {
                self.0 = Some(Cow::Borrowed(""));
            }
            return;
        }
        self.push_tokens(tokens);
    }

    pub fn push_tokens(&mut self, tokens: &[Token<'src>]) {
        if tokens.is_empty() {
            return;
        }
        // Single token: borrow directly when possible.
        if tokens.len() == 1 {
            self.push(&tokens[0]);
            return;
        }
        // Multiple tokens: pre-compute total length, allocate once.
        let total: usize = tokens.iter().map(|t| t.value.len()).sum();
        match &mut self.0 {
            Some(cow) => {
                let s = cow.to_mut();
                s.reserve(total);
                for token in tokens {
                    s.push_str(token.value);
                }
            }
            None => {
                let mut s = String::with_capacity(total);
                for token in tokens {
                    s.push_str(token.value);
                }
                self.0 = Some(Cow::Owned(s));
            }
        }
    }

    /// Determine whether to skip serialization for a raw value
    ///
    /// # Example
    ///
    /// ```
    /// use postcss_rs_parser::Raw;
    ///
    /// let raw = Raw::default();
    /// assert_eq!(raw.skip(), true);
    /// ```
    pub fn skip(&self) -> bool {
        self.0.is_none()
    }

    /// Returns the inner string slice, or `None` if unset.
    pub fn as_deref(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

impl PartialEq<&str> for Raw<'_> {
    fn eq(&self, other: &&str) -> bool {
        match &self.0 {
            Some(cow) => cow.as_ref() == *other,
            None => false,
        }
    }
}

/// A struct representing a semicolon value in the postcss parser grammar.
///
/// A semicolon can either be `true`, `false`, or non-existent. Since this is an artifact of the
/// original JavaScript representation this struct allows for better serialization and
/// deserialization to match those artifacts.
#[derive(PartialEq, Eq, Debug, Deserialize, Serialize, Clone, Default)]
pub struct Semicolon(Option<bool>);

impl Semicolon {
    /// Return a new semicolon struct
    pub fn new(semicolon: bool) -> Self {
        Self(Some(semicolon))
    }

    /// Determine whether to skip serialization for a semicolon value
    ///
    /// # Example
    ///
    /// ```
    /// use postcss_rs_parser::Semicolon;
    ///
    /// let semi = Semicolon::default();
    /// assert_eq!(semi.skip(), true);
    /// ```
    pub fn skip(&self) -> bool {
        self.0.is_none()
    }

    /// Returns the inner `Option<bool>` value.
    pub fn value(&self) -> Option<bool> {
        self.0
    }

    /// Set the semicolon value to true
    ///
    /// # Example
    ///
    /// ```
    /// use postcss_rs_parser::Semicolon;
    ///
    /// let mut semi = Semicolon::default();
    /// semi.set_true();
    ///
    /// assert!(semi.value().unwrap());
    /// ```
    pub fn set_true(&mut self) {
        self.0 = Some(true);
    }

    /// Set the semicolon value to false
    ///
    /// # Example
    ///
    /// ```
    /// use postcss_rs_parser::Semicolon;
    ///
    /// let mut semi = Semicolon::default();
    /// semi.set_false();
    ///
    /// assert!(!semi.value().unwrap());
    /// ```
    pub fn set_false(&mut self) {
        self.0 = Some(false);
    }
}

/// A struct representing value raws with both the raw (with comments) and clean (without comments) versions
#[derive(PartialEq, Eq, Debug, Deserialize, Serialize, Clone, Default)]
pub struct ValueRaw<'src> {
    /// The comment-stripped (clean) value, e.g. `"red"`.
    #[serde(borrow, skip_serializing_if = "Option::is_none")]
    pub value: Option<Cow<'src, str>>,
    /// The raw value preserving inline comments, e.g. `"red /* note */"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<Cow<'src, str>>,
}

impl<'src> ValueRaw<'src> {
    /// Creates a new `ValueRaw` with both a clean `value` and a `raw` string.
    pub fn new(value: impl Into<Cow<'src, str>>, raw: impl Into<Cow<'src, str>>) -> Self {
        Self {
            value: Some(value.into()),
            raw: Some(raw.into()),
        }
    }

    /// Returns `true` when both `value` and `raw` are absent (skip serialization).
    pub fn skip(&self) -> bool {
        self.value.is_none() && self.raw.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcss_rs_tokenizer::{Kind, Token};

    fn space_token<'a>(value: &'a str) -> Token<'a> {
        Token {
            kind: Kind::Space,
            value,
            start: 0,
            end: value.len(),
            column: 1,
        }
    }

    // ---- Raw tests ----

    #[test]
    fn test_raw_default_is_empty() {
        let raw = Raw::default();
        assert!(raw.skip());
        assert!(raw.0.is_none());
    }

    #[test]
    fn test_raw_push_str() {
        let mut raw = Raw::default();
        raw.push_str("hello");
        assert_eq!(&raw, &"hello");
        assert!(!raw.skip());
    }

    #[test]
    fn test_raw_push_appends() {
        let mut raw = Raw::default();
        raw.push_str("hello");
        raw.push_str(" world");
        assert_eq!(&raw, &"hello world");
    }

    #[test]
    fn test_raw_push_token() {
        let mut raw = Raw::default();
        raw.push(&space_token("\n"));
        assert_eq!(&raw, &"\n");
    }

    #[test]
    fn test_raw_push_tokens() {
        let mut raw = Raw::default();
        raw.push_tokens(&[space_token(" "), space_token("\n")]);
        assert_eq!(&raw, &" \n");
    }

    #[test]
    fn test_raw_new() {
        let raw = Raw::new("test");
        assert_eq!(&raw, &"test");
        assert!(!raw.skip());
    }

    #[test]
    fn test_raw_single_token_borrows() {
        let mut raw = Raw::default();
        raw.push(&space_token("\n"));
        // Single token push on empty Raw should produce Cow::Borrowed
        assert!(matches!(raw.0, Some(Cow::Borrowed(_))));
    }

    #[test]
    fn test_raw_push_tokens_or_empty_borrows_empty() {
        let mut raw = Raw::default();
        raw.push_tokens_or_empty(&[]);
        // Empty push should produce Cow::Borrowed("")
        assert!(matches!(raw.0, Some(Cow::Borrowed(""))));
    }

    #[test]
    fn test_raw_push_tokens_single_borrows() {
        let mut raw = Raw::default();
        raw.push_tokens(&[space_token(" ")]);
        // Single token should borrow
        assert!(matches!(raw.0, Some(Cow::Borrowed(_))));
    }

    // ---- Semicolon tests ----

    #[test]
    fn test_semicolon_default_skips() {
        let s = Semicolon::default();
        assert!(s.skip());
        assert!(s.0.is_none());
    }

    #[test]
    fn test_semicolon_new_true() {
        let s = Semicolon::new(true);
        assert!(!s.skip());
        assert_eq!(s.value(), Some(true));
    }

    #[test]
    fn test_semicolon_set_true() {
        let mut s = Semicolon::default();
        s.set_true();
        assert_eq!(s.value(), Some(true));
    }

    #[test]
    fn test_semicolon_set_false() {
        let mut s = Semicolon::default();
        s.set_false();
        assert_eq!(s.value(), Some(false));
    }

    // ---- ValueRaw tests ----

    #[test]
    fn test_value_raw_default_skips() {
        let vr = ValueRaw::default();
        assert!(vr.skip());
    }

    #[test]
    fn test_value_raw_new() {
        let vr = ValueRaw::new("red", "red /* comment */");
        assert!(!vr.skip());
        assert_eq!(vr.value.as_deref(), Some("red"));
        assert_eq!(vr.raw.as_deref(), Some("red /* comment */"));
    }
}
