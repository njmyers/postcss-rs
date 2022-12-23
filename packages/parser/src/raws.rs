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
    #[serde(default, skip_serializing_if = "Semicolon::skip")]
    pub semicolon: Semicolon,
    #[serde(default, skip_serializing_if = "Raw::skip")]
    pub after: Raw,
}

impl Default for Raws {
    fn default() -> Self {
        return Self {
            before: Raw::default(),
            between: Raw::default(),
            semicolon: Semicolon::default(),
            after: Raw::new(""),
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

    /// Push a new string slice to a raw value
    ///
    /// # Example
    ///
    /// ```
    /// use postcss_rs_parser::Raw;
    ///
    /// let mut raw = Raw::default();
    /// raw.push("\n");
    ///
    /// assert_eq!(&raw, "\n");
    /// ```
    pub fn push(&mut self, raw: &str) {
        match self.0 {
            Some(ref mut prev) => prev.push_str(raw),
            None => self.0 = Some(String::from(raw)),
        };
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

impl PartialEq<str> for Raw {
    fn eq(&self, other: &str) -> bool {
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
