#![deny(clippy::all)]
#![allow(clippy::needless_return)]

use napi_derive::napi;

use napi::{Env, Result, Task};

use postcss_rs_parser::{Data, Node, Parser, Raws, Source, Sources};
use postcss_rs_stringifier::{Stringifier, StringifyOptions};

// ── Fast JSON serializer ───────────────────────────────────────────────────────
//
// serde_json's #[serde(flatten)] + #[serde(tag = "type")] combination is O(n²)
// because it buffers every field into an intermediate Value map before writing.
// For large CSS files with thousands of nodes this causes severe slowdowns.
//
// This hand-written serializer visits each node once (O(n)) and writes JSON
// directly to a growing String buffer — no intermediate allocation per node.

fn escape_json_string(s: &str, out: &mut Vec<u8>) {
    out.push(b'"');
    for byte in s.bytes() {
        match byte {
            b'"' => out.extend_from_slice(b"\\\""),
            b'\\' => out.extend_from_slice(b"\\\\"),
            b'\n' => out.extend_from_slice(b"\\n"),
            b'\r' => out.extend_from_slice(b"\\r"),
            b'\t' => out.extend_from_slice(b"\\t"),
            0x00..=0x1f => {
                // Control characters: \uXXXX
                out.extend_from_slice(format!("\\u{:04x}", byte).as_bytes());
            }
            b => out.push(b),
        }
    }
    out.push(b'"');
}

fn write_raws(raws: &Raws, out: &mut Vec<u8>) {
    out.push(b'{');
    let mut first = true;
    macro_rules! write_raw_str {
        ($key:expr, $val:expr) => {
            if let Some(s) = $val {
                if !first { out.push(b','); }
                first = false;
                out.push(b'"');
                out.extend_from_slice($key);
                out.push(b'"');
                out.push(b':');
                escape_json_string(s, out);
            }
        };
    }
    write_raw_str!(b"before", raws.before.as_deref());
    write_raw_str!(b"between", raws.between.as_deref());
    write_raw_str!(b"afterName", raws.after_name.as_deref());
    if let Some(b) = *raws.semicolon {
        if !first { out.push(b','); }
        first = false;
        out.extend_from_slice(b"\"semicolon\":");
        out.extend_from_slice(if b { b"true" } else { b"false" });
    }
    write_raw_str!(b"after", raws.after.as_deref());
    write_raw_str!(b"left", raws.left.as_deref());
    write_raw_str!(b"right", raws.right.as_deref());
    write_raw_str!(b"important", raws.important.as_deref());
    write_raw_str!(b"ownSemicolon", raws.own_semicolon.as_deref());
    if !raws.value.skip() {
        if !first { out.push(b','); }
        first = false;
        out.extend_from_slice(b"\"value\":{");
        let mut inner_first = true;
        if let Some(v) = &raws.value.value {
            inner_first = false;
            out.extend_from_slice(b"\"value\":");
            escape_json_string(v, out);
        }
        if let Some(r) = &raws.value.raw {
            if !inner_first { out.push(b','); }
            out.extend_from_slice(b"\"raw\":");
            escape_json_string(r, out);
        }
        out.push(b'}');
    }
    if !raws.selector.skip() {
        if !first { out.push(b','); }
        out.extend_from_slice(b"\"selector\":{");
        let mut inner_first = true;
        if let Some(v) = &raws.selector.value {
            inner_first = false;
            out.extend_from_slice(b"\"value\":");
            escape_json_string(v, out);
        }
        if let Some(r) = &raws.selector.raw {
            if !inner_first { out.push(b','); }
            out.extend_from_slice(b"\"raw\":");
            escape_json_string(r, out);
        }
        out.push(b'}');
    }
    let _ = first; // suppress unused warning
    out.push(b'}');
}

fn write_source_pos(src: &Source, out: &mut Vec<u8>) {
    use std::io::Write;
    let _ = write!(out, "{{\"offset\":{},\"line\":{},\"column\":{}}}", src.offset, src.line, src.column);
}

fn write_sources(sources: &Sources, out: &mut Vec<u8>) {
    out.extend_from_slice(b"{\"start\":");
    write_source_pos(&sources.start, out);
    if let Some(end) = &sources.end {
        out.extend_from_slice(b",\"end\":");
        write_source_pos(end, out);
    }
    out.push(b'}');
}

fn write_node(node: &Node, out: &mut Vec<u8>) {
    out.push(b'{');

    // raws always comes first
    out.extend_from_slice(b"\"raws\":");
    write_raws(&node.raws, out);
    out.push(b',');

    // type + type-specific fields
    match &node.data {
        Data::Root => {
            out.extend_from_slice(b"\"type\":\"root\"");
        }
        Data::Rule(rule) => {
            out.extend_from_slice(b"\"type\":\"rule\",\"selector\":");
            escape_json_string(&rule.selector, out);
        }
        Data::Declaration(decl) => {
            out.extend_from_slice(b"\"type\":\"decl\",\"prop\":");
            escape_json_string(&decl.prop, out);
            out.extend_from_slice(b",\"value\":");
            escape_json_string(&decl.value, out);
            if let Some(true) = decl.important {
                out.extend_from_slice(b",\"important\":true");
            }
        }
        Data::AtRule(at_rule) => {
            out.extend_from_slice(b"\"type\":\"atrule\",\"name\":");
            escape_json_string(&at_rule.name, out);
            out.extend_from_slice(b",\"params\":");
            escape_json_string(at_rule.params.as_deref().unwrap_or(""), out);
        }
        Data::Comment(comment) => {
            out.extend_from_slice(b"\"type\":\"comment\",\"text\":");
            escape_json_string(&comment.text, out);
        }
    }

    // child nodes
    if let Some(children) = &node.nodes {
        out.extend_from_slice(b",\"nodes\":[");
        for (i, child) in children.iter().enumerate() {
            if i > 0 { out.push(b','); }
            write_node(child, out);
        }
        out.push(b']');
    }

    // source
    out.extend_from_slice(b",\"source\":");
    write_sources(&node.source, out);

    out.push(b'}');
}

fn node_to_json_fast(node: &Node) -> String {
    let mut buf = Vec::with_capacity(4096);
    write_node(node, &mut buf);
    // SAFETY: write_node only emits valid UTF-8 (ASCII + escaped Unicode)
    unsafe { String::from_utf8_unchecked(buf) }
}

// ── Async parse task ──────────────────────────────────────────────────────────

pub struct AsyncParseTask {
    css: String,
    _from: Option<String>,
}

impl Task for AsyncParseTask {
    /// Parse CSS + serialize to JSON — both run on the thread pool.
    type Output = String;
    /// JSON string is returned as-is; JS does JSON.parse() + fromJSON().
    type JsValue = String;

    fn compute(&mut self) -> Result<Self::Output> {
        let mut parser = Parser::new(&self.css).unwrap();
        let node = parser.parse();
        return Ok(node_to_json_fast(&node));
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        return Ok(output);
    }
}

// ── Exported module ───────────────────────────────────────────────────────────

#[napi]
pub mod postcss {
    use napi::bindgen_prelude::AsyncTask;

    pub use super::*;

    #[napi]
    pub fn parse(css: String, from: Option<String>) -> AsyncTask<AsyncParseTask> {
        return AsyncTask::new(AsyncParseTask { css, _from: from });
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
        let node: Node = serde_json::from_str(&json)
            .map_err(|e| napi::Error::from_reason(format!("Failed to parse JSON: {}", e)))?;
        let css = Stringifier::stringify(&node, StringifyOptions::default());
        return Ok(css);
    }
}
