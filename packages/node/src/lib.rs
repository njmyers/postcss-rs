#![deny(clippy::all)]
#![allow(clippy::needless_return)]

use napi::bindgen_prelude::*;
use napi_derive::napi;

use napi::Env;
use napi::Result;
use napi::Task;

use postcss_rs_parser::Parser;
use postcss_rs_tokenizer::Tokenizer;

use std::convert::TryInto;
/// A JavaScript compatible and PostCSS compatible AST.
#[napi(js_name = "Node")]
pub struct JsNode {
    pub json: String,
}

/// Async compute task for parsing css.
pub struct AsyncParseTask {
    css: String,
}

impl Task for AsyncParseTask {
    type Output = JsNode;
    type JsValue = JsNode;

    fn compute(&mut self) -> Result<Self::Output> {
        let mut parser = Parser::new(&self.css).unwrap();
        let node = parser.parse();

        return Ok(JsNode {
            json: node.to_json().unwrap(),
        });
    }

    fn resolve(&mut self, _: Env, output: Self::Output) -> Result<Self::JsValue> {
        return Ok(output);
    }
}

#[napi]
pub mod postcss {
    pub use super::*;

    #[napi]
    pub fn parse(css: String) -> AsyncTask<AsyncParseTask> {
        return AsyncTask::new(AsyncParseTask { css });
    }
}

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
