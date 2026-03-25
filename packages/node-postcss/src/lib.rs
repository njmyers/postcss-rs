#![deny(clippy::all)]
#![allow(clippy::needless_return)]

mod fb_serializer;
mod direct_builder;

use napi_derive::napi;
use napi::{Env, JsObject, Result, Task};
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
        // sonic-rs is a SIMD-accelerated JSON library (AVX2 on this host).
        // Pre-allocate the output buffer to avoid reallocations during
        // serialisation; CSS AST JSON is typically 3–8× the CSS source size.
        let mut buf = Vec::with_capacity(self.css.len() * 5);
        sonic_rs::to_writer(&mut buf, &node)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        // sonic-rs always produces valid UTF-8.
        return Ok(unsafe { String::from_utf8_unchecked(buf) });
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        return Ok(output);
    }
}

pub struct AsyncParseFbTask {
    css: String,
}

pub struct AsyncParseMsgpackTask {
    css: String,
}

impl Task for AsyncParseMsgpackTask {
    type Output = Vec<u8>;
    type JsValue = napi::bindgen_prelude::Buffer;

    fn compute(&mut self) -> Result<Self::Output> {
        let mut parser = Parser::new(&self.css).unwrap();
        let node = parser.parse();
        rmp_serde::to_vec_named(&node)
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        return Ok(output.into());
    }
}

impl Task for AsyncParseFbTask {
    type Output = Vec<u8>;
    type JsValue = napi::bindgen_prelude::Buffer;

    fn compute(&mut self) -> Result<Self::Output> {
        let mut parser = Parser::new(&self.css).unwrap();
        let node = parser.parse();
        return Ok(fb_serializer::to_flatbuffer(&node, self.css.len()));
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
    pub fn parse(css: String, from: Option<String>) -> AsyncTask<AsyncParseTask> {
        return AsyncTask::new(AsyncParseTask { css, _from: from });
    }

    /// Synchronous version of parse(). Blocks the event loop while parsing.
    /// Faster than parse() for small/medium CSS because it avoids the
    /// Promise + thread-pool round-trip overhead (~0.05–0.1 ms per call).
    #[napi]
    pub fn parse_sync(css: String, _from: Option<String>) -> Result<String> {
        let mut parser = Parser::new(&css).unwrap();
        let node = parser.parse();
        let mut buf = Vec::with_capacity(css.len() * 5);
        sonic_rs::to_writer(&mut buf, &node)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        return Ok(unsafe { String::from_utf8_unchecked(buf) });
    }

    /// Synchronous FlatBuffers parse. Returns a raw Buffer instead of a JSON
    /// string, allowing the JS side to skip JSON.parse() entirely.
    #[napi]
    pub fn parse_fb_sync(css: String, _from: Option<String>) -> Result<Buffer> {
        let mut parser = Parser::new(&css).unwrap();
        let node = parser.parse();
        return Ok(fb_serializer::to_flatbuffer(&node, css.len()).into());
    }

    /// Async FlatBuffers parse.
    #[napi]
    pub fn parse_fb(css: String, _from: Option<String>) -> AsyncTask<AsyncParseFbTask> {
        return AsyncTask::new(AsyncParseFbTask { css });
    }

    /// Synchronous MessagePack parse. Returns a raw Buffer of MessagePack bytes.
    #[napi]
    pub fn parse_msgpack_sync(css: String, _from: Option<String>) -> Result<Buffer> {
        let mut parser = Parser::new(&css).unwrap();
        let node = parser.parse();
        rmp_serde::to_vec_named(&node)
            .map(|v| v.into())
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Async MessagePack parse.
    #[napi]
    pub fn parse_msgpack(css: String, _from: Option<String>) -> AsyncTask<AsyncParseMsgpackTask> {
        return AsyncTask::new(AsyncParseMsgpackTask { css });
    }

    /// Direct NAPI V8 object construction — no serialization at all.
    ///
    /// `ctx` must be a JS object with:
    ///   makeRoot, makeRule, makeDecl, makeAtRule, makeComment  (factory fns)
    ///   input  (postcss.Input instance)
    #[napi]
    pub fn parse_direct_sync(env: Env, css: String, ctx: JsObject) -> Result<JsObject> {
        direct_builder::build(&env, &css, &ctx)
    }

    #[napi]
    pub fn stringify(json: String) -> Result<String> {
        let node: Node = serde_json::from_str(&json)
            .map_err(|e| napi::Error::from_reason(format!("Failed to parse JSON: {}", e)))?;
        let css = Stringifier::stringify(&node, StringifyOptions::default());
        return Ok(css);
    }
}
