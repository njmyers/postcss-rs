use napi_derive::napi;

pub use postcss_rs_parser::sinks::offset::{Flags, NodeSlot, NodeType, ABSENT, STRIDE};

/// Binary layout descriptor for the offset-array buffer produced by
/// `parseOffsetsSync`.  All field indices are u32 slot indices within a
/// single node's STRIDE-wide block; multiply by 4 for byte offset.
///
/// Exported via `postcss.nodeSlotLayout()` so TypeScript can use named
/// constants derived directly from the Rust struct rather than magic numbers.
#[napi(object)]
#[allow(dead_code)]
pub struct NodeSlotLayout {
    /// `type_tag` value for a Root node
    pub type_root: u32,
    /// `type_tag` value for a Rule node
    pub type_rule: u32,
    /// `type_tag` value for a Declaration node
    pub type_decl: u32,
    /// `type_tag` value for an AtRule node
    pub type_atrule: u32,
    /// `type_tag` value for a Comment node
    pub type_comment: u32,

    /// Number of u32 slots per node in the buffer (= 25)
    pub stride: u32,
    /// Sentinel value meaning a field is absent/null (= 0xFFFF_FFFF)
    pub absent: u32,

    /// Slot index of the node type tag (see `type_*` constants)
    pub idx_type_tag: u32,
    /// Slot index of the parent node's index in the flat array; `absent` for the root
    pub idx_parent_idx: u32,
    /// Slot index of `source.start` JS char offset
    pub idx_source_start_off: u32,
    /// Slot index of `source.start` 1-based line number
    pub idx_source_start_line: u32,
    /// Slot index of `source.start` 1-based column number
    pub idx_source_start_col: u32,
    /// Slot index of `source.end` JS char offset; `absent` if missing
    pub idx_source_end_off: u32,
    /// Slot index of `source.end` 1-based line number
    pub idx_source_end_line: u32,
    /// Slot index of `source.end` 1-based column number
    pub idx_source_end_col: u32,
    /// Slot index of field1 start: selector (Rule), prop (Decl), name (AtRule), text (Comment)
    pub idx_field1_start: u32,
    /// Slot index of field1 end (exclusive)
    pub idx_field1_end: u32,
    /// Slot index of field2 start: value (Decl), params (AtRule); `absent` if not present
    pub idx_field2_start: u32,
    /// Slot index of field2 end (exclusive)
    pub idx_field2_end: u32,
    /// Slot index of `raws.before` start; `absent` if not present
    pub idx_raws_before_start: u32,
    /// Slot index of `raws.before` end (exclusive)
    pub idx_raws_before_end: u32,
    /// Slot index of `raws.between` start; `absent` if not present
    pub idx_raws_between_start: u32,
    /// Slot index of `raws.between` end (exclusive)
    pub idx_raws_between_end: u32,
    /// Slot index of `raws.after` start; `absent` if not present
    pub idx_raws_after_start: u32,
    /// Slot index of `raws.after` end (exclusive)
    pub idx_raws_after_end: u32,
    /// Slot index of extra1 start: `raws.afterName` (AtRule), `raws.left` (Comment),
    /// `raws.ownSemicolon` (Rule); `absent` if not present
    pub idx_raws_extra1_start: u32,
    /// Slot index of extra1 end (exclusive)
    pub idx_raws_extra1_end: u32,
    /// Slot index of extra2 start: `raws.important` (Decl), `raws.right` (Comment);
    /// `absent` if not present
    pub idx_raws_extra2_start: u32,
    /// Slot index of extra2 end (exclusive)
    pub idx_raws_extra2_end: u32,
    /// Slot index of the bit-flag word (see `flag_*` constants)
    pub idx_flags: u32,
    /// Slot index of clean-string start in the trailing UTF-8 string table;
    /// only meaningful when `flag_has_clean_str` is set
    pub idx_clean_start: u32,
    /// Slot index of clean-string end (exclusive) in the string table
    pub idx_clean_end: u32,

    /// Flag bit: declaration has `!important`
    pub flag_important: u32,
    /// Flag bit: root/rule has an explicit trailing semicolon in `raws.semicolon`
    pub flag_semi_present: u32,
    /// Flag bit: the `raws.semicolon` value is `true` (semicolon present after last child)
    pub flag_semi_value: u32,
    /// Flag bit: a clean string is stored in the trailing table (selector or value
    /// differs from its raw CSS form)
    pub flag_has_clean_str: u32,
    /// Flag bit: the clean string belongs to field2 (declaration value);
    /// when clear it belongs to field1 (rule selector)
    pub flag_clean_is_field2: u32,
    /// Flag bit: node has a `nodes` array (rule, at-rule with block)
    pub flag_has_nodes: u32,
}

impl NodeSlotLayout {
    /// Build the layout descriptor from the actual `NodeSlot` memory layout.
    /// Field indices are derived via `offset_of!` so they stay in sync with
    /// the Rust struct automatically.
    #[allow(dead_code)]
    pub fn build() -> Self {
        use std::mem::offset_of;
        NodeSlotLayout {
            type_root: NodeType::ROOT,
            type_rule: NodeType::RULE,
            type_decl: NodeType::DECL,
            type_atrule: NodeType::ATRULE,
            type_comment: NodeType::COMMENT,

            stride: STRIDE as u32,
            absent: ABSENT,

            idx_type_tag: (offset_of!(NodeSlot, type_tag) / 4) as u32,
            idx_parent_idx: (offset_of!(NodeSlot, parent_idx) / 4) as u32,
            idx_source_start_off: (offset_of!(NodeSlot, source_start_off) / 4) as u32,
            idx_source_start_line: (offset_of!(NodeSlot, source_start_line) / 4) as u32,
            idx_source_start_col: (offset_of!(NodeSlot, source_start_col) / 4) as u32,
            idx_source_end_off: (offset_of!(NodeSlot, source_end_off) / 4) as u32,
            idx_source_end_line: (offset_of!(NodeSlot, source_end_line) / 4) as u32,
            idx_source_end_col: (offset_of!(NodeSlot, source_end_col) / 4) as u32,
            idx_field1_start: (offset_of!(NodeSlot, field1_start) / 4) as u32,
            idx_field1_end: (offset_of!(NodeSlot, field1_end) / 4) as u32,
            idx_field2_start: (offset_of!(NodeSlot, field2_start) / 4) as u32,
            idx_field2_end: (offset_of!(NodeSlot, field2_end) / 4) as u32,
            idx_raws_before_start: (offset_of!(NodeSlot, raws_before_start) / 4) as u32,
            idx_raws_before_end: (offset_of!(NodeSlot, raws_before_end) / 4) as u32,
            idx_raws_between_start: (offset_of!(NodeSlot, raws_between_start) / 4) as u32,
            idx_raws_between_end: (offset_of!(NodeSlot, raws_between_end) / 4) as u32,
            idx_raws_after_start: (offset_of!(NodeSlot, raws_after_start) / 4) as u32,
            idx_raws_after_end: (offset_of!(NodeSlot, raws_after_end) / 4) as u32,
            idx_raws_extra1_start: (offset_of!(NodeSlot, raws_extra1_start) / 4) as u32,
            idx_raws_extra1_end: (offset_of!(NodeSlot, raws_extra1_end) / 4) as u32,
            idx_raws_extra2_start: (offset_of!(NodeSlot, raws_extra2_start) / 4) as u32,
            idx_raws_extra2_end: (offset_of!(NodeSlot, raws_extra2_end) / 4) as u32,
            idx_flags: (offset_of!(NodeSlot, flags) / 4) as u32,
            idx_clean_start: (offset_of!(NodeSlot, clean_start) / 4) as u32,
            idx_clean_end: (offset_of!(NodeSlot, clean_end) / 4) as u32,

            flag_important: Flags::IMPORTANT,
            flag_semi_present: Flags::SEMI_PRESENT,
            flag_semi_value: Flags::SEMI_VALUE,
            flag_has_clean_str: Flags::HAS_CLEAN_STR,
            flag_clean_is_field2: Flags::CLEAN_IS_FIELD2,
            flag_has_nodes: Flags::HAS_NODES,
        }
    }
}
