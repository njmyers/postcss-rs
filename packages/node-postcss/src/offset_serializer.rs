use bytemuck::{Pod, Zeroable};
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

/// Flat binary representation of one PostCSS AST node.
///
/// `#[repr(C)]` guarantees stable field order; all fields are `u32` so there
/// is no padding and `bytemuck::cast_slice` can convert `&[NodeSlot]` to
/// `&[u8]` with zero copies.
///
/// Field layout (declaration order = u32 slot index, 0-based):
///
/// ```text
///  0  type_tag           0=root 1=rule 2=decl 3=atrule 4=comment
///  1  parent_idx         ABSENT for root / top-level nodes
///  2  source_start_off   CSS byte offset (0-based)
///  3  source_start_line  1-based line number
///  4  source_start_col   1-based column number
///  5  source_end_off     ABSENT if missing
///  6  source_end_line
///  7  source_end_col
///  8  field1_start       selector_raw / prop / at_name / comment_text
///  9  field1_end
/// 10  field2_start       value_raw / at_params   (ABSENT if absent)
/// 11  field2_end
/// 12  raws_before_start  (ABSENT if absent)
/// 13  raws_before_end
/// 14  raws_between_start (ABSENT if absent)
/// 15  raws_between_end
/// 16  raws_after_start   (ABSENT if absent)
/// 17  raws_after_end
/// 18  raws_extra1_start  afterName (atrule) / left (comment)  (ABSENT if absent)
/// 19  raws_extra1_end
/// 20  raws_extra2_start  important_raw (decl) / right (comment)  (ABSENT if absent)
/// 21  raws_extra2_end
/// 22  flags              see `flags` module
/// 23  clean_start        byte offset into string table (when HAS_CLEAN_STR set)
/// 24  clean_end
/// ```
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct NodeSlot {
    pub type_tag:           u32,
    pub parent_idx:         u32,
    pub source_start_off:   u32,
    pub source_start_line:  u32,
    pub source_start_col:   u32,
    pub source_end_off:     u32,
    pub source_end_line:    u32,
    pub source_end_col:     u32,
    pub field1_start:       u32,
    pub field1_end:         u32,
    pub field2_start:       u32,
    pub field2_end:         u32,
    pub raws_before_start:  u32,
    pub raws_before_end:    u32,
    pub raws_between_start: u32,
    pub raws_between_end:   u32,
    pub raws_after_start:   u32,
    pub raws_after_end:     u32,
    pub raws_extra1_start:  u32,
    pub raws_extra1_end:    u32,
    pub raws_extra2_start:  u32,
    pub raws_extra2_end:    u32,
    pub flags:              u32,
    pub clean_start:        u32,
    pub clean_end:          u32,
}

/// Number of u32 slots per node — derived from the struct size at compile time.
pub const STRIDE: usize = std::mem::size_of::<NodeSlot>() / 4;
const _STRIDE_CHECK: () = assert!(std::mem::size_of::<NodeSlot>() / 4 == 25);

// ── helpers ──────────────────────────────────────────────────────────────────

struct Entry<'a> {
    node: &'a Node,
    parent_idx: u32,
}

fn collect<'a>(node: &'a Node, parent: u32, entries: &mut Vec<Entry<'a>>) {
    let my_idx = entries.len() as u32;
    entries.push(Entry { node, parent_idx: parent });
    if let Some(children) = &node.nodes {
        for child in children {
            collect(child, my_idx, entries);
        }
    }
}

/// Raw that **ends** at `end_pos` (e.g. `raws.before`).
#[inline]
fn encode_ends_at(raw: &Raw, end_pos: u32) -> (u32, u32) {
    match raw.as_deref() {
        None    => (ABSENT, 0),
        Some(s) => (end_pos.saturating_sub(s.len() as u32), end_pos),
    }
}

/// Raw that **starts** at `start_pos` and extends forward.
#[inline]
fn encode_starts_at(raw: &Raw, start_pos: u32) -> (u32, u32) {
    match raw.as_deref() {
        None    => (ABSENT, 0),
        Some(s) => (start_pos, start_pos + s.len() as u32),
    }
}

// ── public entry point ────────────────────────────────────────────────────────

/// Serialize a PostCSS AST to a compact binary buffer of byte offsets.
///
/// # Buffer layout
///
/// ```text
/// [0..3]              u32 LE  node_count
/// [4 .. 4+n*100]      n × NodeSlot (25 u32s each, little-endian)
/// [4+n*100 ..]        UTF-8 string table (clean values when ≠ raw)
/// ```
///
/// All CSS-offset fields are 0-indexed byte offsets into `css`.
/// `ABSENT` (0xFFFF_FFFF) means the field is not present.
pub fn to_offset_array(root: &Node, _css: &str) -> Vec<u8> {
    let mut entries: Vec<Entry<'_>> = Vec::new();
    collect(root, ABSENT, &mut entries);

    let n = entries.len();
    let mut slots: Vec<NodeSlot> = Vec::with_capacity(n);
    let mut string_table: Vec<u8> = Vec::new();

    for entry in &entries {
        let node = entry.node;
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

        let (raws_before_start, raws_before_end) = encode_ends_at(&node.raws.before, src_start);

        // Zero-initialise via Zeroable; only set the fields we need.
        let mut slot = NodeSlot {
            type_tag:           0,
            parent_idx:         entry.parent_idx,
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
            // ── Root ─────────────────────────────────────────────────────────
            Data::Root => {
                slot.type_tag = NodeType::ROOT;
                if src_end != ABSENT {
                    let (as_, ae) = encode_ends_at(&node.raws.after, src_end);
                    slot.raws_after_start = as_;
                    slot.raws_after_end   = ae;
                }
            }

            // ── Rule ─────────────────────────────────────────────────────────
            Data::Rule(rule) => {
                slot.type_tag = NodeType::RULE;

                let sel_raw = node.raws.selector.raw.as_deref().unwrap_or(&rule.selector);
                slot.field1_start = src_start;
                slot.field1_end   = src_start + sel_raw.len() as u32;

                let (bt_s, bt_e) = encode_starts_at(&node.raws.between, slot.field1_end);
                slot.raws_between_start = bt_s;
                slot.raws_between_end   = bt_e;

                if src_end != ABSENT {
                    let (as_, ae) = encode_ends_at(&node.raws.after, src_end - 1);
                    slot.raws_after_start = as_;
                    slot.raws_after_end   = ae;
                }

                if node.raws.selector.raw.is_some() {
                    let clean = node.raws.selector.value.as_deref().unwrap_or(&rule.selector);
                    slot.clean_start = string_table.len() as u32;
                    string_table.extend_from_slice(clean.as_bytes());
                    slot.clean_end = string_table.len() as u32;
                    slot.flags |= Flags::HAS_CLEAN_STR; // clean_is_field2 left clear → field1
                }
            }

            // ── Declaration ──────────────────────────────────────────────────
            Data::Declaration(decl) => {
                slot.type_tag = NodeType::DECL;

                slot.field1_start = src_start;
                slot.field1_end   = src_start + decl.prop.len() as u32;

                let between_len = node.raws.between.as_deref().map(|b| b.len() as u32).unwrap_or(0);
                let (bt_s, bt_e) = encode_starts_at(&node.raws.between, slot.field1_end);
                slot.raws_between_start = bt_s;
                slot.raws_between_end   = bt_e;
                let value_start = slot.field1_end + between_len;

                let val_raw = node.raws.value.raw.as_deref().unwrap_or(&decl.value);
                slot.field2_start = value_start;
                slot.field2_end   = value_start + val_raw.len() as u32;

                let (i_s, i_e) = encode_starts_at(&node.raws.important, slot.field2_end);
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

            // ── AtRule ───────────────────────────────────────────────────────
            Data::AtRule(at_rule) => {
                slot.type_tag = NodeType::ATRULE;

                let name_start = src_start + 1; // skip '@'
                slot.field1_start = name_start;
                slot.field1_end   = name_start + at_rule.name.len() as u32;

                let after_name_len = node.raws.after_name.as_deref().map(|s| s.len() as u32).unwrap_or(0);
                let (an_s, an_e) = encode_starts_at(&node.raws.after_name, slot.field1_end);
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

                let (bt_s, bt_e) = encode_starts_at(&node.raws.between, params_end);
                slot.raws_between_start = bt_s;
                slot.raws_between_end   = bt_e;

                if src_end != ABSENT && node.nodes.is_some() {
                    let (as_, ae) = encode_ends_at(&node.raws.after, src_end - 1);
                    slot.raws_after_start = as_;
                    slot.raws_after_end   = ae;
                }
            }

            // ── Comment ──────────────────────────────────────────────────────
            Data::Comment(comment) => {
                slot.type_tag = NodeType::COMMENT;

                let left_start = src_start + 2; // skip '/*'
                let left_len = node.raws.left.as_deref().map(|l| l.len() as u32).unwrap_or(0);
                let (l_s, l_e) = encode_starts_at(&node.raws.left, left_start);
                slot.raws_extra1_start = l_s;
                slot.raws_extra1_end   = l_e;
                let text_start = left_start + left_len;

                slot.field1_start = text_start;
                slot.field1_end   = text_start + comment.text.len() as u32;

                if src_end != ABSENT {
                    let (r_s, r_e) = encode_ends_at(&node.raws.right, src_end - 2); // before '*/'
                    slot.raws_extra2_start = r_s;
                    slot.raws_extra2_end   = r_e;
                }
            }
        }

        slots.push(slot);
    }

    // Build output: [node_count u32 LE][NodeSlot × n bytes][string table]
    let count_bytes = (n as u32).to_le_bytes();
    let slots_bytes: &[u8] = bytemuck::cast_slice(&slots);
    let mut out = Vec::with_capacity(4 + slots_bytes.len() + string_table.len());
    out.extend_from_slice(&count_bytes);
    out.extend_from_slice(slots_bytes);
    out.extend_from_slice(&string_table);
    out
}
