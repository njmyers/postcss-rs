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
    ///     value: "\n".to_string(),
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
