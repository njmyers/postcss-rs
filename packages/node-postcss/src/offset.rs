use bytemuck::{Pod, Zeroable};
use napi_derive::napi;
use postcss_rs_parser::{Data, Node, Raw};

/// Sentinel value indicating an absent/null field.
pub const ABSENT: u32 = u32::MAX;

/// Flag bits stored in `NodeSlot::flags`.
pub struct Flags;
impl Flags {
    pub const IMPORTANT:       u32 = 0x01;
    pub const SEMI_PRESENT:    u32 = 0x02;
    pub const SEMI_VALUE:      u32 = 0x04;
    pub const HAS_CLEAN_STR:   u32 = 0x08;
    pub const CLEAN_IS_FIELD2: u32 = 0x10;
    pub const HAS_NODES:       u32 = 0x20;
}

/// Node type tag values stored in `NodeSlot::type_tag`.
pub struct NodeType;
impl NodeType {
    pub const ROOT:    u32 = 0;
    pub const RULE:    u32 = 1;
    pub const DECL:    u32 = 2;
    pub const ATRULE:  u32 = 3;
    pub const COMMENT: u32 = 4;
}

/// Binary layout descriptor for the offset-array buffer produced by
/// `parseOffsetsSync`.  All field indices are u32 slot indices within a
/// single node's STRIDE-wide block; multiply by 4 for byte offset.
///
/// Exported via `postcss.nodeSlotLayout()` so TypeScript can use named
/// constants derived directly from the Rust struct rather than magic numbers.
#[napi(object)]
pub struct NodeSlotLayout {
    /// `type_tag` value for a Root node
    pub type_root:    u32,
    /// `type_tag` value for a Rule node
    pub type_rule:    u32,
    /// `type_tag` value for a Declaration node
    pub type_decl:    u32,
    /// `type_tag` value for an AtRule node
    pub type_atrule:  u32,
    /// `type_tag` value for a Comment node
    pub type_comment: u32,

    /// Number of u32 slots per node in the buffer (= 25)
    pub stride: u32,
    /// Sentinel value meaning a field is absent/null (= 0xFFFF_FFFF)
    pub absent: u32,

    /// Slot index of the node type tag (see `type_*` constants)
    pub idx_type_tag:           u32,
    /// Slot index of the parent node's index in the flat array; `absent` for the root
    pub idx_parent_idx:         u32,
    /// Slot index of `source.start` JS char offset
    pub idx_source_start_off:   u32,
    /// Slot index of `source.start` 1-based line number
    pub idx_source_start_line:  u32,
    /// Slot index of `source.start` 1-based column number
    pub idx_source_start_col:   u32,
    /// Slot index of `source.end` JS char offset; `absent` if missing
    pub idx_source_end_off:     u32,
    /// Slot index of `source.end` 1-based line number
    pub idx_source_end_line:    u32,
    /// Slot index of `source.end` 1-based column number
    pub idx_source_end_col:     u32,
    /// Slot index of field1 start: selector (Rule), prop (Decl), name (AtRule), text (Comment)
    pub idx_field1_start:       u32,
    /// Slot index of field1 end (exclusive)
    pub idx_field1_end:         u32,
    /// Slot index of field2 start: value (Decl), params (AtRule); `absent` if not present
    pub idx_field2_start:       u32,
    /// Slot index of field2 end (exclusive)
    pub idx_field2_end:         u32,
    /// Slot index of `raws.before` start; `absent` if not present
    pub idx_raws_before_start:  u32,
    /// Slot index of `raws.before` end (exclusive)
    pub idx_raws_before_end:    u32,
    /// Slot index of `raws.between` start; `absent` if not present
    pub idx_raws_between_start: u32,
    /// Slot index of `raws.between` end (exclusive)
    pub idx_raws_between_end:   u32,
    /// Slot index of `raws.after` start; `absent` if not present
    pub idx_raws_after_start:   u32,
    /// Slot index of `raws.after` end (exclusive)
    pub idx_raws_after_end:     u32,
    /// Slot index of extra1 start: `raws.afterName` (AtRule), `raws.left` (Comment),
    /// `raws.ownSemicolon` (Rule); `absent` if not present
    pub idx_raws_extra1_start:  u32,
    /// Slot index of extra1 end (exclusive)
    pub idx_raws_extra1_end:    u32,
    /// Slot index of extra2 start: `raws.important` (Decl), `raws.right` (Comment);
    /// `absent` if not present
    pub idx_raws_extra2_start:  u32,
    /// Slot index of extra2 end (exclusive)
    pub idx_raws_extra2_end:    u32,
    /// Slot index of the bit-flag word (see `flag_*` constants)
    pub idx_flags:              u32,
    /// Slot index of clean-string start in the trailing UTF-8 string table;
    /// only meaningful when `flag_has_clean_str` is set
    pub idx_clean_start:        u32,
    /// Slot index of clean-string end (exclusive) in the string table
    pub idx_clean_end:          u32,

    /// Flag bit: declaration has `!important`
    pub flag_important:        u32,
    /// Flag bit: root/rule has an explicit trailing semicolon in `raws.semicolon`
    pub flag_semi_present:     u32,
    /// Flag bit: the `raws.semicolon` value is `true` (semicolon present after last child)
    pub flag_semi_value:       u32,
    /// Flag bit: a clean string is stored in the trailing table (selector or value
    /// differs from its raw CSS form)
    pub flag_has_clean_str:    u32,
    /// Flag bit: the clean string belongs to field2 (declaration value);
    /// when clear it belongs to field1 (rule selector)
    pub flag_clean_is_field2:  u32,
    /// Flag bit: node has a `nodes` array (rule, at-rule with block)
    pub flag_has_nodes:        u32,
}

impl NodeSlotLayout {
    /// Build the layout descriptor from the actual `NodeSlot` memory layout.
    /// Field indices are derived via `offset_of!` so they stay in sync with
    /// the Rust struct automatically.
    pub fn build() -> Self {
        use std::mem::offset_of;
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

/// Flat binary representation of one PostCSS AST node.
///
/// `#[repr(C)]` guarantees stable field order; all fields are `u32` so there
/// is no padding and `bytemuck::cast_slice` can convert `&[NodeSlot]` to
/// `&[u8]` with zero copies.  Declaration order equals the u32 slot index.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct NodeSlot {
    /// Slot 0 — node kind: see `NodeType` constants
    pub type_tag:           u32,
    /// Slot 1 — index of the parent node in the flat array; ABSENT for root
    pub parent_idx:         u32,
    /// Slot 2 — CSS byte offset of `source.start` (0-based)
    pub source_start_off:   u32,
    /// Slot 3 — 1-based line of `source.start`
    pub source_start_line:  u32,
    /// Slot 4 — 1-based column of `source.start`
    pub source_start_col:   u32,
    /// Slot 5 — CSS byte offset of `source.end`; ABSENT if missing
    pub source_end_off:     u32,
    /// Slot 6 — 1-based line of `source.end`
    pub source_end_line:    u32,
    /// Slot 7 — 1-based column of `source.end`
    pub source_end_col:     u32,
    /// Slot 8 — start of field1 raw: selector (rule) / prop (decl) / name (atrule) / text (comment)
    pub field1_start:       u32,
    /// Slot 9 — end of field1 raw
    pub field1_end:         u32,
    /// Slot 10 — start of field2 raw: value (decl) / params (atrule); ABSENT if absent
    pub field2_start:       u32,
    /// Slot 11 — end of field2 raw
    pub field2_end:         u32,
    /// Slot 12 — start of `raws.before`; ABSENT if absent
    pub raws_before_start:  u32,
    /// Slot 13 — end of `raws.before`
    pub raws_before_end:    u32,
    /// Slot 14 — start of `raws.between`; ABSENT if absent
    pub raws_between_start: u32,
    /// Slot 15 — end of `raws.between`
    pub raws_between_end:   u32,
    /// Slot 16 — start of `raws.after`; ABSENT if absent
    pub raws_after_start:   u32,
    /// Slot 17 — end of `raws.after`
    pub raws_after_end:     u32,
    /// Slot 18 — start of extra1: `raws.afterName` (atrule) / `raws.left` (comment); ABSENT if absent
    pub raws_extra1_start:  u32,
    /// Slot 19 — end of extra1
    pub raws_extra1_end:    u32,
    /// Slot 20 — start of extra2: `raws.important` (decl) / `raws.right` (comment); ABSENT if absent
    pub raws_extra2_start:  u32,
    /// Slot 21 — end of extra2
    pub raws_extra2_end:    u32,
    /// Slot 22 — bit flags; see `Flags` constants
    pub flags:              u32,
    /// Slot 23 — start of clean string in the trailing UTF-8 table (when `Flags::HAS_CLEAN_STR` set)
    pub clean_start:        u32,
    /// Slot 24 — end of clean string in the trailing UTF-8 table
    pub clean_end:          u32,
}

/// Number of u32 slots per node — derived from the struct size at compile time.
pub const STRIDE: usize = std::mem::size_of::<NodeSlot>() / 4;
const _STRIDE_CHECK: () = assert!(std::mem::size_of::<NodeSlot>() / 4 == 25);

/// Serializes a PostCSS AST into the compact offset-array binary format.
///
/// # Buffer layout
///
/// ```text
/// [0..3]          u32 LE  node_count
/// [4 .. 4+n*100]  n × NodeSlot (25 u32s each, little-endian)
/// [4+n*100 ..]    UTF-8 string table (clean values when ≠ raw)
/// ```
///
/// All CSS-offset fields are 0-indexed byte offsets into the source string.
/// `ABSENT` (0xFFFF_FFFF) signals a field that is not present.
pub struct OffsetSerializer;

impl OffsetSerializer {
    /// Serialize `root` into the offset-array binary format.
    pub fn serialize(root: &Node, css: &str) -> Vec<u8> {
        let mut slots: Vec<NodeSlot> = Vec::new();
        let mut string_table: Vec<u8> = Vec::new();
        Self::visit(root, ABSENT, &mut slots, &mut string_table, css);

        let count_bytes = (slots.len() as u32).to_le_bytes();
        let slots_bytes: &[u8] = bytemuck::cast_slice(&slots);
        let mut out = Vec::with_capacity(4 + slots_bytes.len() + string_table.len());
        out.extend_from_slice(&count_bytes);
        out.extend_from_slice(slots_bytes);
        out.extend_from_slice(&string_table);
        out
    }

    /// DFS pre-order traversal: build and push each node's slot, then recurse.
    /// `my_idx = slots.len()` before the push gives the stable index used as
    /// `parent_idx` for all direct children.
    fn visit(node: &Node, parent_idx: u32, slots: &mut Vec<NodeSlot>, string_table: &mut Vec<u8>, css: &str) {
        let my_idx = slots.len() as u32;

        let ss = &node.source.start;
        let src_start = ss.offset as u32;

            let (source_end_off, source_end_line, source_end_col, src_end) =
                if let Some(se) = &node.source.end {
                    (se.offset as u32, se.line as u32, se.column as u32, se.offset as u32)
                } else {
                    (ABSENT, 0, 0, ABSENT)
                };

            let mut node_flags: u32 = 0;
            if node.nodes.is_some() {
                node_flags |= Flags::HAS_NODES;
            }
            if let Some(semi_val) = *node.raws.semicolon {
                node_flags |= Flags::SEMI_PRESENT;
                if semi_val { node_flags |= Flags::SEMI_VALUE; }
            }

            let (raws_before_start, raws_before_end) =
                Self::encode_ends_at(&node.raws.before, src_start);

            let mut slot = NodeSlot {
                type_tag:           0,
                parent_idx,
                source_start_off:   src_start,
                source_start_line:  ss.line as u32,
                source_start_col:   ss.column as u32,
                source_end_off,
                source_end_line,
                source_end_col,
                field1_start:       ABSENT,
                field1_end:         0,
                field2_start:       ABSENT,
                field2_end:         0,
                raws_before_start,
                raws_before_end,
                raws_between_start: ABSENT,
                raws_between_end:   0,
                raws_after_start:   ABSENT,
                raws_after_end:     0,
                raws_extra1_start:  ABSENT,
                raws_extra1_end:    0,
                raws_extra2_start:  ABSENT,
                raws_extra2_end:    0,
                flags:              node_flags,
                clean_start:        0,
                clean_end:          0,
            };

            match &node.data {
                Data::Root => {
                    slot.type_tag = NodeType::ROOT;
                    if src_end != ABSENT {
                        let (as_, ae) = Self::encode_ends_at(&node.raws.after, src_end);
                        slot.raws_after_start = as_;
                        slot.raws_after_end   = ae;
                    }
                }

                Data::Rule(rule) => {
                    slot.type_tag = NodeType::RULE;

                    let sel_raw = node.raws.selector.raw.as_deref().unwrap_or(&rule.selector);
                    // Selector may be preceded by a non-ASCII prefix (e.g. BOM) that is
                    // part of raws.before; find where sel_raw actually starts.
                    let (f1s, f1s_byte) = Self::find_field_start(css, src_start, sel_raw);
                    // The parser moves inter-selector whitespace (e.g. the space in ".a, .b")
                    // into raws.between, so sel_raw (".a,.b") can be shorter than the actual
                    // CSS span.  Walk both simultaneously to find the true CSS end.
                    let actual_f1e = Self::find_selector_css_end(css, f1s_byte, f1s, sel_raw);
                    slot.field1_start = f1s;
                    slot.field1_end   = actual_f1e;

                    // raws.before ends where the selector starts (overrides default src_start value)
                    let (rbs, rbe) = Self::encode_ends_at(&node.raws.before, f1s);
                    slot.raws_before_start = rbs;
                    slot.raws_before_end   = rbe;

                    let (bt_s, bt_e) = Self::encode_starts_at(&node.raws.between, slot.field1_end);
                    slot.raws_between_start = bt_s;
                    slot.raws_between_end   = bt_e;

                    if src_end != ABSENT {
                        let (as_, ae) = Self::encode_ends_at(&node.raws.after, src_end - 1);
                        slot.raws_after_start = as_;
                        slot.raws_after_end   = ae;

                        // ownSemicolon stored in raws_extra1 (otherwise unused for RULE).
                        // When ownSemicolon has a preceding space (e.g. " ;"), the parser
                        // folds the trailing newline into src_end; strip it before slicing.
                        let own_semi_end = match css.as_bytes().get(src_end as usize - 1) {
                            Some(b'\n') | Some(b'\r') => src_end - 1,
                            _ => src_end,
                        };
                        let (os, oe) = Self::encode_ends_at(&node.raws.own_semicolon, own_semi_end);
                        slot.raws_extra1_start = os;
                        slot.raws_extra1_end   = oe;
                    }

                    if node.raws.selector.raw.is_some() {
                        // Inline comments: raw is sel_raw (with comments), clean is comment-stripped.
                        let clean = node.raws.selector.value.as_deref().unwrap_or(&rule.selector);
                        slot.clean_start = string_table.len() as u32;
                        string_table.extend_from_slice(clean.as_bytes());
                        slot.clean_end = string_table.len() as u32;
                        slot.flags |= Flags::HAS_CLEAN_STR; // clean_is_field2 left clear → field1
                    } else if actual_f1e != f1s + Self::js_len(sel_raw) {
                        // Inter-selector spaces: CSS span is wider than the stored selector.
                        // Store the normalized selector as the clean value so the JS side
                        // uses it (rule.selector = normalized) rather than the raw CSS slice.
                        slot.clean_start = string_table.len() as u32;
                        string_table.extend_from_slice(sel_raw.as_bytes());
                        slot.clean_end = string_table.len() as u32;
                        slot.flags |= Flags::HAS_CLEAN_STR;
                    }
                }

                Data::Declaration(decl) => {
                    slot.type_tag = NodeType::DECL;

                    // IE hack: *color, _background — source.start points to the prefix
                    // char, not the prop; find where decl.prop actually begins.
                    let f1s = Self::find_field_js_start(css, src_start, &decl.prop);
                    slot.field1_start = f1s;
                    slot.field1_end   = f1s + Self::js_len(&decl.prop);

                    // raws.before ends where the prop starts (overrides default src_start value)
                    let (rbs, rbe) = Self::encode_ends_at(&node.raws.before, f1s);
                    slot.raws_before_start = rbs;
                    slot.raws_before_end   = rbe;

                    let between_len =
                        node.raws.between.as_deref().map(|b| b.len() as u32).unwrap_or(0);
                    let (bt_s, bt_e) =
                        Self::encode_starts_at(&node.raws.between, slot.field1_end);
                    slot.raws_between_start = bt_s;
                    slot.raws_between_end   = bt_e;
                    let value_start = slot.field1_end + between_len;

                    let val_raw = node.raws.value.raw.as_deref().unwrap_or(&decl.value);
                    slot.field2_start = value_start;
                    slot.field2_end   = value_start + val_raw.len() as u32;

                    let (i_s, i_e) =
                        Self::encode_starts_at(&node.raws.important, slot.field2_end);
                    slot.raws_extra2_start = i_s;
                    slot.raws_extra2_end   = i_e;

                    if decl.important == Some(true) {
                        slot.flags |= Flags::IMPORTANT;
                    }

                    if node.raws.value.raw.is_some() {
                        let clean = node.raws.value.value.as_deref().unwrap_or(&decl.value);
                        slot.clean_start = string_table.len() as u32;
                        string_table.extend_from_slice(clean.as_bytes());
                        slot.clean_end = string_table.len() as u32;
                        slot.flags |= Flags::HAS_CLEAN_STR | Flags::CLEAN_IS_FIELD2;
                    }
                }

                Data::AtRule(at_rule) => {
                    slot.type_tag = NodeType::ATRULE;

                    let name_start = src_start + 1; // skip '@'
                    slot.field1_start = name_start;
                    slot.field1_end   = name_start + at_rule.name.len() as u32;

                    let after_name_len =
                        node.raws.after_name.as_deref().map(|s| s.len() as u32).unwrap_or(0);
                    let (an_s, an_e) =
                        Self::encode_starts_at(&node.raws.after_name, slot.field1_end);
                    slot.raws_extra1_start = an_s;
                    slot.raws_extra1_end   = an_e;
                    let params_start = slot.field1_end + after_name_len;

                    let params_end = if let Some(params) = &at_rule.params {
                        slot.field2_start = params_start;
                        slot.field2_end   = params_start + params.len() as u32;
                        slot.field2_end
                    } else {
                        slot.field2_start = ABSENT;
                        slot.field2_end   = 0;
                        params_start
                    };

                    let (bt_s, bt_e) = Self::encode_starts_at(&node.raws.between, params_end);
                    slot.raws_between_start = bt_s;
                    slot.raws_between_end   = bt_e;

                    if src_end != ABSENT && node.nodes.is_some() {
                        let (as_, ae) = Self::encode_ends_at(&node.raws.after, src_end - 1);
                        slot.raws_after_start = as_;
                        slot.raws_after_end   = ae;
                    }
                }

                Data::Comment(comment) => {
                    slot.type_tag = NodeType::COMMENT;

                    let left_start = src_start + 2; // skip '/*'
                    let left_len =
                        node.raws.left.as_deref().map(|l| l.len() as u32).unwrap_or(0);
                    let (l_s, l_e) =
                        Self::encode_starts_at(&node.raws.left, left_start);
                    slot.raws_extra1_start = l_s;
                    slot.raws_extra1_end   = l_e;
                    let text_start = left_start + left_len;

                    slot.field1_start = text_start;
                    slot.field1_end   = text_start + comment.text.len() as u32;

                    if src_end != ABSENT {
                        let (r_s, r_e) =
                            Self::encode_ends_at(&node.raws.right, src_end - 2); // before '*/'
                        slot.raws_extra2_start = r_s;
                        slot.raws_extra2_end   = r_e;
                    }
                }
            }

        slots.push(slot);

        if let Some(children) = &node.nodes {
            for child in children {
                Self::visit(child, my_idx, slots, string_table, css);
            }
        }
    }

    /// Raw that **ends** at `end_pos` (e.g. `raws.before`).
    #[inline]
    fn encode_ends_at(raw: &Raw, end_pos: u32) -> (u32, u32) {
        match raw.as_deref() {
            None    => (ABSENT, 0),
            Some(s) => (end_pos.saturating_sub(Self::js_len(s)), end_pos),
        }
    }

    /// Raw that **starts** at `start_pos` and extends forward.
    #[inline]
    fn encode_starts_at(raw: &Raw, start_pos: u32) -> (u32, u32) {
        match raw.as_deref() {
            None    => (ABSENT, 0),
            Some(s) => (start_pos, start_pos + Self::js_len(s)),
        }
    }

    /// Find the actual CSS end position of a rule selector.
    ///
    /// The parser moves inter-selector whitespace (e.g. the ` ` in `.a, .b`) into
    /// `raws.between`, so the stored `sel_raw` (`.a,.b`) is shorter than the CSS
    /// span.  This walks both strings simultaneously, skipping CSS whitespace that
    /// is absent from `sel_raw`, and returns the JS-char end position.
    ///
    /// `byte_start` is the **byte** offset in `css` where `sel_raw` begins (may
    /// differ from the JS-char offset `f1s` for non-ASCII prefixes like BOM).
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
                        // inter-selector whitespace in CSS; skip without advancing sel
                    }
                    // else: unexpected mismatch; advance css only
                }
            }
        }

        f1s + Self::js_len(&css[byte_start..byte_start + last_byte])
    }

    /// Find the JS-char start index AND byte offset of `field` in the CSS.
    ///
    /// Returns `(js_char_start, byte_start)`.  For typical ASCII CSS `field`
    /// begins right at `src_start` so both values equal `src_start as usize`.
    /// Edge cases handled:
    ///  - Non-ASCII BOM or other multi-byte prefix at `src_start` (bom.css)
    ///  - IE hack prefix chars (`*`, `_`) for declarations (prop.css)
    fn find_field_start(css: &str, src_start: u32, field: &str) -> (u32, usize) {
        if field.is_empty() {
            return (src_start, src_start as usize);
        }
        let byte_pos = src_start as usize;
        // Fast path: field starts exactly at src_start (all ASCII, no prefix).
        if css.as_bytes().get(byte_pos).copied() == field.as_bytes().first().copied() {
            return (src_start, byte_pos);
        }
        // Slow path: find field in the CSS from byte_pos, then convert the prefix
        // length to UTF-16 code units (JS char count) before adding to src_start.
        let prefix_bytes = css.get(byte_pos..)
            .and_then(|s| s.find(field))
            .unwrap_or(0);
        let prefix_js = Self::js_len(&css[byte_pos..byte_pos + prefix_bytes]);
        (src_start + prefix_js, byte_pos + prefix_bytes)
    }

    /// Convenience wrapper returning only the JS-char start.
    #[inline]
    fn find_field_js_start(css: &str, src_start: u32, field: &str) -> u32 {
        Self::find_field_start(css, src_start, field).0
    }

    /// Length of `s` in UTF-16 code units (i.e. JavaScript `String.length`).
    /// For pure-ASCII strings this equals the byte length.
    #[inline]
    fn js_len(s: &str) -> u32 {
        if s.is_ascii() {
            return s.len() as u32;
        }
        s.chars().map(|c| c.len_utf16() as u32).sum()
    }
}
