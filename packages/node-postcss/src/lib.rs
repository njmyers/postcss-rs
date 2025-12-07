#![deny(clippy::all)]
#![allow(clippy::needless_return)]

use napi::bindgen_prelude::*;
use napi_derive::napi;

// use napi::*;
use napi::Env;
use napi::Result;
use napi::Task;

use postcss_rs_parser::Parser;

/// A JavaScript compatible and PostCSS compatible AST.
#[napi(js_name = "Node")]
pub struct JsNode {
    pub json: String,
}

#[napi]
impl JsNode {
    #[napi(constructor)]
    pub fn new(json: String) -> Self {
        return JsNode { json };
    }
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
    use napi::bindgen_prelude::AsyncTask;

    pub use super::*;

    #[napi]
    pub fn parse(css: String) -> AsyncTask<AsyncParseTask> {
        return AsyncTask::new(AsyncParseTask { css });
    }
}
