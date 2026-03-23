#![deny(clippy::all)]
#![allow(clippy::needless_return)]

use napi_derive::napi;

use napi::{Env, Result, Task};

use postcss_rs_parser::{Node, Parser};
use postcss_rs_stringifier::{Stringifier, StringifyOptions};

pub struct AsyncParseTask {
    css: String,
    _from: Option<String>,
}

impl Task for AsyncParseTask {
    type Output = String;
    type JsValue = String;

    fn compute(&mut self) -> Result<Self::Output> {
        let mut parser = Parser::new(&self.css).unwrap();
        let node = parser.parse();
        return node.to_json().map_err(|e| napi::Error::from_reason(e.to_string()));
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        return Ok(output);
    }
}

#[napi]
pub mod postcss {
    use napi::bindgen_prelude::AsyncTask;

    pub use super::*;

    #[napi]
    pub fn parse(css: String, from: Option<String>) -> AsyncTask<AsyncParseTask> {
        return AsyncTask::new(AsyncParseTask { css, _from: from });
    }

    #[napi]
    pub fn stringify(json: String) -> Result<String> {
        let node: Node = serde_json::from_str(&json)
            .map_err(|e| napi::Error::from_reason(format!("Failed to parse JSON: {}", e)))?;
        let css = Stringifier::stringify(&node, StringifyOptions::default());
        return Ok(css);
    }
}
