#![deny(clippy::all)]
#![allow(clippy::needless_return)]

mod json;
mod offset;

pub use offset::NodeSlotLayout;

use napi::{Env, Result, Task};
use napi_derive::napi;
use postcss_rs_parser::{Node, Parser};
use postcss_rs_stringifier::{Stringifier, StringifyOptions};

pub struct AsyncJsonTask {
    css: String,
    _from: Option<String>,
}

impl Task for AsyncJsonTask {
    type Output = String;
    type JsValue = String;

    fn compute(&mut self) -> Result<Self::Output> {
        let mut parser = Parser::new(&self.css).unwrap();
        let node = parser.parse();
        let buf = json::serialize(&node, &self.css);
        return Ok(unsafe { String::from_utf8_unchecked(buf) });
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        return Ok(output);
    }
}

pub struct AsyncOffsetTask {
    css: String,
    _from: Option<String>,
}

impl Task for AsyncOffsetTask {
    type Output = Vec<u8>;
    type JsValue = napi::bindgen_prelude::Buffer;

    fn compute(&mut self) -> Result<Self::Output> {
        let mut parser = Parser::new(&self.css).unwrap();
        let node = parser.parse();
        return Ok(offset::OffsetSerializer::serialize(&node, &self.css));
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        return Ok(output.into());
    }
}

#[napi]
pub mod postcss {
    use napi::bindgen_prelude::{AsyncTask, Buffer};

    pub use super::*;

    #[napi]
    pub fn parse_to_json_async(css: String, from: Option<String>) -> AsyncTask<AsyncJsonTask> {
        return AsyncTask::new(AsyncJsonTask { css, _from: from });
    }

    /// Synchronous JSON parse. Blocks the event loop while parsing.
    /// Faster than parse_to_json_async() for small/medium CSS because it
    /// avoids the Promise + thread-pool round-trip overhead (~0.05–0.1 ms).
    #[napi]
    pub fn parse_to_json_sync(css: String, _from: Option<String>) -> Result<String> {
        let mut parser = Parser::new(&css).unwrap();
        let node = parser.parse();
        let buf = json::serialize(&node, &css);
        return Ok(unsafe { String::from_utf8_unchecked(buf) });
    }

    /// Synchronous offset-array parse.  Returns a compact binary buffer
    /// containing a flat `Uint32Array` of CSS byte offsets plus a UTF-8
    /// string table for any clean values that differ from their raw form.
    /// The JS side reconstructs strings with `css.slice(start, end)` and
    /// never needs to call `JSON.parse()`.
    #[napi]
    pub fn parse_to_offsets_sync(css: String, _from: Option<String>) -> Result<Buffer> {
        let mut parser = Parser::new(&css).unwrap();
        let node = parser.parse();
        return Ok(offset::OffsetSerializer::serialize(&node, &css).into());
    }

    /// Async offset-array parse.  Same output as `parse_to_offsets_sync` but
    /// runs on the thread pool so the event loop is not blocked.
    #[napi]
    pub fn parse_to_offsets_async(css: String, from: Option<String>) -> AsyncTask<AsyncOffsetTask> {
        return AsyncTask::new(AsyncOffsetTask { css, _from: from });
    }

    #[napi]
    pub fn stringify(json: String) -> Result<String> {
        let node: Node = serde_json::from_str(&json)
            .map_err(|e| napi::Error::from_reason(format!("Failed to parse JSON: {}", e)))?;
        let css = Stringifier::stringify(&node, StringifyOptions::default());
        return Ok(css);
    }

    /// Return the binary layout descriptor for the offset-array buffer.
    ///
    /// Field indices are derived from the actual `NodeSlot` struct memory
    /// layout via `offset_of!`, so they stay in sync with the Rust code
    /// automatically.  Call once at module startup and cache the result.
    #[napi]
    pub fn node_slot_layout() -> NodeSlotLayout {
        NodeSlotLayout::build()
    }

}
