#![deny(clippy::all)]
#![allow(clippy::needless_return)]

use napi_derive::napi;

// use napi::*;
use napi::Env;
use napi::Result;
use napi::Task;

use postcss_rs_parser::{Parser, Node};

mod stringify;

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
    from: Option<String>,
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
    pub fn parse(css: String, from: Option<String>) -> AsyncTask<AsyncParseTask> {
        return AsyncTask::new(AsyncParseTask { css, from });
    }

    /// Stringify a PostCSS AST back to CSS
    ///
    /// Takes JSON representation of AST and converts it back to CSS string,
    /// preserving all original formatting from the `raws` fields.
    ///
    /// @param json - JSON string representation of PostCSS AST
    /// @returns CSS string
    #[napi]
    pub fn stringify(json: String) -> Result<String> {
        // Deserialize JSON to Node
        let node: Node = serde_json::from_str(&json)
            .map_err(|e| napi::Error::from_reason(format!("Failed to parse JSON: {}", e)))?;

        // Stringify the node
        let css = stringify::Stringifier::stringify(&node, stringify::StringifyOptions::default());

        return Ok(css);
    }
}
