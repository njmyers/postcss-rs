use bytemuck::{Pod, Zeroable};

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
