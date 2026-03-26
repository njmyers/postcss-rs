use postcss_rs_parser::{Data, Node, Raw};

/// Number of u32 slots per node in the flat output array.
pub const STRIDE: usize = 25;

/// Sentinel value indicating an absent/null field.
const ABSENT: u32 = u32::MAX;

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

/// Encode a `Raw` that **ends** at `end_pos` (e.g. `raws.before`, which
/// occupies the bytes immediately before `source.start.offset`).
///
/// Returns `(start, end)` in CSS byte-offset space, or `(ABSENT, 0)` when
/// the raw is absent (`Raw::None`).
#[inline]
fn encode_ends_at(raw: &Raw, end_pos: u32) -> (u32, u32) {
    match raw.as_deref() {
        None => (ABSENT, 0),
        Some(s) => (end_pos.saturating_sub(s.len() as u32), end_pos),
    }
}

/// Encode a `Raw` that **starts** at `start_pos` and extends forward.
///
/// Returns `(start, end)` in CSS byte-offset space, or `(ABSENT, 0)` when
/// the raw is absent.
#[inline]
fn encode_starts_at(raw: &Raw, start_pos: u32) -> (u32, u32) {
    match raw.as_deref() {
        None => (ABSENT, 0),
        Some(s) => (start_pos, start_pos + s.len() as u32),
    }
}

/// Serialize a PostCSS AST to a compact binary buffer of byte offsets.
///
/// # Buffer layout
///
/// ```text
/// [0..3]               u32 LE  node_count
/// [4..4+n*STRIDE*4]    u32 LE  node_count × STRIDE u32 slots (see below)
/// [4+n*STRIDE*4..]     UTF-8   string table (clean values when ≠ raw)
/// ```
///
/// Each node occupies `STRIDE = 25` consecutive u32 slots:
///
/// ```text
/// [0]  type  0=root 1=rule 2=decl 3=atrule 4=comment
/// [1]  parent_idx  (ABSENT = root / no parent)
/// [2]  source_start_offset
/// [3]  source_start_line
/// [4]  source_start_col
/// [5]  source_end_offset   (ABSENT if absent)
/// [6]  source_end_line     (0 if absent)
/// [7]  source_end_col      (0 if absent)
/// [8]  field1_start  selector_raw / prop / at_name / comment_text
/// [9]  field1_end
/// [10] field2_start  value_raw / at_params   (ABSENT if absent)
/// [11] field2_end
/// [12] raws_before_start   (ABSENT if absent)
/// [13] raws_before_end
/// [14] raws_between_start  (ABSENT if absent)
/// [15] raws_between_end
/// [16] raws_after_start    (ABSENT if absent)
/// [17] raws_after_end
/// [18] raws_extra1_start   afterName (atrule) / left (comment)  (ABSENT if absent)
/// [19] raws_extra1_end
/// [20] raws_extra2_start   important_raw (decl) / right (comment)  (ABSENT if absent)
/// [21] raws_extra2_end
/// [22] flags
///        bit 0  important
///        bit 1  semicolon_present
///        bit 2  semicolon_value (only meaningful when bit 1 set)
///        bit 3  has_clean_str  → slot[23..24] = byte range in string table
///        bit 4  clean_is_field2  → clean string belongs to field2 (value), not field1 (selector)
///        bit 5  has_nodes  → node carries a block / children array
/// [23] clean_str_start  (byte offset into string table; valid when flag bit 3 set)
/// [24] clean_str_end
/// ```
///
/// All CSS-offset fields are 0-indexed byte offsets into `css`.
/// 0xFFFF_FFFF (`ABSENT`) means the field is not present.
pub fn to_offset_array(root: &Node, _css: &str) -> Vec<u8> {
    let mut entries: Vec<Entry<'_>> = Vec::new();
    collect(root, ABSENT, &mut entries);

    let n = entries.len();
    // One leading u32 for node_count, then n*STRIDE u32s.
    let mut data: Vec<u32> = vec![0u32; 1 + n * STRIDE];
    data[0] = n as u32;

    let mut string_table: Vec<u8> = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        let node = entry.node;
        let base = 1 + i * STRIDE;
        let s = &mut data[base..base + STRIDE];

        // Slot 1: parent
        s[1] = entry.parent_idx;

        // Slots 2-7: source
        let ss = &node.source.start;
        let src_start = ss.offset as u32;
        s[2] = src_start;
        s[3] = ss.line as u32;
        s[4] = ss.column as u32;

        let src_end: u32 = if let Some(se) = &node.source.end {
            s[5] = se.offset as u32;
            s[6] = se.line as u32;
            s[7] = se.column as u32;
            se.offset as u32
        } else {
            s[5] = ABSENT;
            s[6] = 0;
            s[7] = 0;
            ABSENT
        };

        // Flags
        let mut flags: u32 = 0;
        if node.nodes.is_some() {
            flags |= 0x20;
        }
        // Encode semicolon (present in raws for root/rule/atrule)
        if let Some(semi_val) = *node.raws.semicolon {
            flags |= 0x02;
            if semi_val {
                flags |= 0x04;
            }
        }

        // raws.before always ends at src_start
        let (bs, be) = encode_ends_at(&node.raws.before, src_start);
        s[12] = bs;
        s[13] = be;

        // Defaults for fields not set by every variant
        s[8]  = ABSENT; s[9]  = 0;
        s[10] = ABSENT; s[11] = 0;
        s[14] = ABSENT; s[15] = 0;
        s[16] = ABSENT; s[17] = 0;
        s[18] = ABSENT; s[19] = 0;
        s[20] = ABSENT; s[21] = 0;
        s[23] = 0;      s[24] = 0;

        match &node.data {
            // ── Root ──────────────────────────────────────────────────────────
            Data::Root => {
                s[0] = 0;
                // Root has no field1/field2/between/extra
                // raws.after ends at src_end (no closing brace for root)
                if src_end != ABSENT {
                    let (as_, ae) = encode_ends_at(&node.raws.after, src_end);
                    s[16] = as_;
                    s[17] = ae;
                }
            }

            // ── Rule ──────────────────────────────────────────────────────────
            Data::Rule(rule) => {
                s[0] = 1;

                // field1 = selector_raw (starts at src_start)
                let sel_raw = node.raws.selector.raw.as_deref().unwrap_or(&rule.selector);
                s[8] = src_start;
                s[9] = src_start + sel_raw.len() as u32;
                let sel_end = s[9];

                // raws.between starts at selector end
                let (bt_s, bt_e) = encode_starts_at(&node.raws.between, sel_end);
                s[14] = bt_s;
                s[15] = bt_e;

                // raws.after ends at src_end-1 (just before closing '}')
                if src_end != ABSENT {
                    let (as_, ae) = encode_ends_at(&node.raws.after, src_end - 1);
                    s[16] = as_;
                    s[17] = ae;
                }

                // Clean selector (stored in string table when inline comments stripped)
                if node.raws.selector.raw.is_some() {
                    let clean = node.raws.selector.value.as_deref().unwrap_or(&rule.selector);
                    let st_s = string_table.len() as u32;
                    string_table.extend_from_slice(clean.as_bytes());
                    s[23] = st_s;
                    s[24] = string_table.len() as u32;
                    flags |= 0x08; // has_clean_str, clean_is_field1 (bit4 clear)
                }
            }

            // ── Declaration ───────────────────────────────────────────────────
            Data::Declaration(decl) => {
                s[0] = 2;

                // field1 = prop (starts at src_start)
                s[8] = src_start;
                s[9] = src_start + decl.prop.len() as u32;
                let prop_end = s[9];

                // raws.between starts at prop_end
                let between_len = node.raws.between.as_deref().map(|b| b.len() as u32).unwrap_or(0);
                let (bt_s, bt_e) = encode_starts_at(&node.raws.between, prop_end);
                s[14] = bt_s;
                s[15] = bt_e;
                let value_start = prop_end + between_len;

                // field2 = value_raw (starts after between)
                let val_raw = node.raws.value.raw.as_deref().unwrap_or(&decl.value);
                s[10] = value_start;
                s[11] = value_start + val_raw.len() as u32;
                let val_raw_end = s[11];

                // raws.important (extra2) starts right after value_raw
                let (i_s, i_e) = encode_starts_at(&node.raws.important, val_raw_end);
                s[20] = i_s;
                s[21] = i_e;

                if decl.important == Some(true) {
                    flags |= 0x01;
                }

                // Clean value (stored in string table when inline comments stripped)
                if node.raws.value.raw.is_some() {
                    let clean = node.raws.value.value.as_deref().unwrap_or(&decl.value);
                    let st_s = string_table.len() as u32;
                    string_table.extend_from_slice(clean.as_bytes());
                    s[23] = st_s;
                    s[24] = string_table.len() as u32;
                    flags |= 0x08 | 0x10; // has_clean_str + clean_is_field2
                }
            }

            // ── AtRule ────────────────────────────────────────────────────────
            Data::AtRule(at_rule) => {
                s[0] = 3;

                // field1 = name (starts at src_start+1, after '@')
                let name_start = src_start + 1;
                s[8] = name_start;
                s[9] = name_start + at_rule.name.len() as u32;
                let name_end = s[9];

                // extra1 = after_name (starts at name_end)
                let after_name_len = node.raws.after_name.as_deref().map(|s| s.len() as u32).unwrap_or(0);
                let (an_s, an_e) = encode_starts_at(&node.raws.after_name, name_end);
                s[18] = an_s;
                s[19] = an_e;
                let params_start = name_end + after_name_len;

                // field2 = params (starts at params_start; may be absent)
                let params_end = if let Some(params) = &at_rule.params {
                    s[10] = params_start;
                    s[11] = params_start + params.len() as u32;
                    s[11]
                } else {
                    s[10] = ABSENT;
                    s[11] = 0;
                    params_start
                };

                // raws.between starts at params_end
                let (bt_s, bt_e) = encode_starts_at(&node.raws.between, params_end);
                s[14] = bt_s;
                s[15] = bt_e;

                // raws.after only present when the at-rule has a block
                if src_end != ABSENT && node.nodes.is_some() {
                    let (as_, ae) = encode_ends_at(&node.raws.after, src_end - 1);
                    s[16] = as_;
                    s[17] = ae;
                }
            }

            // ── Comment ───────────────────────────────────────────────────────
            Data::Comment(comment) => {
                s[0] = 4;

                // extra1 = left (starts at src_start+2, after '/*')
                let left_start = src_start + 2;
                let left_len = node.raws.left.as_deref().map(|l| l.len() as u32).unwrap_or(0);
                let (l_s, l_e) = encode_starts_at(&node.raws.left, left_start);
                s[18] = l_s;
                s[19] = l_e;
                let text_start = left_start + left_len;

                // field1 = text (starts after left)
                s[8] = text_start;
                s[9] = text_start + comment.text.len() as u32;

                // extra2 = right (ends at src_end-2, before '*/')
                if src_end != ABSENT {
                    let (r_s, r_e) = encode_ends_at(&node.raws.right, src_end - 2);
                    s[20] = r_s;
                    s[21] = r_e;
                }
            }
        }

        s[22] = flags;
    }

    // Serialise to little-endian bytes, then append string table.
    let mut out: Vec<u8> = Vec::with_capacity(data.len() * 4 + string_table.len());
    for &x in &data {
        out.extend_from_slice(&x.to_le_bytes());
    }
    out.extend_from_slice(&string_table);
    out
}
