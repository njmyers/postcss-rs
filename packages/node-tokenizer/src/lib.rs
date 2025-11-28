#![deny(clippy::all)]
#![allow(clippy::needless_return)]

#[macro_use]
extern crate napi_derive;

use napi::Result;
use postcss_rs_tokenizer::Tokenizer;
use std::convert::TryInto;

/// A JavaScript compatible and PostCSS compatible AST.
#[napi(js_name = "Tokenizer")]
pub struct JsTokenizer {
    tokenizer: Tokenizer,
}

#[napi]
impl JsTokenizer {
    #[napi(constructor)]
    pub fn new(css: String) -> Self {
        return Self {
            tokenizer: Tokenizer::new(css),
        };
    }

    #[napi]
    pub fn position(&mut self) -> u32 {
        return self.tokenizer.index.try_into().unwrap();
    }

    #[napi]
    pub fn back() -> Result<()> {
        return Ok(());
    }

    #[napi]
    pub fn next_token(&mut self) -> Option<String> {
        return self.tokenizer.next().map(|token| token.value);
    }

    #[napi]
    pub fn end_of_file(&mut self) -> bool {
        return self.tokenizer.peek().is_none();
    }
}

#[napi]
pub fn tokenizer(css: String) -> JsTokenizer {
    return JsTokenizer::new(css);
}
