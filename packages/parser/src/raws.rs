use postcss_rs_tokenizer::Token;

use serde::Deserialize;
use serde::Serialize;

use std::ops::Deref;

/// A struct representing the raws value from the post css grammar
///
/// For specification please see postcss api documentation <https://postcss.org/api/#node-raws>
#[derive(PartialEq, Eq, Debug, Deserialize, Serialize, Clone)]
pub struct Raws {
    #[serde(default, skip_serializing_if = "Raw::skip")]
    pub before: Raw,
    #[serde(default, skip_serializing_if = "Raw::skip")]
    pub between: Raw,
    #[serde(default, rename = "afterName", skip_serializing_if = "Raw::skip")]
    pub after_name: Raw,
    #[serde(default, skip_serializing_if = "Semicolon::skip")]
    pub semicolon: Semicolon,
    #[serde(default, skip_serializing_if = "Raw::skip")]
    pub after: Raw,
    #[serde(default, skip_serializing_if = "Raw::skip")]
    pub left: Raw,
    #[serde(default, skip_serializing_if = "Raw::skip")]
    pub right: Raw,
    #[serde(default, skip_serializing_if = "Raw::skip")]
    pub important: Raw,
    #[serde(default, rename = "ownSemicolon", skip_serializing_if = "Raw::skip")]
    pub own_semicolon: Raw,
    #[serde(default, skip_serializing_if = "ValueRaw::skip")]
    pub value: ValueRaw,
    #[serde(default, skip_serializing_if = "ValueRaw::skip")]
    pub selector: ValueRaw,
}

impl Default for Raws {
    fn default() -> Self {
        return Self {
            before: Raw::default(),
            between: Raw::default(),
            after_name: Raw::default(),
            semicolon: Semicolon::default(),
            after: Raw::default(),
            left: Raw::default(),
            right: Raw::default(),
            important: Raw::default(),
            own_semicolon: Raw::default(),
            value: ValueRaw::default(),
            selector: ValueRaw::default(),
        };
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Raw(Option<String>);

impl Raw {
    /// Return a new raw struct
    pub fn new(raw: &str) -> Self {
        return Self(Some(String::from(raw)));
    }

    /// Push a new token value to a raw value
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
    ///     value: "\n".into(),
    ///     column: 0,
    /// });
    ///
    /// assert_eq!(&raw, &"\n");
    /// ```
    pub fn push(&mut self, token: &Token) {
        self.push_str(&token.value);
    }

    /// Push a new string slice to a raw value
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
        match self.0 {
            Some(ref mut value) => value.push_str(raws),
            None => self.0 = Some(String::from(raws)),
        };
    }

    pub fn push_tokens(&mut self, tokens: &[Token]) {
        for token in tokens {
            self.push(token);
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
        return self.0.is_none();
    }
}

impl Default for Raw {
    fn default() -> Self {
        return Self(None);
    }
}

impl Deref for Raw {
    type Target = Option<String>;

    fn deref(&self) -> &Option<String> {
        return &self.0;
    }
}

impl PartialEq<&str> for Raw {
    fn eq(&self, other: &&str) -> bool {
        let option: &Option<String> = self;

        return match option {
            Some(string) => string == other,
            None => false,
        };
    }
}

/// A struct representing a semicolon value in the postcss parser grammar.
///
/// A semicolon can either be `true`, `false`, or non-existent. Since this is an artifact of the
/// original JavaScript representation this struct allows for better serialization and
/// deserialization to match those artifacts.
#[derive(PartialEq, Eq, Debug, Deserialize, Serialize, Clone)]
pub struct Semicolon(Option<bool>);

impl Semicolon {
    /// Return a new semicolon struct
    pub fn new(semicolon: bool) -> Self {
        return Self(Some(semicolon));
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
        return self.0.is_none();
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
    /// assert!(semi.unwrap());
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
    /// assert!(!semi.unwrap());
    /// ```
    pub fn set_false(&mut self) {
        self.0 = Some(false);
    }
}

impl Default for Semicolon {
    fn default() -> Self {
        return Semicolon(None);
    }
}

impl Deref for Semicolon {
    type Target = Option<bool>;

    fn deref(&self) -> &Option<bool> {
        return &self.0;
    }
}

/// A struct representing value raws with both the raw (with comments) and clean (without comments) versions
#[derive(PartialEq, Eq, Debug, Deserialize, Serialize, Clone)]
pub struct ValueRaw {
    /// The comment-stripped (clean) value, e.g. `"red"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// The raw value preserving inline comments, e.g. `"red /* note */"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
}

impl ValueRaw {
    /// Creates a new `ValueRaw` with both a clean `value` and a `raw` string.
    pub fn new(value: String, raw: String) -> Self {
        return Self {
            value: Some(value),
            raw: Some(raw),
        };
    }

    /// Returns `true` when both `value` and `raw` are absent (skip serialization).
    pub fn skip(&self) -> bool {
        return self.value.is_none() && self.raw.is_none();
    }
}

impl Default for ValueRaw {
    fn default() -> Self {
        return Self {
            value: None,
            raw: None,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcss_rs_tokenizer::{Kind, Token};

    fn space_token(value: &str) -> Token {
        Token { kind: Kind::Space, value: value.into(), start: 0, end: value.len(), column: 1 }
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
        assert_eq!(*s, Some(true));
    }

    #[test]
    fn test_semicolon_set_true() {
        let mut s = Semicolon::default();
        s.set_true();
        assert_eq!(*s, Some(true));
    }

    #[test]
    fn test_semicolon_set_false() {
        let mut s = Semicolon::default();
        s.set_false();
        assert_eq!(*s, Some(false));
    }

    // ---- ValueRaw tests ----

    #[test]
    fn test_value_raw_default_skips() {
        let vr = ValueRaw::default();
        assert!(vr.skip());
    }

    #[test]
    fn test_value_raw_new() {
        let vr = ValueRaw::new("red".to_string(), "red /* comment */".to_string());
        assert!(!vr.skip());
        assert_eq!(vr.value.as_deref(), Some("red"));
        assert_eq!(vr.raw.as_deref(), Some("red /* comment */"));
    }
}
