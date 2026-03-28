#![deny(clippy::all)]
#![allow(clippy::needless_return)]

mod offset;

use napi_derive::napi;
use napi::{Env, Result, Task};
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

/// Binary layout descriptor for the offset-array buffer produced by
/// `parseOffsetsSync`.  All field indices are u32 slot indices within a
/// single node's STRIDE-wide block; multiply by 4 for byte offset.
///
/// Exported via `postcss.nodeSlotLayout()` so TypeScript can use named
/// constants derived directly from the Rust struct rather than magic numbers.
#[napi(object)]
pub struct NodeSlotLayout {
    pub type_root:    u32,
    pub type_rule:    u32,
    pub type_decl:    u32,
    pub type_atrule:  u32,
    pub type_comment: u32,

    /// u32 slots per node (= 25)
    pub stride: u32,
    /// Absent-field sentinel (= 0xFFFF_FFFF)
    pub absent: u32,

    pub idx_type_tag:           u32,
    pub idx_parent_idx:         u32,
    pub idx_source_start_off:   u32,
    pub idx_source_start_line:  u32,
    pub idx_source_start_col:   u32,
    pub idx_source_end_off:     u32,
    pub idx_source_end_line:    u32,
    pub idx_source_end_col:     u32,
    pub idx_field1_start:       u32,
    pub idx_field1_end:         u32,
    pub idx_field2_start:       u32,
    pub idx_field2_end:         u32,
    pub idx_raws_before_start:  u32,
    pub idx_raws_before_end:    u32,
    pub idx_raws_between_start: u32,
    pub idx_raws_between_end:   u32,
    pub idx_raws_after_start:   u32,
    pub idx_raws_after_end:     u32,
    pub idx_raws_extra1_start:  u32,
    pub idx_raws_extra1_end:    u32,
    pub idx_raws_extra2_start:  u32,
    pub idx_raws_extra2_end:    u32,
    pub idx_flags:              u32,
    pub idx_clean_start:        u32,
    pub idx_clean_end:          u32,

    pub flag_important:        u32,
    pub flag_semi_present:     u32,
    pub flag_semi_value:       u32,
    pub flag_has_clean_str:    u32,
    pub flag_clean_is_field2:  u32,
    pub flag_has_nodes:        u32,
}

#[napi]
pub mod postcss {
    use napi::bindgen_prelude::{AsyncTask, Buffer};
    use std::mem::offset_of;
    use crate::offset::{NodeSlot, STRIDE, Flags, NodeType, ABSENT};

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
        let mut buf = Vec::with_capacity(css.len() * 5);
        sonic_rs::to_writer(&mut buf, &node)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
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
        // offset_of! returns byte offset; divide by 4 to get u32 slot index.
        NodeSlotLayout {
            type_root:    NodeType::ROOT,
            type_rule:    NodeType::RULE,
            type_decl:    NodeType::DECL,
            type_atrule:  NodeType::ATRULE,
            type_comment: NodeType::COMMENT,

            stride: STRIDE as u32,
            absent: ABSENT,

            idx_type_tag:           (offset_of!(NodeSlot, type_tag)           / 4) as u32,
            idx_parent_idx:         (offset_of!(NodeSlot, parent_idx)         / 4) as u32,
            idx_source_start_off:   (offset_of!(NodeSlot, source_start_off)   / 4) as u32,
            idx_source_start_line:  (offset_of!(NodeSlot, source_start_line)  / 4) as u32,
            idx_source_start_col:   (offset_of!(NodeSlot, source_start_col)   / 4) as u32,
            idx_source_end_off:     (offset_of!(NodeSlot, source_end_off)     / 4) as u32,
            idx_source_end_line:    (offset_of!(NodeSlot, source_end_line)    / 4) as u32,
            idx_source_end_col:     (offset_of!(NodeSlot, source_end_col)     / 4) as u32,
            idx_field1_start:       (offset_of!(NodeSlot, field1_start)       / 4) as u32,
            idx_field1_end:         (offset_of!(NodeSlot, field1_end)         / 4) as u32,
            idx_field2_start:       (offset_of!(NodeSlot, field2_start)       / 4) as u32,
            idx_field2_end:         (offset_of!(NodeSlot, field2_end)         / 4) as u32,
            idx_raws_before_start:  (offset_of!(NodeSlot, raws_before_start)  / 4) as u32,
            idx_raws_before_end:    (offset_of!(NodeSlot, raws_before_end)    / 4) as u32,
            idx_raws_between_start: (offset_of!(NodeSlot, raws_between_start) / 4) as u32,
            idx_raws_between_end:   (offset_of!(NodeSlot, raws_between_end)   / 4) as u32,
            idx_raws_after_start:   (offset_of!(NodeSlot, raws_after_start)   / 4) as u32,
            idx_raws_after_end:     (offset_of!(NodeSlot, raws_after_end)     / 4) as u32,
            idx_raws_extra1_start:  (offset_of!(NodeSlot, raws_extra1_start)  / 4) as u32,
            idx_raws_extra1_end:    (offset_of!(NodeSlot, raws_extra1_end)    / 4) as u32,
            idx_raws_extra2_start:  (offset_of!(NodeSlot, raws_extra2_start)  / 4) as u32,
            idx_raws_extra2_end:    (offset_of!(NodeSlot, raws_extra2_end)    / 4) as u32,
            idx_flags:              (offset_of!(NodeSlot, flags)              / 4) as u32,
            idx_clean_start:        (offset_of!(NodeSlot, clean_start)        / 4) as u32,
            idx_clean_end:          (offset_of!(NodeSlot, clean_end)          / 4) as u32,

            flag_important:       Flags::IMPORTANT,
            flag_semi_present:    Flags::SEMI_PRESENT,
            flag_semi_value:      Flags::SEMI_VALUE,
            flag_has_clean_str:   Flags::HAS_CLEAN_STR,
            flag_clean_is_field2: Flags::CLEAN_IS_FIELD2,
            flag_has_nodes:       Flags::HAS_NODES,
        }
    }
}
