use bytemuck::{Pod, Zeroable};

use crate::node::Source;
use crate::sinks::parse::{ParseSink, SinkNode};

// ── NodeSlot types ────────────────────────────────────────────────────────────

/// Sentinel value indicating an absent/null field.
pub const ABSENT: u32 = u32::MAX;

/// Flag bits stored in `NodeSlot::flags`.
pub struct Flags;
impl Flags {
    pub const IMPORTANT: u32 = 0x01;
    pub const SEMI_PRESENT: u32 = 0x02;
    pub const SEMI_VALUE: u32 = 0x04;
    pub const HAS_CLEAN_STR: u32 = 0x08;
    pub const CLEAN_IS_FIELD2: u32 = 0x10;
    pub const HAS_NODES: u32 = 0x20;
}

/// Node type tag values stored in `NodeSlot::type_tag`.
pub struct NodeType;
impl NodeType {
    pub const ROOT: u32 = 0;
    pub const RULE: u32 = 1;
    pub const DECL: u32 = 2;
    pub const ATRULE: u32 = 3;
    pub const COMMENT: u32 = 4;
}

/// Flat binary representation of one PostCSS AST node.
///
/// `#[repr(C)]` guarantees stable field order; all fields are `u32` so there
/// is no padding and `bytemuck::cast_slice` can convert `&[NodeSlot]` to
/// `&[u8]` with zero copies.  Declaration order equals the u32 slot index.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct NodeSlot {
    /// Slot 0 — node kind: see `NodeType` constants
    pub type_tag: u32,
    /// Slot 1 — index of the parent node in the flat array; ABSENT for root
    pub parent_idx: u32,
    /// Slot 2 — CSS byte offset of `source.start` (0-based)
    pub source_start_off: u32,
    /// Slot 3 — 1-based line of `source.start`
    pub source_start_line: u32,
    /// Slot 4 — 1-based column of `source.start`
    pub source_start_col: u32,
    /// Slot 5 — CSS byte offset of `source.end`; ABSENT if missing
    pub source_end_off: u32,
    /// Slot 6 — 1-based line of `source.end`
    pub source_end_line: u32,
    /// Slot 7 — 1-based column of `source.end`
    pub source_end_col: u32,
    /// Slot 8 — start of field1 raw: selector (rule) / prop (decl) / name (atrule) / text (comment)
    pub field1_start: u32,
    /// Slot 9 — end of field1 raw
    pub field1_end: u32,
    /// Slot 10 — start of field2 raw: value (decl) / params (atrule); ABSENT if absent
    pub field2_start: u32,
    /// Slot 11 — end of field2 raw
    pub field2_end: u32,
    /// Slot 12 — start of `raws.before`; ABSENT if absent
    pub raws_before_start: u32,
    /// Slot 13 — end of `raws.before`
    pub raws_before_end: u32,
    /// Slot 14 — start of `raws.between`; ABSENT if absent
    pub raws_between_start: u32,
    /// Slot 15 — end of `raws.between`
    pub raws_between_end: u32,
    /// Slot 16 — start of `raws.after`; ABSENT if absent
    pub raws_after_start: u32,
    /// Slot 17 — end of `raws.after`
    pub raws_after_end: u32,
    /// Slot 18 — start of extra1: `raws.afterName` (atrule) / `raws.left` (comment); ABSENT if absent
    pub raws_extra1_start: u32,
    /// Slot 19 — end of extra1
    pub raws_extra1_end: u32,
    /// Slot 20 — start of extra2: `raws.important` (decl) / `raws.right` (comment); ABSENT if absent
    pub raws_extra2_start: u32,
    /// Slot 21 — end of extra2
    pub raws_extra2_end: u32,
    /// Slot 22 — bit flags; see `Flags` constants
    pub flags: u32,
    /// Slot 23 — start of clean string in the trailing UTF-8 table (when `Flags::HAS_CLEAN_STR` set)
    pub clean_start: u32,
    /// Slot 24 — end of clean string in the trailing UTF-8 table
    pub clean_end: u32,
}

/// Number of u32 slots per node — derived from the struct size at compile time.
pub const STRIDE: usize = std::mem::size_of::<NodeSlot>() / 4;
const _STRIDE_CHECK: () = assert!(std::mem::size_of::<NodeSlot>() / 4 == 25);

// ── OffsetSink ────────────────────────────────────────────────────────────────

/// A [`ParseSink`] that writes flat [`NodeSlot`]s directly during parsing,
/// without building an intermediate `Node<'src>` tree.
pub struct OffsetSink<'src> {
    css: &'src str,
    slots: Vec<NodeSlot>,
    string_table: Vec<u8>,
}

impl<'src> OffsetSink<'src> {
    pub fn new(css: &'src str) -> Self {
        Self {
            css,
            slots: Vec::with_capacity(64),
            string_table: Vec::new(),
        }
    }

    #[inline]
    fn encode_ends_at(raw: Option<&str>, end_pos: u32) -> (u32, u32) {
        match raw {
            None => (ABSENT, 0),
            Some(s) => (end_pos.saturating_sub(Self::js_len(s)), end_pos),
        }
    }

    #[inline]
    fn encode_starts_at(raw: Option<&str>, start_pos: u32) -> (u32, u32) {
        match raw {
            None => (ABSENT, 0),
            Some(s) => (start_pos, start_pos + Self::js_len(s)),
        }
    }

    fn find_field_start(css: &str, src_start: u32, field: &str) -> (u32, usize) {
        if field.is_empty() {
            return (src_start, src_start as usize);
        }
        let byte_pos = src_start as usize;
        if css.as_bytes().get(byte_pos).copied() == field.as_bytes().first().copied() {
            return (src_start, byte_pos);
        }
        let prefix_bytes = css.get(byte_pos..).and_then(|s| s.find(field)).unwrap_or(0);
        let prefix_js = Self::js_len(&css[byte_pos..byte_pos + prefix_bytes]);
        (src_start + prefix_js, byte_pos + prefix_bytes)
    }

    #[inline]
    fn find_field_js_start(css: &str, src_start: u32, field: &str) -> u32 {
        Self::find_field_start(css, src_start, field).0
    }

    fn find_selector_css_end(css: &str, byte_start: usize, f1s: u32, sel_raw: &str) -> u32 {
        let mut css_chars = css[byte_start..].char_indices().peekable();
        let mut sel_chars = sel_raw.chars().peekable();
        let mut last_byte = 0usize;

        while let Some(&sel_ch) = sel_chars.peek() {
            match css_chars.next() {
                None => break,
                Some((i, css_ch)) => {
                    last_byte = i + css_ch.len_utf8();
                    if css_ch == sel_ch {
                        sel_chars.next();
                    } else if css_ch.is_whitespace() {
                        // skip inter-selector whitespace
                    }
                }
            }
        }

        f1s + Self::js_len(&css[byte_start..byte_start + last_byte])
    }

    #[inline]
    fn js_len(s: &str) -> u32 {
        if s.is_ascii() {
            return s.len() as u32;
        }
        s.chars().map(|c| c.len_utf16() as u32).sum()
    }

    #[inline]
    fn make_slot(parent: u32, type_tag: u32, ss: &Source) -> NodeSlot {
        let mut slot: NodeSlot = Zeroable::zeroed();
        slot.type_tag = type_tag;
        slot.parent_idx = parent;
        slot.source_start_off = ss.offset as u32;
        slot.source_start_line = ss.line as u32;
        slot.source_start_col = ss.column as u32;
        slot.source_end_off = ABSENT;
        slot.field2_start = ABSENT;
        slot.raws_before_start = ABSENT;
        slot.raws_between_start = ABSENT;
        slot.raws_after_start = ABSENT;
        slot.raws_extra1_start = ABSENT;
        slot.raws_extra2_start = ABSENT;
        slot
    }

    #[inline]
    fn push_string(&mut self, s: &str) -> (u32, u32) {
        let start = self.string_table.len() as u32;
        self.string_table.extend_from_slice(s.as_bytes());
        (start, self.string_table.len() as u32)
    }
}

impl<'src> ParseSink<'src> for OffsetSink<'src> {
    type NodeRef = u32;
    type Output = Vec<u8>;

    fn open(&mut self, parent: u32, node: SinkNode<'src>) -> u32 {
        let my_idx = self.slots.len() as u32;

        match node {
            SinkNode::Root { source_start } => {
                let mut slot = Self::make_slot(parent, NodeType::ROOT, &source_start);
                slot.parent_idx = ABSENT;
                slot.flags |= Flags::HAS_NODES;
                self.slots.push(slot);
            }

            SinkNode::Rule {
                source_start,
                selector,
                raws,
            } => {
                let src_start = source_start.offset as u32;
                let sel_raw = raws.selector.raw.as_deref().unwrap_or(selector.as_ref());

                let (f1s, f1s_byte) = Self::find_field_start(self.css, src_start, sel_raw);
                let actual_f1e = Self::find_selector_css_end(self.css, f1s_byte, f1s, sel_raw);

                let (rbs, rbe) = Self::encode_ends_at(raws.before.as_deref(), f1s);
                let (bt_s, bt_e) = Self::encode_starts_at(raws.between.as_deref(), actual_f1e);

                let mut flags = Flags::HAS_NODES;
                let (clean_start, clean_end);

                if raws.selector.raw.is_some() {
                    let (cs, ce) = self.push_string(selector.as_ref());
                    clean_start = cs;
                    clean_end = ce;
                    flags |= Flags::HAS_CLEAN_STR;
                } else if actual_f1e != f1s + Self::js_len(sel_raw) {
                    let (cs, ce) = self.push_string(sel_raw);
                    clean_start = cs;
                    clean_end = ce;
                    flags |= Flags::HAS_CLEAN_STR;
                } else {
                    clean_start = 0;
                    clean_end = 0;
                }

                let mut slot = Self::make_slot(parent, NodeType::RULE, &source_start);
                slot.field1_start = f1s;
                slot.field1_end = actual_f1e;
                slot.raws_before_start = rbs;
                slot.raws_before_end = rbe;
                slot.raws_between_start = bt_s;
                slot.raws_between_end = bt_e;
                slot.flags = flags;
                slot.clean_start = clean_start;
                slot.clean_end = clean_end;
                self.slots.push(slot);
            }

            SinkNode::AtRule {
                source_start,
                name,
                params,
                raws,
            } => {
                let src_start = source_start.offset as u32;

                let name_start = src_start + 1;
                let f1s = name_start;
                let f1e = name_start + Self::js_len(name);

                let after_name_len = raws
                    .after_name
                    .as_deref()
                    .map(|s| s.len() as u32)
                    .unwrap_or(0);
                let (an_s, an_e) = Self::encode_starts_at(raws.after_name.as_deref(), f1e);
                let params_start = f1e + after_name_len;

                let (f2s, f2e, params_end) = if let Some(p) = &params {
                    let pe = params_start + p.len() as u32;
                    (params_start, pe, pe)
                } else {
                    (ABSENT, 0, params_start)
                };

                let (bt_s, bt_e) = Self::encode_starts_at(raws.between.as_deref(), params_end);
                let (rbs, rbe) = Self::encode_ends_at(raws.before.as_deref(), src_start);

                let mut slot = Self::make_slot(parent, NodeType::ATRULE, &source_start);
                slot.field1_start = f1s;
                slot.field1_end = f1e;
                slot.field2_start = f2s;
                slot.field2_end = f2e;
                slot.raws_before_start = rbs;
                slot.raws_before_end = rbe;
                slot.raws_between_start = bt_s;
                slot.raws_between_end = bt_e;
                slot.raws_extra1_start = an_s;
                slot.raws_extra1_end = an_e;
                slot.flags = Flags::HAS_NODES;
                self.slots.push(slot);
            }
            _ => {}
        }

        my_idx
    }

    fn close(&mut self, handle: u32, node: SinkNode<'src>) {
        let SinkNode::Close { source_end, raws } = node else {
            return;
        };
        let src_end = source_end.offset as u32;
        let slot = &mut self.slots[handle as usize];

        slot.source_end_off = src_end;
        slot.source_end_line = source_end.line as u32;
        slot.source_end_col = source_end.column as u32;

        if let Some(semi_val) = *raws.semicolon {
            slot.flags |= Flags::SEMI_PRESENT;
            if semi_val {
                slot.flags |= Flags::SEMI_VALUE;
            }
        }

        match slot.type_tag {
            NodeType::ROOT => {
                if src_end != ABSENT {
                    let (as_, ae) = Self::encode_ends_at(raws.after.as_deref(), src_end);
                    slot.raws_after_start = as_;
                    slot.raws_after_end = ae;
                }
            }
            NodeType::RULE => {
                if src_end > 0 {
                    let (as_, ae) = Self::encode_ends_at(raws.after.as_deref(), src_end - 1);
                    slot.raws_after_start = as_;
                    slot.raws_after_end = ae;
                }
                if let Some(own_semi) = raws.own_semicolon.as_deref() {
                    let own_semi_end = match self.css.as_bytes().get(src_end as usize - 1) {
                        Some(b'\n') | Some(b'\r') => src_end - 1,
                        _ => src_end,
                    };
                    let (os, oe) = Self::encode_ends_at(Some(own_semi), own_semi_end);
                    slot.raws_extra1_start = os;
                    slot.raws_extra1_end = oe;
                }
            }
            NodeType::ATRULE => {
                if src_end > 0 && slot.flags & Flags::HAS_NODES != 0 {
                    let (as_, ae) = Self::encode_ends_at(raws.after.as_deref(), src_end - 1);
                    slot.raws_after_start = as_;
                    slot.raws_after_end = ae;
                }
            }
            _ => {}
        }
    }

    fn emit(&mut self, parent: u32, node: SinkNode<'src>) {
        match node {
            SinkNode::Declaration {
                sources,
                prop,
                value,
                important,
                raws,
            } => {
                let src_start = sources.start.offset as u32;
                let f1s = Self::find_field_js_start(self.css, src_start, prop.as_ref());
                let f1e = f1s + Self::js_len(prop.as_ref());

                let (rbs, rbe) = Self::encode_ends_at(raws.before.as_deref(), f1s);
                let between_len = raws.between.as_deref().map(|s| s.len() as u32).unwrap_or(0);
                let (bt_s, bt_e) = Self::encode_starts_at(raws.between.as_deref(), f1e);
                let value_start = f1e + between_len;

                let val_raw_str = raws.value.raw.as_deref().unwrap_or(value.as_ref());
                let f2s = value_start;
                let f2e = value_start + val_raw_str.len() as u32;

                let (i_s, i_e) = Self::encode_starts_at(raws.important.as_deref(), f2e);

                let mut flags = 0u32;
                if important == Some(true) {
                    flags |= Flags::IMPORTANT;
                }

                let (clean_start, clean_end) = if raws.value.raw.is_some() {
                    let (cs, ce) = self.push_string(value.as_ref());
                    flags |= Flags::HAS_CLEAN_STR | Flags::CLEAN_IS_FIELD2;
                    (cs, ce)
                } else {
                    (0, 0)
                };

                let mut slot = Self::make_slot(parent, NodeType::DECL, &sources.start);
                if let Some(ref end) = sources.end {
                    slot.source_end_off = end.offset as u32;
                    slot.source_end_line = end.line as u32;
                    slot.source_end_col = end.column as u32;
                }
                slot.field1_start = f1s;
                slot.field1_end = f1e;
                slot.field2_start = f2s;
                slot.field2_end = f2e;
                slot.raws_before_start = rbs;
                slot.raws_before_end = rbe;
                slot.raws_between_start = bt_s;
                slot.raws_between_end = bt_e;
                slot.raws_extra2_start = i_s;
                slot.raws_extra2_end = i_e;
                slot.flags = flags;
                slot.clean_start = clean_start;
                slot.clean_end = clean_end;
                self.slots.push(slot);
            }

            SinkNode::Comment {
                sources,
                text,
                raws,
            } => {
                let src_start = sources.start.offset as u32;
                let src_end = sources.end.as_ref().map_or(0u32, |e| e.offset as u32);

                let left_start = src_start + 2;
                let left_len = raws.left.as_deref().map(|l| l.len() as u32).unwrap_or(0);
                let (l_s, l_e) = Self::encode_starts_at(raws.left.as_deref(), left_start);
                let text_start = left_start + left_len;

                let f1s = text_start;
                let f1e = text_start + Self::js_len(text);

                let (rbs, rbe) = Self::encode_ends_at(raws.before.as_deref(), src_start);
                let (r_s, r_e) = if src_end >= 2 {
                    Self::encode_ends_at(raws.right.as_deref(), src_end - 2)
                } else {
                    (ABSENT, 0)
                };

                let mut slot = Self::make_slot(parent, NodeType::COMMENT, &sources.start);
                if let Some(ref end) = sources.end {
                    slot.source_end_off = end.offset as u32;
                    slot.source_end_line = end.line as u32;
                    slot.source_end_col = end.column as u32;
                }
                slot.field1_start = f1s;
                slot.field1_end = f1e;
                slot.raws_before_start = rbs;
                slot.raws_before_end = rbe;
                slot.raws_extra1_start = l_s;
                slot.raws_extra1_end = l_e;
                slot.raws_extra2_start = r_s;
                slot.raws_extra2_end = r_e;
                self.slots.push(slot);
            }

            SinkNode::AtRuleStatement {
                sources,
                name,
                params,
                raws,
            } => {
                let src_start = sources.start.offset as u32;

                let name_start = src_start + 1;
                let f1s = name_start;
                let f1e = name_start + Self::js_len(name);

                let after_name_len = raws
                    .after_name
                    .as_deref()
                    .map(|s| s.len() as u32)
                    .unwrap_or(0);
                let (an_s, an_e) = Self::encode_starts_at(raws.after_name.as_deref(), f1e);
                let params_start = f1e + after_name_len;

                let (f2s, f2e, params_end) = if let Some(p) = &params {
                    let pe = params_start + p.len() as u32;
                    (params_start, pe, pe)
                } else {
                    (ABSENT, 0, params_start)
                };

                let (bt_s, bt_e) = Self::encode_starts_at(raws.between.as_deref(), params_end);
                let (rbs, rbe) = Self::encode_ends_at(raws.before.as_deref(), src_start);

                let mut slot = Self::make_slot(parent, NodeType::ATRULE, &sources.start);
                if let Some(ref end) = sources.end {
                    slot.source_end_off = end.offset as u32;
                    slot.source_end_line = end.line as u32;
                    slot.source_end_col = end.column as u32;
                }
                slot.field1_start = f1s;
                slot.field1_end = f1e;
                slot.field2_start = f2s;
                slot.field2_end = f2e;
                slot.raws_before_start = rbs;
                slot.raws_before_end = rbe;
                slot.raws_between_start = bt_s;
                slot.raws_between_end = bt_e;
                slot.raws_extra1_start = an_s;
                slot.raws_extra1_end = an_e;
                self.slots.push(slot);
            }
            _ => {}
        }
    }

    fn finish(self) -> Vec<u8> {
        let count_bytes = (self.slots.len() as u32).to_le_bytes();
        let slots_bytes: &[u8] = bytemuck::cast_slice(&self.slots);
        let mut out = Vec::with_capacity(4 + slots_bytes.len() + self.string_table.len());
        out.extend_from_slice(&count_bytes);
        out.extend_from_slice(slots_bytes);
        out.extend_from_slice(&self.string_table);
        out
    }
}
